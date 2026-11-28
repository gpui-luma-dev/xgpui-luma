#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

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
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledResizablePanelsTheme::sdk())).clone()
}
