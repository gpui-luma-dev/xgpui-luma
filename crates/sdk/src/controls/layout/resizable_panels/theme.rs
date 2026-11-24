use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::InteractionState;

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
    pub disabled_opacity: f32,
}

pub trait ResizablePanelsTheme: Send + Sync {
    /// Handle pixels; the SDK's unless a theme reads its own `resizable_panels.lss`.
    fn resize_handle_metrics(&self) -> super::ResizeHandleMetrics {
        super::ResizeHandleMetrics::default()
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook;
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledResizablePanelsTheme::sdk())).clone()
}
