use std::sync::Arc;

use xgpui_luma::controls::resizable_panels::ResizablePanelsTheme;
use xgpui_luma::theme::InteractionState;

use crate::controls::resizable_panels::resizable_panels_look;
use crate::look::ShadcnLook;

pub fn resizable_panels_theme(theme: ShadcnLook) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(ShadcnResizablePanelsTheme { theme: theme.clone() })
}

struct ShadcnResizablePanelsTheme {
    theme: ShadcnLook,
}

impl ResizablePanelsTheme for ShadcnResizablePanelsTheme {
    /// Handle pixels for the Look's Shadcn size.
    fn resize_handle_metrics(&self) -> xgpui_luma::controls::resizable_panels::ResizeHandleMetrics {
        let tokens = self.theme.mode_tokens();
        xgpui_luma::controls::resizable_panels::ResizeHandleMetrics::from_stylesheet(
            &tokens.common_for(self.theme.size()),
        )
    }

    fn resolve(&self, state: InteractionState) -> xgpui_luma::controls::resizable_panels::ResizablePanelsLook {
        let tokens = self.theme.mode_tokens();
        resizable_panels_look(tokens.as_ref(), self.theme.mode(), state)
    }
}
