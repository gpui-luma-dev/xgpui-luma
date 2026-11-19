//! The SDK's own styles, used the way a template would use them.

mod common;

use common::*;
use xgpui_luma_style::{Easing, Motion, Resolved};

#[test]
fn fixtures_compile_and_resolve_without_problems() {
    let theme = sdk_theme();
    for control in ["checkbox", "button", "popup_menu"] {
        for states in [&[][..], &["hover"], &["pressed"], &["checked", "hover"], &["disabled"], &["open"]] {
            theme.resolve(&theme.key(control).states(states));
        }
    }
    assert!(theme.diagnostics().is_empty(), "{}", theme.diagnostics());
}

#[test]
fn checkbox_geometry_comes_from_declared_properties() {
    let theme = sdk_theme();
    let style = theme.resolve(&theme.key("checkbox"));
    let root = style.root();
    assert_eq!(root.length("indicator_size"), Some(18.0)); // control height 36 × 0.5
    assert_eq!(root.length("glyph_size"), Some(12.0)); // 18 − 2 × 3
    assert_eq!(root.length("indicator_radius"), Some(4.0));
    assert_eq!(root.number("disabled_opacity"), Some(0.56));
    assert_eq!(root.height(), Some(36.0));
    assert_eq!(root.radius(), Some(6.0));
}

#[test]
fn lengths_snap_to_device_pixels_unless_exact() {
    let theme = sdk_theme();
    let style = theme.resolve(&theme.key("checkbox").scale(1.25));
    // 18 × 1.25 = 22.5 device px → 23 → 18.4 logical px.
    assert_eq!(style.root().length("indicator_size"), Some(18.4));
    // The label line height is 14 × 18 / 12.5 = 20.16, snapped to 20.
    assert_eq!(style.part("label").line_height(), Some(20.0));
    let one_x = theme.resolve(&theme.key("checkbox"));
    assert_eq!(one_x.part("label").line_height(), Some(20.0));
    // `exact(…)` keeps the radius as authored.
    let style = theme.resolve(&theme.key("checkbox").scale(1.5));
    assert_eq!(style.root().radius(), Some(6.0));
}

#[test]
fn checkbox_indicator_follows_the_state_table() {
    let theme = sdk_theme();
    let indicator = |states: &[&str]| {
        let style = theme.resolve(&theme.key("checkbox").states(states));
        (style.part("indicator").background(), style.part("indicator").border())
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
    let style = theme.resolve(&theme.key("checkbox"));
    let label = style.part("label");
    assert_eq!(label.font_size(), Some(14.0));
    assert_eq!(label.string("font_family"), Some("System UI"));
    assert_color(label.foreground(), sdk_gray(11.18), "label");
}

#[test]
fn motion_is_a_part_property() {
    let theme = sdk_theme();
    let timed = Motion::Timed { duration_ms: 200.0, easing: Easing::OutCubic };
    let checkbox = theme.resolve(&theme.key("checkbox"));
    assert_eq!(checkbox.root().transition(), Some(timed));

    // Presence: the destination state picks the transition. Opening is timed, closing is instant.
    let open = theme.resolve(&theme.key("popup_menu").state("open"));
    let closed = theme.resolve(&theme.key("popup_menu"));
    assert_eq!(open.part("menu").transition(), Some(timed));
    assert_eq!(closed.part("menu").transition(), Some(Motion::Instant));
    assert_eq!(
        closed.part("highlight").transition(),
        Some(Motion::Timed { duration_ms: 180.0, easing: Easing::OutCubic })
    );
}

#[test]
fn shadows_come_from_elevation_tokens() {
    let theme = sdk_theme();
    let style = theme.resolve(&theme.key("popup_menu"));
    assert_eq!(style.part("trigger").shadow("shadow").map(<[_]>::len), Some(1));
    assert_eq!(style.part("menu").shadow("shadow").map(<[_]>::len), Some(2));
    // `shadow = none` leaves the property unset.
    let button = theme.resolve(&theme.key("button"));
    assert_eq!(button.root().get("shadow"), None);
}

#[test]
fn root_properties_are_visible_by_bare_name() {
    let theme = sdk_theme();
    let style = theme.resolve(&theme.key("popup_menu"));
    assert_eq!(style.root().length("menu_offset_y"), Some(4.0)); // gap 8 × 0.5
}

#[test]
fn explain_names_the_winning_rule_and_what_it_beat() {
    let theme = sdk_theme();
    let key = theme.key("checkbox").states(&["checked", "hover"]);
    let why = theme.explain(&key, "indicator", "background");
    let winner = why.winner.expect("a winner");
    assert_eq!(winner.source.as_ref(), "sdk/checkbox.lss");
    assert_eq!(winner.states, ":hover:checked");
    assert_eq!(winner.expression, "palette.state.selected.background");
    let beaten: Vec<_> = why.overridden.iter().map(|r| r.states.as_str()).collect();
    assert_eq!(beaten, [":hover", ":checked", ""]);
}

#[test]
fn docs_come_from_the_style_files() {
    let theme_sheet = xgpui_luma_style::Stylesheet::compile(luma_config(), sdk_sources()).expect("compiles");
    assert_eq!(theme_sheet.doc("checkbox"), Some("An indicator box with a checkmark, and a label."));
    assert_eq!(theme_sheet.doc("checkbox.indicator"), Some("The checkbox box."));
    assert_eq!(theme_sheet.doc("checkbox.glyph_size"), Some("Checkmark size."));
    assert_eq!(theme_sheet.parts("checkbox"), ["root", "indicator", "glyph", "label"]);
}

#[test]
fn checked_animation_lerps_between_resolved_styles() {
    let theme = sdk_theme();
    let off = theme.resolve(&theme.key("checkbox"));
    let on = theme.resolve(&theme.key("checkbox").state("checked"));
    let mid = Resolved::lerp(&off, &on, 0.5);
    let expected = sdk_gray(100.0).mix(sdk_gray(12.0), 0.5);
    assert_color(mid.part("indicator").background(), expected, "halfway");
}
