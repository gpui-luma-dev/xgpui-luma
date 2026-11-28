use std::sync::{Arc, OnceLock};

use crate::style::StyleBinding;

pub trait OverlayWindowTheme: Send + Sync {
    /// The overlay window's style (#433): the binding the template resolves (see [`resolve_overlay_window`]) and
    /// draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("overlay_window")
    }
}

pub fn default_overlay_window_theme() -> Arc<dyn OverlayWindowTheme> {
    static THEME: OnceLock<Arc<dyn OverlayWindowTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledOverlayWindowTheme::sdk())).clone()
}
