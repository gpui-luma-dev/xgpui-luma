#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::style::{PartRead, StyleBinding};
use crate::theme::LumaTextStyle;

use super::OverlayWindowMode;
use super::styled::resolve_overlay_window;

#[deprecated(
    note = "the overlay window draws from `OverlayWindowTheme::style()`; resolve it with `resolve_overlay_window` (#433)"
)]
#[derive(Clone, Debug)]
pub struct OverlayWindowLook {
    /// Placement inset, clamp margin and draggable header hit height in pixels.
    pub viewport_padding: f32,
    pub window_margin: f32,
    pub drag_header_height: f32,

    /// Shell border thickness in pixels.
    pub border_width: f32,
    pub background: Hsla,
    pub border: Hsla,
    pub foreground: Hsla,
    pub backdrop_background: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub estimated_height: f32,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

impl OverlayWindowLook {
    /// The look `resolved` (an overlay window's resolved style, see [`resolve_overlay_window`]) describes: the
    /// compatibility path for code that still reads an `OverlayWindowLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let root = resolved.root();
        Self {
            viewport_padding: root.number("viewport_padding"),
            window_margin: root.number("window_margin"),
            drag_header_height: root.number("drag_header_height"),
            border_width: root.number("border_width"),
            background: root.color("background"),
            border: root.color("border"),
            foreground: root.color("foreground"),
            backdrop_background: root.color("backdrop"),
            shadow: root.box_shadow(),
            radius: root.number("radius"),
            padding: root.number("padding"),
            min_width: root.number("min_width"),
            max_width: root.number("max_width"),
            estimated_height: root.number("estimated_height"),
            body: root.typography(14.0, 20.0),
            font_family: root.font_family_name(),
        }
    }
}

pub trait OverlayWindowTheme: Send + Sync {
    /// The overlay window's style (#433): the binding the template resolves (see [`resolve_overlay_window`]) and
    /// draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("overlay_window")
    }

    /// The overlay window's look, built from [`Self::style`] for code that still reads an [`OverlayWindowLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_overlay_window` (#433)")]
    fn resolve(&self, mode: OverlayWindowMode) -> OverlayWindowLook {
        OverlayWindowLook::from_resolved(&resolve_overlay_window(&self.style(), mode))
    }
}

pub fn default_overlay_window_theme() -> Arc<dyn OverlayWindowTheme> {
    static THEME: OnceLock<Arc<dyn OverlayWindowTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledOverlayWindowTheme::sdk())).clone()
}
