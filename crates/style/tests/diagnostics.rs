//! Load-time and resolve-time problems, as an author would see them.

mod common;

use std::sync::Arc;

use common::*;
use xgpui_luma_style::{Code, ControlId, Diagnostics, NoNativeTokens, Source, StyleKey, Stylesheet, Theme};

/// Compile the SDK fixtures plus one extra file in `layer`.
fn compile_with(layer: &str, text: &str) -> Result<Stylesheet, Diagnostics> {
    Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source(layer, Source::new("test.lss", text))
        .compile()
}

fn errors(layer: &str, text: &str) -> Diagnostics {
    match compile_with(layer, text) {
        Ok(_) => panic!("expected errors for:\n{text}"),
        Err(d) => d,
    }
}

fn first(d: &Diagnostics) -> &xgpui_luma_style::Diagnostic {
    d.iter().next().expect("at least one diagnostic")
}

#[test]
fn unknown_part_lists_the_parts_and_suggests_one() {
    let d = errors("look", "checkbox { part indicatr { background = black } }");
    let e = first(&d);
    assert_eq!(e.code, Code::UnknownPart);
    assert_eq!(
        e.message,
        "unknown part \"indicatr\" for checkbox (parts: root, indicator, glyph, label) — did you mean \"indicator\"?"
    );
    assert_eq!(e.range, Some((16, 24)));
    let shown = d.to_string();
    assert!(shown.starts_with("test.lss:1:17: error:"), "{shown}");
    assert!(shown.contains("checkbox { part indicatr"), "snippet: {shown}");
}

#[test]
fn unknown_property_suggests_a_close_name() {
    let d = errors("look", "checkbox { part indicator { backgroud = black } }");
    assert_eq!(first(&d).code, Code::UnknownProperty);
    assert!(
        d.mentions("\"backgroud\" is not a core or declared property of checkbox — did you mean \"background\"?"),
        "{d}"
    );
}

#[test]
fn declarations_belong_to_the_declaring_layer() {
    let d = errors("look", "checkbox { tick_size: length = 3 }");
    assert_eq!(first(&d).code, Code::Declaration);
    assert!(d.mentions("\"tick_size\" can only be declared in the layer that declares checkbox"), "{d}");
}

#[test]
fn core_properties_need_no_declaration() {
    let d = errors("sdk", "chip { background: color = black }");
    assert!(d.mentions("\"background\" is a core property"), "{d}");
}

#[test]
fn kind_mismatch_names_the_value() {
    let d = errors("look", "checkbox { part indicator { border = metrics.control.radius } }");
    assert_eq!(first(&d).code, Code::KindMismatch);
    assert!(d.mentions("border expects color, found number (metrics.control.radius)"), "{d}");
}

#[test]
fn unknown_state_and_attribute_suggest_close_names() {
    let d = errors("look", "checkbox { part indicator { :hovered { background = black } } }");
    assert_eq!(first(&d).code, Code::UnknownState);
    assert!(d.mentions("unknown state \":hovered\""), "{d}");
    assert!(d.mentions("did you mean \"hover\"?"), "{d}");
    let d = errors("look", "checkbox { varient soft { gap = 4 } }");
    assert_eq!(first(&d).code, Code::UnknownAttribute);
    assert!(
        d.mentions("unknown attribute \"varient\" (attributes: variant, size, tone) — did you mean \"variant\"?"),
        "{d}"
    );
}

#[test]
fn unknown_names_and_token_groups() {
    let d = errors("look", "checkbox { part indicator { background = palette.app } }");
    assert!(d.mentions("\"palette.app\" is a token group, not a value"), "{d}");
    let d = errors("look", "checkbox { part indicator { background = palette.app.backgroud } }");
    assert_eq!(first(&d).code, Code::UnknownName);
    assert!(
        d.mentions("unknown name \"palette.app.backgroud\" — did you mean \"palette.app.background\"?"),
        "{d}"
    );
}

#[test]
fn a_misspelled_token_root_is_reported_at_bind() {
    // An unknown first segment may be a native root (css, gray, …), which only the token source knows.
    let sheet =
        compile_with("look", "checkbox { part indicator { background = pallete.app.background } }").expect("compiles");
    let err = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").err().expect("bind fails");
    assert!(err.mentions("unknown name \"pallete\""), "{err}");
    assert!(err.mentions("did you mean \"palette\"?"), "{err}");
}

#[test]
fn ambiguous_names_are_errors() {
    // A part named like a token group makes `palette.x` ambiguous.
    let d = errors(
        "sdk",
        "chip {\n  part palette { background = black }\n  part root { foreground = palette.background }\n}",
    );
    assert!(d.has_code(Code::AmbiguousName), "{d}");
    assert!(d.mentions("is ambiguous"), "{d}");
}

#[test]
fn variant_extends_must_name_a_known_variant() {
    let d = errors("look", "checkbox { variant ghost extends outlin { gap = 4 } }");
    assert_eq!(first(&d).code, Code::Extends);
    assert!(d.mentions("variant ghost extends unknown variant \"outlin\""), "{d}");
}

