//! Slider property mappings (primary / secondary action styling):
//!
//! | Part  | Token |
//! |-------|-------|
//! | Track | `border` (neutral track rail) |
//! | Fill  | primary: `@action_default` / `@action_layer`; secondary: `@action_foreground` |
//! | Thumb | `first(background,card)` |
//! | Ring  | primary: `@action_default`; secondary: `@action_foreground` |
//!
//! Track uses `border` rather than `muted` because many tweakcn light themes
//! set `--muted` near white (e.g. 98% lightness), which disappears on card panels.

use crate::controls::button::ButtonRadiusPreset;

/// The track and thumb end radii for a corner preset at `size`, from `.lss` (`style/slider.lss`'s
/// `radius` attribute).
pub(crate) fn slider_preset_corners(
    look: &crate::ShadcnLook,
    size: crate::ShadcnSize,
    preset: ButtonRadiusPreset,
) -> (f32, f32) {
    use xgpui_luma::style::lss::{PartId, Value};
    let source = look.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "slider",
        xgpui_luma::style::ThemeSource::Live(std::sync::Arc::new(move || source.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let resolved = binding.attr("radius", preset.key()).resolve(&[], 1.0);
    let radius = |part: &'static str| {
        resolved.part(PartId::new(part)).value("radius").and_then(Value::as_number).unwrap_or_default()
    };
    (radius("track"), radius("thumb"))
}
