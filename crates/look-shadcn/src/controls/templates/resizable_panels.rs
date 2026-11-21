use std::sync::Arc;

use xgpui_luma::controls::resizable_panels::{
    ResizablePanelsLook, ResizablePanelsTheme, ResizeHandleMetrics, StyledResizablePanelsTheme,
};
use xgpui_luma::theme::InteractionState;

use crate::look::ShadcnLook;

/// The Shadcn resizable panels from `.lss` (`style/resizable_panels.lss`); handle sizes from the look's
/// sized stylesheet.
pub fn resizable_panels_theme(theme: ShadcnLook) -> Arc<dyn ResizablePanelsTheme> {
    let styled =
        StyledResizablePanelsTheme::new(theme.style_binding("resizable_panels"), ResizeHandleMetrics::default());
    Arc::new(ShadcnResizablePanelsTheme { theme, styled })
}

struct ShadcnResizablePanelsTheme {
    theme: ShadcnLook,
    styled: StyledResizablePanelsTheme,
}

impl ResizablePanelsTheme for ShadcnResizablePanelsTheme {
    /// Handle pixels for the look's Shadcn size.
    fn resize_handle_metrics(&self) -> ResizeHandleMetrics {
        ResizeHandleMetrics::from_stylesheet(&self.theme.mode_tokens().common_for(self.theme.size()))
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        self.styled.resolve(state)
    }
}
