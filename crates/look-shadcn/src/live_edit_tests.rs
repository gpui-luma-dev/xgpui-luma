//! Live look edits reach controls already bound to the look (#287). Controls bind the look's `.lss`
//! theme live (`ShadcnLook::style_binding`), and the look rebinds its theme when its tokens or mode
//! change. These bind first, like a spawned control, then edit the look.

use std::collections::HashMap;

use xgpui_luma::color::ColorValue;
use xgpui_luma::style::StyleBinding;
use xgpui_luma::style::lss::{Color, PartId};
use xgpui_luma::theme::ThemeMode;

use crate::ShadcnLook;

const RETRO_ARCADE: &str = include_str!("../tests/fixtures/tweakcn/retro-arcade.css");

fn fallback_look() -> ShadcnLook {
    let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
    look.set_mode(ThemeMode::Light);
    look
}

fn checked_fill(binding: &StyleBinding) -> Color {
    binding
        .resolve(&[("checked", true)], 1.0)
        .part(PartId::new("indicator"))
        .background()
        .expect("a fill")
}

fn button_radius(binding: &StyleBinding) -> f32 {
    binding.resolve(&[], 1.0).root().radius().expect("a radius")
}

#[test]
fn color_overrides_reach_bound_controls() {
    let look = fallback_look();
    let checkbox = look.style_binding("checkbox");
    let before = checked_fill(&checkbox);
    let red = ColorValue::srgb(1.0, 0.0, 0.0, 1.0);
    look.apply_color_overrides(&HashMap::from([("primary".to_string(), red)])).expect("overrides");
    let after = checked_fill(&checkbox);
    assert_ne!(after, before, "the checked fill follows --primary");
    assert!(after.approx_eq(Color::srgb(1.0, 0.0, 0.0, 1.0), 0.02), "{after:?}");
}

#[test]
fn token_overrides_reach_bound_controls() {
    let look = fallback_look();
    let button = look.style_binding("button");
    let before = button_radius(&button);
    look.apply_token_overrides(&HashMap::from([("radius".to_string(), "1.5rem".to_string())]))
        .expect("overrides");
    assert!(button_radius(&button) > before, "the corner follows --radius");
}

#[test]
fn replacing_the_theme_reaches_bound_controls() {
    let look = fallback_look();
    let checkbox = look.style_binding("checkbox");
    let other = ShadcnLook::from_css_str(RETRO_ARCADE).expect("catalog");
    other.set_mode(ThemeMode::Light);
    assert_ne!(checked_fill(&checkbox), checked_fill(&other.style_binding("checkbox")), "the themes differ");
    look.replace_theme(&other);
    assert_eq!(checked_fill(&checkbox), checked_fill(&other.style_binding("checkbox")));
}

#[test]
fn changing_the_mode_reaches_bound_controls() {
    let label = |binding: &StyleBinding| {
        binding.resolve(&[], 1.0).part(PartId::new("label")).foreground().expect("a label color")
    };
    let look = fallback_look();
    let checkbox = look.style_binding("checkbox");
    let light = label(&checkbox);
    look.set_mode(ThemeMode::Dark);
    let dark = label(&checkbox);
    assert_ne!(light, dark, "the label follows --foreground, which differs by mode");
    let reference = fallback_look();
    reference.set_mode(ThemeMode::Dark);
    assert_eq!(dark, label(&reference.style_binding("checkbox")));
}
