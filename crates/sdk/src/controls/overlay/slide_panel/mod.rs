//! Edge-anchored drawer overlay (app-owned state + templates).
//!
//! Not a spawnable LMTP entity — there is no `SlidePanel` control. Layout:
//! - `model` — [`SlidePanelState`] / edge / size config
//! - `template` — overlay render plus resize-handle chrome (`template/handle`, `template/resize`)
//!
//! Apps own [`SlidePanelState`] and call [`render_slide_panel_overlay`].

mod model;
mod template;

pub use model::{SlidePanelEdge, SlidePanelSizeConfig, SlidePanelState, SlidePanelTopAnchor};

pub use template::{
    SlidePanelOverlayHandlers, SlidePanelResizeDrag, SlidePanelResizeHandlers, SlidePanelResizeHitHandlers,
    render_slide_panel_inner_resize_handle, render_slide_panel_inset, render_slide_panel_overlay,
    render_slide_panel_resize_handle, render_slide_panel_surface,
};

pub use model::SlidePanelMotion;

/// The drawer's size, from `slide_panel.lss`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SlidePanelGeometry {
    /// A side drawer's width.
    pub side_width: f32,
    /// A top or bottom drawer's height.
    pub edge_height: f32,
    /// The top or bottom drawer's height cap, as a share of the viewport height.
    pub edge_max_viewport_ratio: f32,
}

impl SlidePanelGeometry {
    /// The size `binding` (a `slide_panel` control) resolves to.
    pub fn from_binding(binding: &crate::style::StyleBinding) -> Self {
        let resolved = binding.resolve(&[], 1.0);
        let root = resolved.root();
        let number = |name| root.value(name).and_then(xgpui_luma_style::Value::as_number).unwrap_or(0.0);
        Self {
            side_width: number("side_width"),
            edge_height: number("edge_height"),
            edge_max_viewport_ratio: number("edge_max_viewport_ratio"),
        }
    }

    /// The SDK's own drawer size.
    pub fn sdk() -> Self {
        Self::from_binding(&crate::style::StyleBinding::sdk("slide_panel"))
    }
}
