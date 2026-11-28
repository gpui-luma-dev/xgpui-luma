#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::InteractionState;

#[deprecated(
    note = "resizable panels draw from `ResizablePanelsTheme::style()`; resolve it with `resolve_resizable_panels` (#433)"
)]
#[derive(Clone, Debug)]
pub struct ResizablePanelsLook {
    pub border: Hsla,
    /// Outer frame thickness in pixels, when the frame is enabled.
    pub border_width: f32,
    /// Painted split divider thickness in pixels; independent of the resize hit target.
    pub divider_width: f32,
    pub divider: Hsla,
    pub grip: Hsla,
    pub grip_emphasis: Hsla,
    /// The resize handles' fade in this state: the root's resolved `opacity` (`:disabled { opacity }`), 1 when unset.
    pub opacity: f32,
}

pub trait ResizablePanelsTheme: Send + Sync {
    /// Handle pixels; the SDK's unless a theme reads its own `resizable_panels.lss`.
    fn resize_handle_metrics(&self) -> super::ResizeHandleMetrics {
        super::ResizeHandleMetrics::default()
    }

    /// The panels' style (#433): the binding the template resolves (see
    /// [`resolve_resizable_panels`](super::resolve_resizable_panels)) and draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("resizable_panels")
    }

    /// The panels' look, built from [`Self::style`] for code that still reads a [`ResizablePanelsLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_resizable_panels` (#433)")]
    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        ResizablePanelsLook::from_resolved(&super::resolve_resizable_panels(&self.style(), state))
    }
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledResizablePanelsTheme::sdk())).clone()
}
