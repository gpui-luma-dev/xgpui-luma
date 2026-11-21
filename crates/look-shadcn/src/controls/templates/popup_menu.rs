use std::sync::Arc;

use xgpui_luma::controls::popup_menu::{PopupMenuTheme, StyledPopupMenuTheme, ThemedPopupMenuTemplate};
use xgpui_luma::style::ThemeSource;

use crate::look::ShadcnLook;

pub fn popup_menu_theme(theme: ShadcnLook) -> Arc<dyn PopupMenuTheme> {
    let menu_size = theme.size();
    styled_popup_menu_theme(theme, menu_size)
}

/// The Shadcn popup trigger drawn from `.lss` (`popup_menu extends button`, so this look's button styles)
/// at the look's size, with its floating menu at `menu_size`.
fn styled_popup_menu_theme(theme: ShadcnLook, menu_size: crate::ShadcnSize) -> Arc<dyn PopupMenuTheme> {
    let size = theme.size();
    let style = theme.clone();
    Arc::new(
        StyledPopupMenuTheme::new(
            ThemeSource::Live(Arc::new(move || style.style_theme())),
            Arc::new(move || {
                crate::controls::floating_menu::floating_menu_look(
                    theme.mode_tokens().as_ref(),
                    theme.mode(),
                    menu_size,
                )
            }),
        )
        .with(|binding| {
            if size != crate::ShadcnSize::Md {
                binding.attr("size", size.key())
            } else {
                binding
            }
        }),
    )
}

pub fn popup_menu_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::popup_menu::PopupMenuTemplate> {
    let menu_size = theme.size();
    popup_menu_template_with_menu_size(theme, menu_size)
}

/// Popup template whose trigger uses the look handle's size and whose menu uses `menu_size`.
pub fn popup_menu_template_with_menu_size(
    theme: ShadcnLook,
    menu_size: crate::ShadcnSize,
) -> Arc<dyn xgpui_luma::controls::popup_menu::PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(styled_popup_menu_theme(theme, menu_size)))
}

/// Dedicated template factory with the shared Shadcn button/menu theme recipe.
pub fn split_button_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::split_button::SplitButtonTemplate> {
    Arc::new(xgpui_luma::controls::split_button::ThemedSplitButtonTemplate::new(popup_menu_theme(theme)))
}
