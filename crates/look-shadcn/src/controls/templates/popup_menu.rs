use std::sync::Arc;

use xgpui_luma::controls::popup_menu::{PopupMenuTheme, ThemedPopupMenuTemplate};
use xgpui_luma::theme::InteractionState;

use crate::look::ShadcnLook;

pub fn popup_menu_theme(theme: ShadcnLook) -> Arc<dyn PopupMenuTheme> {
    Arc::new(ShadcnPopupMenuTheme { theme: theme.clone() })
}

struct ShadcnPopupMenuTheme {
    theme: ShadcnLook,
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
            state,
            scale_factor,
            cx,
        )
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn popup_menu_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::popup_menu::PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(Arc::new(ShadcnPopupMenuTheme { theme: theme.clone() })))
}

/// Dedicated template factory with the shared Shadcn button/menu theme recipe.
pub fn split_button_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::split_button::SplitButtonTemplate> {
    Arc::new(xgpui_luma::controls::split_button::ThemedSplitButtonTemplate::new(popup_menu_theme(theme)))
}
