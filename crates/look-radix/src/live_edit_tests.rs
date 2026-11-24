//! Live look edits reach controls already bound to the look (#287). Controls bind the look's `.lss`
//! theme live (`textfield::live_source`), and every setter bumps the look's revision, which rebinds the
//! theme. These bind first, like a spawned control, then edit the look.

use xgpui_luma::style::StyleBinding;
use xgpui_luma::style::lss::{Color, PartId};
use xgpui_luma::theme::ThemeMode;

use crate::{Accent, Gray, Look};

fn checkbox(look: &Look) -> StyleBinding {
    StyleBinding::new("checkbox", crate::textfield::live_source(look))
}

fn checked_fill(binding: &StyleBinding) -> Color {
    binding
        .resolve(&[("checked", true)], 1.0)
        .part(PartId::new("indicator"))
        .background()
        .expect("a fill")
}

fn label(binding: &StyleBinding) -> (Color, String) {
    let resolved = binding.resolve(&[], 1.0);
    let label = resolved.part(PartId::new("label"));
    (label.foreground().expect("a color"), label.font_family().expect("a family").to_string())
}

#[test]
fn palette_changes_reach_bound_controls() {
    let look = Look::built_in();
    look.set_mode(ThemeMode::Light);
    let binding = checkbox(&look);
    look.set_palettes(Accent::Indigo, Gray::Auto);
    let indigo = checked_fill(&binding);
    look.set_palettes(Accent::Tomato, Gray::Auto);
    assert_ne!(checked_fill(&binding), indigo, "the checked fill follows the accent");
}

#[test]
fn font_changes_reach_bound_controls() {
    let look = Look::built_in();
    let binding = checkbox(&look);
    let mut fonts = look.typography().font;
    fonts.sans.family = "Replacement Sans".into();
    look.set_fonts(fonts);
    assert_eq!(label(&binding).1, "Replacement Sans");
}

#[test]
fn mode_changes_reach_bound_controls() {
    let look = Look::built_in();
    look.set_mode(ThemeMode::Light);
    let binding = checkbox(&look);
    let light = label(&binding).0;
    look.set_mode(ThemeMode::Dark);
    assert_ne!(label(&binding).0, light, "the label color follows the mode");
}
