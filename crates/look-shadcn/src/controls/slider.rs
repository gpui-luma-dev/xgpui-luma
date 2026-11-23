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

/// Constructs the native Shadcn common slider sizes in pure Rust without preloading TOML.
pub fn native_shadcn_common_slider() -> xgpui_luma::theme::stylesheet::SliderStylesheet {
    use xgpui_luma::theme::stylesheet::{SliderGeometryStylesheet, SliderStylesheet};
    SliderStylesheet {
        geometry: SliderGeometryStylesheet::default(),
        sizes: [("sm", 24.0, 4.0, 12.0), ("md", 32.0, 6.0, 16.0), ("lg", 40.0, 8.0, 20.0)]
            .into_iter()
            .map(|(key, height, track_height, thumb_size)| {
                (
                    key.to_string(),
                    SliderGeometryStylesheet {
                        width: Some(260.0),
                        height: Some(height),
                        track_height: Some(track_height),
                        thumb_size: Some(thumb_size),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}

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
