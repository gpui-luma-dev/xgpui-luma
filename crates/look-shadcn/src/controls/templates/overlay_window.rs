use std::sync::Arc;

use xgpui_luma::controls::overlay_window::{
    OverlayWindowTemplate, OverlayWindowTheme, StyledOverlayWindowTheme, ThemedOverlayWindowTemplate,
};

use crate::look::ShadcnLook;

pub fn overlay_window_template(theme: ShadcnLook) -> Arc<dyn OverlayWindowTemplate> {
    Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(theme)))
}

/// The Shadcn overlay window drawn from `.lss` (`style/overlay_window.lss`) at the look's size.
pub fn overlay_window_theme(theme: ShadcnLook) -> Arc<dyn OverlayWindowTheme> {
    let size = theme.size();
    let binding = xgpui_luma::style::StyleBinding::new(
        "overlay_window",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || theme.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(StyledOverlayWindowTheme::new(binding))
}
