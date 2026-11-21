use std::sync::Arc;

use xgpui_luma::controls::dock_splitter::{DockSplitterTheme, StyledDockSplitterTheme};

use crate::look::ShadcnLook;

/// The Shadcn dock splitter from `.lss` (`style/dock_splitter.lss`).
pub fn dock_splitter_theme(theme: ShadcnLook) -> Arc<dyn DockSplitterTheme> {
    Arc::new(StyledDockSplitterTheme::new(theme.style_binding("dock_splitter")))
}
