//! The Radix look: indexed native scales, tone, high contrast, `:=` against the SDK, look shadow functions.

mod common;

use common::*;
use xgpui_luma_style::{Color, PartId};

const INDICATOR: PartId = PartId::new("indicator");
const GLYPH: PartId = PartId::new("glyph");

fn step(scale: &str, n: usize) -> Color {
    radix_step(scale, n, "light")
}

#[test]
fn every_axis_combination_resolves_without_problems() {
    let theme = radix_theme("light");
    let checkbox = theme.stylesheet().control("checkbox").expect("declared");
    let problems = theme.check(checkbox);
    assert!(problems.is_empty(), "{problems}");
}

#[test]
fn surface_owns_its_resting_face_against_the_sdk() {
    let theme = radix_theme("light");
    // The SDK has :hover and :pressed fills; Radix's `background := …` stops them.
    for states in [&[][..], &["hover"], &["pressed"]] {
        let style = theme.resolve(&key(&theme, "checkbox", &[], states));
        assert_color(style.part(INDICATOR).background(), step("gray", 2), "unchecked face");
    }
    let style = theme.resolve(&key(&theme, "checkbox", &[], &["checked"]));
    assert_color(style.part(INDICATOR).background(), step("accent", 9), "checked");
    assert_color(style.part(INDICATOR).border(), step("accent", 9), "checked border follows the fill");
}

#[test]
fn tone_switches_the_scale_per_instance() {
    let theme = radix_theme("light");
    let hovered = |tone: &str| {
        theme
            .resolve(&key(&theme, "checkbox", &[("tone", tone)], &["checked", "hover"]))
            .part(INDICATOR)
            .background()
    };
    assert_color(hovered("accent"), step("accent", 10), "accent tone");
    assert_color(hovered("gray"), step("gray", 10), "gray tone");
}

#[test]
fn high_contrast_and_disabled() {
    let theme = radix_theme("light");
    let hc = key(&theme, "checkbox", &[], &["checked", "high_contrast"]);
    let style = theme.resolve(&hc);
    assert_color(style.part(INDICATOR).background(), step("accent", 12), "high contrast fill");
    assert_color(style.part(GLYPH).foreground(), step("accent", 1), "high contrast checkmark");

    let disabled = theme.stylesheet().config().state("disabled").expect("declared state");
    let style = theme.resolve(&hc.state(disabled));
    assert_color(style.part(INDICATOR).background(), Color::TRANSPARENT, "disabled high contrast");
    assert_color(style.part(GLYPH).foreground(), step("gray", 8), "disabled checkmark");
}

#[test]
fn soft_keeps_one_tint_in_every_state() {
    let theme = radix_theme("light");
    for states in [&[][..], &["hover"], &["checked"], &["checked", "hover"], &["checked", "high_contrast"]] {
        let style = theme.resolve(&key(&theme, "checkbox", &[("variant", "soft")], states));
        assert_color(style.part(INDICATOR).background(), step("accent", 5), "soft fill");
        assert_color(style.part(INDICATOR).border(), step("accent", 5), "soft border");
    }
    let style = theme.resolve(&key(&theme, "checkbox", &[("variant", "soft")], &["checked", "high_contrast"]));
    assert_color(style.part(GLYPH).foreground(), step("accent", 12), "soft high-contrast checkmark");
    let style = theme.resolve(&key(&theme, "checkbox", &[("variant", "soft")], &["disabled"]));
    assert_color(style.part(INDICATOR).border(), Color::TRANSPARENT, "soft disabled border");
}

#[test]
fn classic_shadows_come_from_look_functions() {
    let theme = radix_theme("light");
    let resting = theme.resolve(&key(&theme, "checkbox", &[("variant", "classic")], &[]));
    let layers = resting.part(INDICATOR).shadow().expect("resting shadow");
    assert_eq!(layers.len(), 2);
    assert!(layers.iter().all(|l| l.inset));
    assert!(layers[0].color.approx_eq(step("gray", 5), 1e-4));

    // The checked rim reads the checkmark color of the same instance (`glyph.foreground`).
    let checked = theme.resolve(&key(&theme, "checkbox", &[("variant", "classic")], &["checked"]));
    let rim = checked.part(INDICATOR).shadow().expect("checked shadow")[0];
    assert!(rim.color.approx_eq(step("accent", 1).with_alpha(0.35), 1e-4), "{rim:?}");
}

#[test]
fn sizes_set_geometry_and_none_unsets() {
    let theme = radix_theme("light");
    let style = theme.resolve(&key(&theme, "checkbox", &[("size", "1")], &[]));
    assert_eq!(style.root().value("indicator_size").and_then(|v| v.as_number()), Some(14.0));
    assert_eq!(style.root().value("glyph_size").and_then(|v| v.as_number()), Some(9.0));
    // `indicator_radius = none`: the Rust side derives it from the Radix radius prop.
    assert_eq!(style.root().value("indicator_radius"), None);
    assert_eq!(style.root().gap(), Some(8.0));
}

#[test]
fn dark_mode_reads_the_dark_scales() {
    let theme = radix_theme("dark");
    let style = theme.resolve(&key(&theme, "checkbox", &[], &["checked"]));
    assert_color(style.part(INDICATOR).background(), radix_step("accent", 9, "dark"), "dark checked");
    assert_color(style.part(GLYPH).foreground(), radix_step("gray", 12, "dark"), "dark checkmark");
}
