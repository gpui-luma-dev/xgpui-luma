use std::sync::Arc;

use xgpui_luma::controls::split_view::{SplitViewTheme, StyledSplitViewTheme};

use crate::look::ShadcnLook;

/// The Shadcn split view from `.lss` (`style/split_view.lss`).
pub fn split_view_theme(theme: ShadcnLook) -> Arc<dyn SplitViewTheme> {
    Arc::new(StyledSplitViewTheme::new(theme.style_binding("split_view")))
}
