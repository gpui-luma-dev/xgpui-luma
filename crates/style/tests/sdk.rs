//! The SDK's own styles.

mod common;

use common::*;
use xgpui_luma_style::{Easing, Motion, PartId, Resolved, ValueSource, props};

const INDICATOR: PartId = PartId::new("indicator");
const LABEL: PartId = PartId::new("label");

#[test]
fn every_axis_combination_resolves_without_problems() {
    let theme = sdk_theme();
    for control in ["checkbox", "button", "popup_menu"] {
        let id = theme.stylesheet().control(control).expect("declared");
        let problems = theme.check(id);
        assert!(problems.is_empty(), "{control}: {problems}");
    }
}

#[test]
fn checkbox_geometry_comes_from_declared_properties() {
    let theme = sdk_theme();
    let sheet = theme.stylesheet();
    let checkbox = sheet.control("checkbox").expect("declared");
    let indicator_size = sheet.prop::<f32>(checkbox, "indicator_size").expect("declared length");
    let glyph_size = sheet.prop::<f32>(checkbox, "glyph_size").expect("declared length");
    let opacity = sheet.prop::<f32>(checkbox, "disabled_opacity").expect("declared number");

    let style = theme.resolve(&key(&theme, "checkbox", &[], &[]));
    let root = style.root();
    assert_eq!(root.get(indicator_size), Some(18.0)); // control height 36 × 0.5
    assert_eq!(root.get(glyph_size), Some(12.0)); // 18 − 2 × 3
    assert_eq!(root.get(opacity), Some(0.56));
    assert_eq!(root.get(props::HEIGHT), Some(36.0));
    assert_eq!(root.radius(), Some(6.0));
}

#[test]
fn lengths_snap_to_device_pixels_unless_exact() {
    let theme = sdk_theme();
    let size = theme
        .stylesheet()
        .prop::<f32>(theme.stylesheet().control("checkbox").unwrap(), "indicator_size")
        .unwrap();
    let style = theme.resolve(&key(&theme, "checkbox", &[], &[]).scale(1.25));
    // 18 × 1.25 = 22.5 device px → 23 → 18.4 logical px.
    assert_eq!(style.root().get(size), Some(18.4));
    // Text metrics are never snapped: the label line height stays 14 × 18 / 12.5 = 20.16.
    assert_eq!(style.part(LABEL).line_height(), Some(20.16));
    // Geometry built from other geometry reads snapped values: at 1.5×, inset 3 → 3.33, so the glyph is
    // 18 − 2 × 3.33 = 11.33 (as the native CheckboxScale computes it), not 12.
    let glyph = theme
        .stylesheet()
        .prop::<f32>(theme.stylesheet().control("checkbox").unwrap(), "glyph_size")
        .unwrap();
    let at_1_5 = theme.resolve(&key(&theme, "checkbox", &[], &[]).scale(1.5));
    assert!((at_1_5.root().get(glyph).unwrap() - 34.0 / 3.0).abs() < 1e-4);
    // `exact(…)` keeps the radius as authored.
    let style = theme.resolve(&key(&theme, "checkbox", &[], &[]).scale(1.5));
    assert_eq!(style.root().radius(), Some(6.0));
}

#[test]
fn checkbox_indicator_follows_the_state_table() {
    let theme = sdk_theme();
    let indicator = |states: &[&str]| {
        let style = theme.resolve(&key(&theme, "checkbox", &[], states));
        (style.part(INDICATOR).background(), style.part(INDICATOR).border())
    };
    let input = sdk_gray(100.0);
    let input_border = sdk_gray(65.1);
    let hover = sdk_gray(91.37);
    let pressed = sdk_gray(83.92);
    let selected = sdk_gray(12.0);
    let disabled = sdk_gray(96.08);

    let (bg, border) = indicator(&[]);
    assert_color(bg, input, "resting");
    assert_color(border, input_border, "resting border");
    assert_color(indicator(&["hover"]).0, hover, "hover");
    assert_color(indicator(&["checked"]).0, selected, "checked");
    assert_color(indicator(&["checked", "hover"]).0, selected, "checked + hover");

    // Pressed outranks checked (interaction state at equal state count); the border follows the fill.
    let (bg, border) = indicator(&["checked", "pressed"]);
    assert_color(bg, pressed, "checked + pressed");
    assert_color(border, pressed, "checked + pressed border = background");

    // Pressed and hovered together: only pressed counts.
    assert_color(indicator(&["hover", "pressed"]).0, pressed, "hover + pressed");

    // Disabled wins over checked; the border goes back to the input border.
    let (bg, border) = indicator(&["checked", "disabled", "hover"]);
    assert_color(bg, disabled, "checked + disabled");
    assert_color(border, input_border, "checked + disabled border");
}

