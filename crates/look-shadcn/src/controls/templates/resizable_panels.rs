use std::sync::Arc;

use xgpui_luma::controls::resizable_panels::{ResizablePanelsTheme, StyledResizablePanelsTheme};

use crate::look::ShadcnLook;

/// The Shadcn resizable panels from `.lss` (`style/resizable_panels.lss`), handle sizes included.
pub fn resizable_panels_theme(theme: ShadcnLook) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(StyledResizablePanelsTheme::new(theme.style_binding("resizable_panels")))
}
