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
    fn resize_handle_metrics(
        &self,
        size: xgpui_luma::controls::resizable_panels::ResizeHandleSize,
    ) -> xgpui_luma::controls::resizable_panels::ResizeHandleMetrics {
        size.metrics_with_stylesheet(&self.theme.mode_tokens().stylesheet().common)
    }

    fn resolve(&self, state: InteractionState) -> xgpui_luma::controls::resizable_panels::ResizablePanelsLook {
        let tokens = self.theme.mode_tokens();
        resizable_panels_look(tokens.as_ref(), self.theme.mode(), state)
    }
}
