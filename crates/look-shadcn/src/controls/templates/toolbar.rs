use std::sync::Arc;

use xgpui_luma::controls::toolbar::{StyledToolbarTheme, ThemedToolbarTemplate, ToolbarTemplate, ToolbarTheme};

use crate::look::ShadcnLook;

/// The Shadcn toolbar drawn from `.lss` (`style/toolbar.lss`) at the look's size.
pub fn toolbar_theme(theme: ShadcnLook) -> Arc<dyn ToolbarTheme> {
    let size = theme.size();
    let binding = xgpui_luma::style::StyleBinding::new(
        "toolbar",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || theme.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(StyledToolbarTheme::new(binding))
}

pub fn toolbar_template(theme: ShadcnLook) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(toolbar_theme(theme)))
}