#[test]
fn label_reads_typography_tokens() {
    let theme = sdk_theme();
    let style = theme.resolve(&key(&theme, "checkbox", &[], &[]));
    let label = style.part(LABEL);
    assert_eq!(label.font_size(), Some(14.0));
    assert_eq!(label.font_family(), Some("System UI"));
    assert_color(label.foreground(), sdk_gray(11.18), "label");
}

#[test]
fn motion_is_a_part_property() {
    let theme = sdk_theme();
    let timed = Motion::Timed { duration_ms: 200.0, easing: Easing::OutCubic };
    let checkbox = theme.resolve(&key(&theme, "checkbox", &[], &[]));
    assert_eq!(checkbox.root().transition(), Some(timed));

    // Presence: the destination state picks the transition. Opening is timed, closing is instant.
    let menu = PartId::new("menu");
    let open = theme.resolve(&key(&theme, "popup_menu", &[], &["open"]));
    let closed = theme.resolve(&key(&theme, "popup_menu", &[], &[]));
    assert_eq!(open.part(menu).transition(), Some(timed));
    assert_eq!(closed.part(menu).transition(), Some(Motion::Instant));
    let highlight = closed.part(PartId::new("highlight")).transition();
    assert_eq!(highlight, Some(Motion::Timed { duration_ms: 180.0, easing: Easing::OutCubic }));
}

#[test]
fn shadows_come_from_elevation_tokens() {
    let theme = sdk_theme();
    let style = theme.resolve(&key(&theme, "popup_menu", &[], &[]));
    assert_eq!(style.part(PartId::new("trigger")).shadow().map(<[_]>::len), Some(1));
    assert_eq!(style.part(PartId::new("menu")).get(props::SHADOW).map(|s| s.len()), Some(2));
    // `shadow = none` leaves the property unset.
    let button = theme.resolve(&key(&theme, "button", &[], &[]));
    assert_eq!(button.root().shadow(), None);
}

#[test]
fn root_properties_are_visible_by_bare_name() {
    let theme = sdk_theme();
    let style = theme.resolve(&key(&theme, "popup_menu", &[], &[]));
    assert_eq!(style.root().value("menu_offset_y").and_then(|v| v.as_number()), Some(4.0)); // gap 8 × 0.5
}

#[test]
fn explain_names_the_winning_rule_and_what_it_beat() {
    let theme = sdk_theme();
    let why = theme.explain(&key(&theme, "checkbox", &[], &["checked", "hover"]), INDICATOR, "background");
    assert_eq!(why.source, ValueSource::Rule);
    let winner = why.winner.expect("a winner");
    assert_eq!(winner.source.as_ref(), "sdk/checkbox.lss");
    assert_eq!(winner.states, ":hover:checked");
    assert_eq!(winner.expression, "palette.state.selected.background");
    let beaten: Vec<_> = why.overridden.iter().map(|r| r.states.as_str()).collect();
    assert_eq!(beaten, [":hover", ":checked", ""]);
}

#[test]
fn docs_parts_and_modes_come_from_the_style_files() {
    let sheet = sdk_sheet();
    let checkbox = sheet.control("checkbox").expect("declared");
    assert_eq!(sheet.doc("checkbox"), Some("An indicator box with a checkmark, and a label."));
    assert_eq!(sheet.doc("checkbox.indicator"), Some("The checkbox box."));
    assert_eq!(sheet.doc("checkbox.glyph_size"), Some("Checkmark size."));
    let parts: Vec<_> = sheet.parts(checkbox).iter().map(|p| p.name()).collect();
    assert_eq!(parts, ["root", "indicator", "glyph", "label"]);
    assert_eq!(sheet.modes(), ["light", "dark"], "the config declares the SDK's modes");
}

#[test]
fn checked_animation_lerps_between_resolved_styles() {
    let theme = sdk_theme();
    let off = theme.resolve(&key(&theme, "checkbox", &[], &[]));
    let on = theme.resolve(&key(&theme, "checkbox", &[], &["checked"]));
    let mid = Resolved::lerp(&off, &on, 0.5);
    let expected = sdk_gray(100.0).mix(sdk_gray(12.0), 0.5);
    assert_color(mid.part(INDICATOR).background(), expected, "halfway");
}