#[test]
fn syntax_errors_point_at_the_problem() {
    let d = errors("look", "checkbox {\n  part indicator { background = }\n}");
    let e = first(&d);
    assert_eq!(e.code, Code::Syntax);
    assert_eq!((e.line, e.column), (2, 33));
    assert!(e.message.starts_with("syntax error: expected"), "{}", e.message);
}

#[test]
fn unknown_layers_list_the_layers() {
    let d = Stylesheet::builder(luma_config()).layer("theme", sdk_sources()).compile().err().expect("fails");
    assert_eq!(first(&d).code, Code::UnknownLayer);
    assert!(d.mentions("unknown layer \"theme\" (layers: sdk, look, app)"), "{d}");
}

#[test]
fn unknown_native_roots_and_functions_fail_at_bind() {
    let sheet =
        compile_with("look", "checkbox { part indicator { background = css.primary  border = shift(black, 0.1) } }")
            .expect("compiles: native roots are checked at bind");
    let err = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").err().expect("bind fails");
    assert!(err.mentions("unknown name \"css\""), "{err}");
    assert!(err.has_code(Code::UnknownFunction), "{err}");
}

#[test]
fn unknown_modes_fail_at_bind() {
    let err = Theme::bind(&shadcn_sheet(), Arc::new(ShadcnCatalog::new()), "drak").err().expect("bind fails");
    assert_eq!(first(&err).code, Code::UnknownMode);
    assert!(err.mentions("unknown mode \"drak\" (modes: dark, light) — did you mean \"dark\"?"), "{err}");
}

#[test]
fn cycles_are_reported_when_resolving() {
    let sheet = compile_with("sdk", "chip {\n  a: length = b + 1\n  b: length = a * 2\n}").expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").expect("binds");
    let chip = sheet.control("chip").expect("declared");
    let style = theme.resolve(&StyleKey::new(chip));
    assert_eq!(style.root().value("a"), None);
    let d = theme.diagnostics();
    assert!(d.has_code(Code::Cycle), "{d}");
    assert!(d.mentions("cycle: chip.root.a → chip.root.b → chip.root.a"), "{d}");
}

#[test]
fn problems_can_be_polled_incrementally() {
    let sheet = compile_with("sdk", "chip {\n  a: length = b + 1\n  b: length = a * 2\n}").expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").expect("binds");
    let (none, mark) = theme.diagnostics_since(0);
    assert!(none.is_empty());
    theme.resolve(&StyleKey::new(ControlId::new("chip")));
    let (new, mark) = theme.diagnostics_since(mark);
    assert!(!new.is_empty());
    let (again, _) = theme.diagnostics_since(mark);
    assert!(again.is_empty(), "already seen");
}

#[test]
fn unknown_controls_and_names_are_reported_not_panicked() {
    let theme = sdk_theme();
    let style = theme.resolve(&StyleKey::new(ControlId::new("chekbox")));
    assert_eq!(style.parts().count(), 0);
    assert!(theme.diagnostics().has_code(Code::UnknownControl));
    let err = theme.key_named("checkbox", &[], &["hovered"]).err().expect("unknown state");
    assert_eq!(err.message, "unknown state \"hovered\" — did you mean \"hover\"?");
}

#[test]
fn references_to_another_control_find_the_attribute_from_the_files() {
    // The crate does not assume an attribute called "variant".
    let config = Arc::new(
        xgpui_luma_style::Config::builder()
            .layers(&["sdk"])
            .states(&["hover"])
            .attributes(&["style", "size"])
            .build()
            .unwrap(),
    );
    let src = "popover { offset: length = 4  style panel { offset = 9 } }\nchip { gap = popover.panel.offset }";
    let sheet = Stylesheet::builder(config.clone())
        .layer("sdk", [Source::new("p.lss", src)])
        .compile()
        .expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "").expect("binds");
    assert_eq!(theme.resolve(&StyleKey::new(ControlId::new("chip"))).root().gap(), Some(9.0));

    let unknown = "popover { offset: length = 4  style panel { offset = 9 } }\nchip { gap = popover.pannel.offset }";
    let d = Stylesheet::builder(config.clone())
        .layer("sdk", [Source::new("p.lss", unknown)])
        .compile()
        .err()
        .expect("fails");
    assert!(d.mentions("popover has no scope \"pannel\" — did you mean \"panel\"?"), "{d}");

    let ambiguous = "popover { offset: length = 4  style big { offset = 9 }  size big { offset = 12 } }\nchip { gap = popover.big.offset }";
    let d = Stylesheet::builder(config)
        .layer("sdk", [Source::new("p.lss", ambiguous)])
        .compile()
        .err()
        .expect("fails");
    assert!(d.has_code(Code::AmbiguousName), "{d}");
    assert!(d.mentions("\"big\" is a value of more than one attribute of popover (size, style)"), "{d}");
}
