use std::sync::Arc;

use xgpui_luma::controls::context_menu::{ContextMenuTheme, StyledContextMenuTheme, ThemedContextMenuTemplate};
use xgpui_luma::style::ThemeSource;

use crate::look::ShadcnLook;

/// The Shadcn context-menu target drawn from `.lss` (`style/context_menu.lss`); its menu is the md floating menu.
pub fn context_menu_theme(theme: ShadcnLook) -> Arc<dyn ContextMenuTheme> {
    let style = theme.clone();
    Arc::new(StyledContextMenuTheme::new(
        ThemeSource::Live(Arc::new(move || style.style_theme())),
        Arc::new(move || theme.with_size(crate::ShadcnSize::Md).floating_menu_theme().style()),
    ))
}

pub fn context_menu_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::context_menu::ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(theme.clone())))
}
