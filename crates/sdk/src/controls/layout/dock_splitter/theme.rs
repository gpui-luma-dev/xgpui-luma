#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

/// The dock splitter's geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DockSplitterGeometry {
    pub hit_target: f32,
    pub visible_line: f32,
    pub thumb_width: f32,
    pub thumb_height: f32,
    pub thumb_radius: f32,
    pub thumb_rest_opacity: f32,
}

#[deprecated(
    note = "the dock splitter draws from `DockSplitterTheme::style()`; resolve it with `resolve_dock_splitter` (#433)"
)]
#[derive(Clone, Debug)]
pub struct DockSplitterLook {
    /// Captured, resolved thumb geometry and resting opacity.
    pub geometry: crate::theme::stylesheet::DockSplitterGeometry,
    pub line_color: Hsla,
    pub hover_color: Hsla,
    pub thumb_color: Hsla,
    pub hit_target_px: f32,
    pub visible_line_px: f32,
}

pub trait DockSplitterTheme: Send + Sync {
    /// The splitter's style (#433): the binding the template resolves (see
    /// [`resolve_dock_splitter`](super::resolve_dock_splitter)) and draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("dock_splitter")
    }

    /// The splitter's look, built from [`Self::style`] for code that still reads a [`DockSplitterLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_dock_splitter` (#433)")]
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        DockSplitterLook::from_resolved(&super::resolve_dock_splitter(&self.style(), enabled))
    }
}

pub fn default_dock_splitter_theme() -> Arc<dyn DockSplitterTheme> {
    static THEME: OnceLock<Arc<dyn DockSplitterTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledDockSplitterTheme::sdk())).clone()
}
