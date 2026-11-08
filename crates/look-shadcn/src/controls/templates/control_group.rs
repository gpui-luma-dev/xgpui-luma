use std::sync::Arc;

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::control_group::{
    ControlGroupItemLike, ControlGroupItemPalette, ControlGroupTemplate, ControlGroupTheme,
    control_group_template_with_theme,
};
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale};

use crate::controls::control_group::control_group_list_look;
use crate::look::ShadcnLook;

struct ShadcnControlGroupTheme {
    theme: ShadcnLook,
}

impl ControlGroupTheme for ShadcnControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> xgpui_luma::controls::control_group::ControlGroupListLook {
        let tokens = self.theme.mode_tokens();
        control_group_list_look(tokens.as_ref(), enabled)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        _scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let look = self.theme.resolve_ghost_button(ButtonFamilyRole::Toggle { selected }, size, state);
        let chrome = self.theme.chrome();

        ControlGroupItemPalette {
            foreground: look.foreground,
            background: look.background,
            muted_foreground: chrome.muted_text,
            typography: look.typography,
            font_family: look.font_family.clone(),
        }
    }
}

pub fn control_group_theme(theme: ShadcnLook) -> Arc<dyn ControlGroupTheme> {
    Arc::new(ShadcnControlGroupTheme { theme: theme.clone() })
}

pub fn control_group_template<T>(theme: ShadcnLook) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    control_group_template_with_theme(control_group_theme(theme))
}
