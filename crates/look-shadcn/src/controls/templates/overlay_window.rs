use std::sync::Arc;

use xgpui_luma::controls::overlay_window::{
    OverlayWindowLook, OverlayWindowMode, OverlayWindowTemplate, OverlayWindowTheme, ThemedOverlayWindowTemplate,
};
use xgpui_luma::theme::ControlSize;

use crate::controls::overlay_window::overlay_window_look;
use crate::look::ShadcnLook;

pub fn overlay_window_template(theme: ShadcnLook) -> Arc<dyn OverlayWindowTemplate> {
    Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(theme)))
}

pub fn overlay_window_theme(theme: ShadcnLook) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(ShadcnOverlayWindowTheme { theme: theme.clone() })
}

struct ShadcnOverlayWindowTheme {
    theme: ShadcnLook,
}

impl OverlayWindowTheme for ShadcnOverlayWindowTheme {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        overlay_window_look(&self.theme, size, mode)
    }
}
