use std::sync::Arc;

use xgpui_luma::controls::floating_menu::FloatingMenuTheme;

use crate::controls::floating_menu::floating_menu_binding;
use crate::look::ShadcnLook;

struct ShadcnFloatingMenuTheme {
    theme: ShadcnLook,
}

impl FloatingMenuTheme for ShadcnFloatingMenuTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        let tokens = self.theme.mode_tokens();
        floating_menu_binding(tokens.as_ref(), self.theme.mode(), crate::ShadcnSize::Md)
    }
}

pub fn floating_menu_theme(theme: ShadcnLook) -> Arc<dyn FloatingMenuTheme> {
    Arc::new(ShadcnFloatingMenuTheme { theme: theme.clone() })
}
