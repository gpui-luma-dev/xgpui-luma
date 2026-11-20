use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonData, ButtonTemplate, StyledButtonTemplate};
use xgpui_luma::controls::button_family::{ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme};
use xgpui_luma::controls::toggle::ToggleData;
use xgpui_luma::theme::{InteractionState};

use crate::controls::button::{ShadcnButtonStyle, button_look_with_stylesheet, button_palette};
use crate::look::ShadcnLook;
use crate::look_context::LookContext;

struct ShadcnStyledButtonFamilyTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl ButtonFamilyTheme for ShadcnStyledButtonFamilyTheme {
    fn resolve(&self, role: ButtonFamilyRole, state: InteractionState) -> ButtonFamilyPalette {
        let size = self.theme.size();
        let tokens = self.theme.mode_tokens();
        let stylesheet = self.theme.stylesheet();
        let ctx = LookContext::new(tokens.as_ref(), self.theme.mode(), state);
        button_palette(&ctx, stylesheet.as_ref(), self.style, role, size)
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        state: InteractionState,
        _scale: &xgpui_luma::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<xgpui_luma::controls::button_family::ButtonFamilyLook> {
        let size = self.theme.size();
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
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn styled_button_family_theme(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ShadcnStyledButtonFamilyTheme { theme: theme.clone(), style })
}

pub fn button_family_theme(theme: ShadcnLook) -> Arc<dyn ButtonFamilyTheme> {
    styled_button_family_theme(theme, ShadcnButtonStyle::Secondary)
}

/// The Shadcn button drawn from `.lss` (`style/button.lss`), following the look's mode and catalog.
pub fn button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(styled_button_template::<()>(theme, style))
}

/// The Shadcn toggle drawn from `.lss` (`style/toggle.lss`): its colors blend with the toggle's progress.
pub fn toggle_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<ToggleData>> {
    Arc::new(styled_button_template::<ToggleData>(theme, style))
}

pub(crate) fn styled_button_template<D: ButtonData>(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> StyledButtonTemplate<D> {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledButtonTemplate::<D>::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = super::checkbox::checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    template
}

/// Toggle chrome for control-group items (`ButtonTemplate<bool>`), drawn from `.lss`; the item's role
/// carries its selection.
pub fn toggle_item_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(styled_button_template::<bool>(theme, style))
}

/// Animated toggle chrome for control-group items carrying [`ToggleData`], drawn from `.lss`.
pub fn animated_toggle_item_template(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    Arc::new(styled_button_template::<ToggleData>(theme, style))
}
