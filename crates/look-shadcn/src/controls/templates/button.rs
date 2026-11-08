use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::button_family::{ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme};
use xgpui_luma::controls::toggle::{ToggleData, apply_toggle_progress_chrome};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::button::{ShadcnButtonStyle, button_look_with_stylesheet, button_palette};
use crate::look::ShadcnLook;
use crate::look_context::LookContext;

struct ShadcnStyledButtonFamilyTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl ButtonFamilyTheme for ShadcnStyledButtonFamilyTheme {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        let tokens = self.theme.mode_tokens();
        let stylesheet = self.theme.stylesheet();
        let ctx = LookContext::new(tokens.as_ref(), self.theme.mode(), state);
        button_palette(&ctx, stylesheet.as_ref(), self.style, role, size)
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        _scale: &xgpui_luma::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<xgpui_luma::controls::button_family::ButtonFamilyLook> {
        let tokens = self.theme.mode_tokens();
        Some(button_look_with_stylesheet(
            tokens.as_ref(),
            self.theme.stylesheet().as_ref(),
            self.theme.mode(),
            self.style,
            role,
            size,
            None,
            state,
        ))
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn styled_button_family_theme(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ShadcnStyledButtonFamilyTheme { theme: theme.clone(), style })
}

pub fn button_family_theme(theme: ShadcnLook) -> Arc<dyn ButtonFamilyTheme> {
    styled_button_family_theme(theme, ShadcnButtonStyle::Secondary)
}

pub fn button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(DefaultButtonTemplate::new(styled_button_family_theme(theme, style)))
}

pub fn toggle_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let button_family_theme = styled_button_family_theme(theme, style);
    let theme_for_mod = Arc::clone(&button_family_theme);
    Arc::new(DefaultButtonTemplate::<ToggleData>::new(button_family_theme).with_modifier(move |element, model| {
        if model.look.is_some() {
            return element;
        }

        let off = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: false }, model.size, model.state);
        let on = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: true }, model.size, model.state);
        apply_toggle_progress_chrome(element, &off, &on, model.data.progress)
    }))
}

/// Non-animated toggle chrome for control-group items (`ButtonTemplate<bool>`).
pub fn toggle_item_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(DefaultButtonTemplate::new(styled_button_family_theme(theme, style)))
}

/// Animated toggle chrome for control-group items carrying [`ToggleData`].
pub fn animated_toggle_item_template(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let button_family_theme = styled_button_family_theme(theme, style);
    let theme_for_mod = Arc::clone(&button_family_theme);
    Arc::new(DefaultButtonTemplate::<ToggleData>::new(button_family_theme).with_modifier(move |element, model| {
        let off = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: false }, model.size, model.state);
        let on = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: true }, model.size, model.state);
        apply_toggle_progress_chrome(element, &off, &on, model.data.progress)
    }))
}
