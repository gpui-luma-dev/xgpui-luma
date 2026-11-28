use std::sync::{Arc, OnceLock};

pub trait FloatingMenuTheme: Send + Sync {
    /// The menu's style (#433): the binding menu templates resolve and draw from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("floating_menu")
    }
}

pub fn default_floating_menu_theme() -> Arc<dyn FloatingMenuTheme> {
    static THEME: OnceLock<Arc<dyn FloatingMenuTheme>> = OnceLock::new();

    THEME.get_or_init(super::styled::sdk_theme).clone()
}
