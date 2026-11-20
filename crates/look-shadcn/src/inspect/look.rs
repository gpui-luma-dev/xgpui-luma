use xgpui_luma::theme::ThemeMode;

use crate::ShadcnLook;
use crate::size::ShadcnSize;

use crate::inspect::controls::{
    ResizablePanelsInspectMetrics, SelectorInspectMetrics, inspect_resizable_panels_metrics, inspect_selector_metrics,
};

/// Inspect-time wrapper around a [`ShadcnLook`] look resolver.
pub struct ShadcnInspect<'a> {
    look: &'a ShadcnLook,
}

impl<'a> ShadcnInspect<'a> {
    pub fn new(look: &'a ShadcnLook) -> Self {
        Self { look }
    }

    pub fn look(&self) -> &ShadcnLook {
        self.look
    }

    fn mode_tokens(&self) -> std::sync::Arc<crate::ShadcnModeTokens> {
        self.look.mode_tokens()
    }

    fn theme_mode(&self) -> ThemeMode {
        self.look.mode()
    }

    pub fn inspect_resizable_panels_metrics(&self, handle_size: crate::ShadcnSize) -> ResizablePanelsInspectMetrics {
        inspect_resizable_panels_metrics(self.look.mode_tokens().as_ref(), handle_size)
    }

    pub fn inspect_selector_metrics(&self, size: ShadcnSize) -> SelectorInspectMetrics {
        inspect_selector_metrics(
            &self.mode_tokens(),
            self.theme_mode(),
            xgpui_luma::controls::selector::SelectorTriggerStyle::Outline,
            size,
        )
    }
}
