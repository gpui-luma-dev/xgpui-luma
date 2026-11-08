use std::sync::Arc;

use xgpui_luma::controls::floating_menu::FloatingMenuTheme;

use crate::controls::floating_menu::floating_menu_look;
use crate::look::ShadcnLook;

struct ShadcnFloatingMenuTheme {
    theme: ShadcnLook,
}

impl FloatingMenuTheme for ShadcnFloatingMenuTheme {
    fn resolve(&self) -> xgpui_luma::controls::floating_menu::FloatingMenuLook {
        let tokens = self.theme.mode_tokens();
        floating_menu_look(tokens.as_ref(), self.theme.mode(), xgpui_luma::theme::ControlSize::Md)
    }
}

pub fn floating_menu_theme(theme: ShadcnLook) -> Arc<dyn FloatingMenuTheme> {
    Arc::new(ShadcnFloatingMenuTheme { theme: theme.clone() })
}
