use std::sync::Arc;

use xgpui_luma::controls::split_view::SplitViewTheme;

use crate::controls::split_view::split_view_look;
use crate::look::ShadcnLook;

pub fn split_view_theme(theme: ShadcnLook) -> Arc<dyn SplitViewTheme> {
    Arc::new(ShadcnSplitViewTheme { theme: theme.clone() })
}

struct ShadcnSplitViewTheme {
    theme: ShadcnLook,
}

impl SplitViewTheme for ShadcnSplitViewTheme {
    fn resolve(&self, hovered: bool, enabled: bool) -> xgpui_luma::controls::split_view::SplitViewLook {
        let tokens = self.theme.mode_tokens();
        split_view_look(tokens.as_ref(), self.theme.mode(), hovered, enabled)
    }
}
