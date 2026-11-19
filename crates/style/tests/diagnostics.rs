//! Load-time and resolve-time problems, as an author would see them.

mod common;

use std::sync::Arc;

use common::*;
use xgpui_luma_style::{Diagnostics, NoNativeTokens, Source, Stylesheet, Theme};

/// Compile the SDK fixtures plus one extra file in `layer`, expecting errors.
fn errors(layer: &str, text: &str) -> Diagnostics {
    let mut sources = sdk_sources();
    sources.push(Source::new(layer, "test.lss", text));
    match Stylesheet::compile(luma_config(), sources) {
        Ok(_) => panic!("expected errors for:\n{text}"),
        Err(d) => d,
    }
}

fn first_message(d: &Diagnostics) -> String {
    d.iter().next().map(|d| d.message.clone()).unwrap_or_default()
}

#[test]
fn unknown_part_lists_the_parts() {
    let d = errors("look", "checkbox { part indicatr { background = black } }");
    assert_eq!(first_message(&d), "unknown part \"indicatr\" for checkbox (parts: root, indicator, glyph, label)");
    let shown = d.to_string();
    assert!(shown.starts_with("test.lss:1:17: error:"), "{shown}");
    assert!(shown.contains("checkbox { part indicatr"), "snippet: {shown}");
}

#[test]
fn unknown_property() {
    let d = errors("look", "checkbox { part indicator { backgroud = black } }");
    assert!(d.mentions("\"backgroud\" is not a core or declared property of checkbox"), "{d}");
}

#[test]
fn declarations_belong_to_the_declaring_layer() {
    let d = errors("look", "checkbox { tick_size: length = 3 }");
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
    assert!(d.mentions("border expects color, found number (metrics.control.radius)"), "{d}");
}

#[test]
fn unknown_state_and_attribute() {
    let d = errors("look", "checkbox { part indicator { :hovered { background = black } } }");
    assert!(d.mentions("unknown state \":hovered\""), "{d}");
    let d = errors("look", "checkbox { flavor soft { gap = 4 } }");
    assert!(d.mentions("unknown attribute \"flavor\" (attributes: variant, size, tone)"), "{d}");
}

#[test]
fn unknown_names_and_token_groups() {
    let d = errors("look", "checkbox { part indicator { background = palette.app } }");
    assert!(d.mentions("\"palette.app\" is a token group, not a value"), "{d}");
    let d = errors("look", "checkbox { part indicator { background = palette.app.backgroud } }");
    assert!(d.mentions("unknown name \"palette.app.backgroud\""), "{d}");
    let d = errors("look", "checkbox { part indicator { background = shade } }");
    assert!(d.mentions("unknown name \"shade\""), "{d}");
}

#[test]
fn a_misspelled_token_root_is_reported_at_bind() {
    // An unknown first segment may be a native root (css, gray, …), which only the token source knows.
    let mut sources = sdk_sources();
    sources.push(Source::new(
        "look",
        "test.lss",
        "checkbox { part indicator { background = pallete.app.background } }",
    ));
    let sheet = Stylesheet::compile(luma_config(), sources).expect("compiles");
    let err = match Theme::bind(&sheet, Arc::new(NoNativeTokens), "light") {
        Ok(_) => panic!("bind should fail"),
        Err(d) => d,
    };
    assert!(err.mentions("unknown name \"pallete\""), "{err}");
}

#[test]
fn ambiguous_names_are_errors() {
    // A part named like a token group makes `palette.x` ambiguous.
    let d = errors(
        "sdk",
        "chip {\n  part palette { background = black }\n  part root { foreground = palette.background }\n}",
    );
    assert!(d.mentions("is ambiguous"), "{d}");
}

#[test]
fn variant_extends_must_name_a_known_variant() {
    let d = errors("look", "checkbox { variant ghost extends outlin { gap = 4 } }");
    assert!(d.mentions("variant ghost extends unknown variant \"outlin\""), "{d}");
}

#[test]
fn syntax_errors_point_at_the_problem() {
    let d = errors("look", "checkbox {\n  part indicator { background = }\n}");
    let first = d.iter().next().expect("one error");
    assert_eq!((first.line, first.column), (2, 33));
    assert!(first.message.starts_with("syntax error: expected"), "{}", first.message);
}

#[test]
fn unknown_native_roots_and_functions_fail_at_bind() {
    let mut sources = sdk_sources();
    sources.push(Source::new(
        "look",
        "test.lss",
        "checkbox { part indicator { background = css.primary  border = shift(black, 0.1) } }",
    ));
    let sheet = Stylesheet::compile(luma_config(), sources).expect("compiles: native roots are checked at bind");
    let err = match Theme::bind(&sheet, Arc::new(NoNativeTokens), "light") {
        Ok(_) => panic!("bind should fail"),
        Err(d) => d,
    };
    assert!(err.mentions("unknown name \"css\""), "{err}");
    assert!(err.mentions("unknown function \"shift\""), "{err}");
}

#[test]
fn cycles_are_reported_when_resolving() {
    let mut sources = sdk_sources();
    sources.push(Source::new("sdk", "test.lss", "chip {\n  a: length = b + 1\n  b: length = a * 2\n}"));
    let sheet = Stylesheet::compile(luma_config(), sources).expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    let style = theme.resolve(&theme.key("chip"));
    assert_eq!(style.root().get("a"), None);
    assert!(
        theme.diagnostics().mentions("cycle: chip.root.a → chip.root.b → chip.root.a"),
        "{}",
        theme.diagnostics()
    );
}

#[test]
fn unknown_keys_are_reported_not_panicked() {
    let theme = sdk_theme();
    let style = theme.resolve(&theme.key("checkbox").state("hovered"));
    assert_eq!(style.parts().count(), 0);
    assert!(theme.diagnostics().mentions("unknown state \"hovered\""));
}
