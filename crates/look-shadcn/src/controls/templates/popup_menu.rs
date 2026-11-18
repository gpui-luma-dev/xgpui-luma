use std::sync::Arc;

use xgpui_luma::controls::popup_menu::{PopupMenuTheme, ThemedPopupMenuTemplate};
use xgpui_luma::theme::InteractionState;

use crate::look::ShadcnLook;

pub fn popup_menu_theme(theme: ShadcnLook) -> Arc<dyn PopupMenuTheme> {
    let menu_size = theme.size();
    Arc::new(ShadcnPopupMenuTheme { theme, menu_size })
}

/// Trigger size comes from the look handle; the menu may use a different Shadcn size.
struct ShadcnPopupMenuTheme {
    theme: ShadcnLook,
    menu_size: crate::ShadcnSize,
}

impl PopupMenuTheme for ShadcnPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        metrics: xgpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> xgpui_luma::controls::popup_menu::PopupMenuPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::popup_menu::popup_menu_palette(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            metrics,
            self.theme.size(),
            self.menu_size,
            state,
        )
    }

    fn resolve_look(
        &self,
        trigger_style: xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        metrics: xgpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        cx: &mut gpui::App,
    ) -> xgpui_luma::controls::popup_menu::PopupMenuLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::popup_menu::popup_menu_look(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            metrics,
            self.theme.size(),
            self.menu_size,
            state,
            scale_factor,
            cx,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
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
    Arc::new(ThemedPopupMenuTemplate::new(Arc::new(ShadcnPopupMenuTheme { theme, menu_size })))
}

/// Dedicated template factory with the shared Shadcn button/menu theme recipe.
pub fn split_button_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::split_button::SplitButtonTemplate> {
    Arc::new(xgpui_luma::controls::split_button::ThemedSplitButtonTemplate::new(popup_menu_theme(theme)))
}
