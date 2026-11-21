//! The overlay window read from `.lss` (issue #285): `overlay_window.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value, box_shadows};

use super::OverlayWindowMode;
use super::theme::{OverlayWindowLook, OverlayWindowTheme};
use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The overlay window for `binding` (the `overlay_window` control) in `mode`.
pub fn styled_overlay_window_look(binding: &StyleBinding, mode: OverlayWindowMode) -> OverlayWindowLook {
    let attrs: &[(&str, &str)] = match mode {
        OverlayWindowMode::Modal => &[("variant", "modal")],
        _ => &[],
    };
    let resolved = binding.resolve_for(attrs, &[], 1.0);
    let root = resolved.root();
    OverlayWindowLook {
        viewport_padding: number(root, "viewport_padding"),
        window_margin: number(root, "window_margin"),
        drag_header_height: number(root, "drag_header_height"),
        border_width: number(root, "border_width"),
        background: color(root, "background"),
        border: color(root, "border"),
        foreground: color(root, "foreground"),
        backdrop_background: color(root, "backdrop"),
        shadow: root.shadow().map(|layers| box_shadows(&layers)).unwrap_or_default(),
        radius: number(root, "radius"),
        padding: number(root, "padding"),
        min_width: number(root, "min_width"),
        max_width: number(root, "max_width"),
        estimated_height: number(root, "estimated_height"),
        body: LumaTextStyle {
            size: root.font_size().unwrap_or(14.0),
            line_height: root.line_height().unwrap_or(20.0),
            weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
        },
        font_family: root.font_family().unwrap_or("").to_string().into(),
    }
}

/// An overlay-window theme read from `.lss`, resolved on every presentation.
#[derive(Clone)]
pub struct StyledOverlayWindowTheme {
    binding: StyleBinding,
}

impl StyledOverlayWindowTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("overlay_window") }
    }
}

impl OverlayWindowTheme for StyledOverlayWindowTheme {
    fn resolve(&self, mode: OverlayWindowMode) -> OverlayWindowLook {
        styled_overlay_window_look(&self.binding, mode)
    }
}
