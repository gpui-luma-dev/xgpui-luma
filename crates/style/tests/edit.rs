//! Editing and exporting `.lss` (API topic 7): edits change text in place; exports round-trip.

mod common;

use std::sync::Arc;

use common::*;
use xgpui_luma_style::{
    Color, Edit, Motion, NoNativeTokens, PartId, RuleSpec, Source, Stylesheet, Theme, Value, export_tokens,
};

const INDICATOR: PartId = PartId::new("indicator");

#[test]
fn replacing_a_rule_value_keeps_the_rest_of_the_file() {
    let theme = radix_theme("light");
    let k = key(&theme, "checkbox", &[], &["checked", "hover"]);
    let winner = theme.explain(&k, INDICATOR, "background").winner.expect("winner");
    assert_eq!(winner.expression, "tone[10]");

    let (sheet, changed) = theme.stylesheet().apply(&[Edit::replace(&winner, "tone[11]")]).expect("applies");
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].name.as_ref(), "radix/checkbox.lss");
    assert!(changed[0].text.contains("{ background = tone[11] }"));
    assert!(changed[0].text.contains("// Radix checkbox"), "comments survive");

    let edited = theme.with_stylesheet(&sheet).expect("binds");
    assert_color(edited.resolve(&k).part(INDICATOR).background(), radix_step("accent", 11, "light"), "edited");
}

#[test]
fn a_bad_edit_returns_diagnostics_and_changes_nothing() {
    let theme = radix_theme("light");
    let k = key(&theme, "checkbox", &[], &["checked", "hover"]);
    let winner = theme.explain(&k, INDICATOR, "background").winner.expect("winner");
    let err = theme.stylesheet().apply(&[Edit::replace(&winner, "tone[")]).err().expect("syntax error");
    assert!(err.has_errors());
}

#[test]
fn inserting_a_rule_creates_the_blocks_it_needs() {
    let sheet = sdk_sheet();
    let rule = RuleSpec::new("checkbox", "indicator", "background", "black")
        .scope("variant", "brand")
        .state("checked");
    let (sheet, changed) = sheet.apply(&[Edit::insert("app", "app/overrides.lss", &rule)]).expect("applies");
    assert_eq!(
        changed[0].text.as_ref(),
        "checkbox {\n  variant brand {\n    part indicator {\n      :checked { background = black }\n    }\n  }\n}\n"
    );
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    let k = key(&theme, "checkbox", &[("variant", "brand")], &["checked"]);
    assert_color(theme.resolve(&k).part(INDICATOR).background(), Color::BLACK, "inserted rule applies");
}

#[test]
fn exported_variants_round_trip() {
    // The lab: edit, then export the edits as a named variant.
    let sheet = sdk_sheet();
    let edits = [
        RuleSpec::new("checkbox", "indicator", "background", "palette.state.hover.background"),
        RuleSpec::new("checkbox", "indicator", "background", "black").state("checked"),
        RuleSpec::new("checkbox", "root", "gap", "12"),
    ];
    let text = sheet.export_scope("checkbox", "variant", "brand", &edits);
    assert_eq!(
        text,
        "checkbox {\n  variant brand {\n    part indicator {\n      background = palette.state.hover.background\n      :checked { background = black }\n    }\n    gap = 12\n  }\n}\n"
    );
    let with_variant = Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source("app", Source::new("app/brand.lss", text))
        .compile()
        .expect("exported text compiles");
    let theme = Theme::bind(&with_variant, Arc::new(NoNativeTokens), "light").expect("binds");
    let brand = |states: &[&str]| theme.resolve(&key(&theme, "checkbox", &[("variant", "brand")], states));
    assert_color(brand(&[]).part(INDICATOR).background(), sdk_gray(91.37), "resting");
    assert_color(brand(&["checked"]).part(INDICATOR).background(), Color::BLACK, "checked");
    assert_eq!(brand(&[]).root().gap(), Some(12.0));
}

#[test]
fn exported_token_values_round_trip() {
    let theme = sdk_theme();
    let red = Color::oklch(0.6, 0.2, 25.0, 1.0);
    let edited = theme
        .with_token_values(&[
            ("palette.form.input.background", Value::Color(red)),
            ("metrics.control.gap", Value::Number(10.0)),
            ("motion.default", Value::Motion(Motion::Instant)),
        ])
        .expect("known tokens");
    let text = export_tokens(&edited.token_values());
    assert!(text.starts_with("tokens {\n  metrics {\n    control {\n      gap = 10\n"), "{text}");

    let sheet = Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source("app", Source::new("app/tokens.lss", text))
        .compile()
        .expect("exported tokens compile");
    let from_file = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    let k = key(&from_file, "checkbox", &[], &[]);
    assert_eq!(from_file.resolve(&k), edited.resolve(&k), "same values from the file as from the live edits");
}

#[test]
fn values_print_as_lss() {
    assert_eq!(Value::Number(12.5).to_lss(), "12.5");
    assert_eq!(Value::Color(Color::oklch(0.5, 0.1, 200.0, 1.0)).to_lss(), "oklch(0.5, 0.1, 200, 1)");
    assert_eq!(Value::Motion(Motion::Instant).to_lss(), "instant");
    assert_eq!(Value::None.to_lss(), "none");
}

#[test]
fn parenthesised_values_are_edited_whole() {
    let sheet = Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source("sdk", Source::new("chip.lss", "chip {\n  a: length = (2 + 3) * 4\n}"))
        .compile()
        .expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    let chip = key(&theme, "chip", &[], &[]);
    let winner = theme.explain(&chip, PartId::ROOT, "a").winner.expect("winner");
    assert_eq!(winner.expression, "(2 + 3) * 4");
    let (sheet, changed) = sheet.apply(&[Edit::replace(&winner, "7")]).expect("applies");
    assert_eq!(changed[0].text.as_ref(), "chip {\n  a: length = 7\n}");
    let theme = theme.with_stylesheet(&sheet).expect("binds");
    assert_eq!(theme.resolve(&chip).root().value("a").and_then(|v| v.as_number()), Some(7.0));
}

#[test]
fn edits_made_against_another_file_version_are_rejected() {
    let theme = radix_theme("light");
    let k = key(&theme, "checkbox", &[], &["checked", "hover"]);
    let fill = theme.explain(&k, INDICATOR, "background").winner.expect("winner");
    let mark = theme.explain(&k, PartId::new("glyph"), "foreground").winner.expect("winner");
    let (edited, _) = theme.stylesheet().apply(&[Edit::replace(&fill, "tone[10]  ")]).expect("applies");
    // `mark` was explained against the old text; its range no longer fits the edited file.
    let err = edited.apply(&[Edit::replace(&mark, "gray[9]")]).err().expect("rejected");
    assert!(err.has_code(xgpui_luma_style::Code::StaleEdit), "{err}");
    // Explaining again against the edited stylesheet gives a fresh range that applies.
    let fresh = theme
        .with_stylesheet(&edited)
        .expect("binds")
        .explain(&k, PartId::new("glyph"), "foreground")
        .winner
        .expect("winner");
    assert!(edited.apply(&[Edit::replace(&fresh, "gray[9]")]).is_ok());
}
