use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::textfield::TextFieldState;
use xgpui_luma::theme::{InteractionState, ThemeMode};

use crate::{ShadcnButtonStyle, ShadcnLook, ShadcnTextFieldStyle};
use crate::BadgeVariant;
use crate::size::ShadcnSize;

use crate::inspect::controls::{
    BadgeInspectMetrics, BadgeInspectPalette, ButtonInspectElevation, ButtonInspectMetrics, ButtonInspectPalette,
    ButtonInspectTypography, FloatingMenuInspectMetrics, FloatingMenuInspectPalette, ResizablePanelsInspectMetrics,
    SelectorInspectMetrics, TextFieldInspectMetrics, TextFieldInspectPalette, inspect_badge_color_palette,
    inspect_badge_metrics, inspect_floating_menu_color_palette, inspect_floating_menu_metrics,
    inspect_resizable_panels_metrics, inspect_selector_metrics, inspect_textfield_color_palette,
    inspect_textfield_metrics,
};

/// Inspect-time wrapper around a [`ShadcnLook`] look resolver.
pub struct ShadcnInspect<'a> {
    look: &'a ShadcnLook,
}

impl<'a> ShadcnInspect<'a> {
    pub fn new(look: &'a ShadcnLook) -> Self {
        Self { look }
    }

    pub fn look(&self) -> &ShadcnLook {
        self.look
    }

    fn mode_tokens(&self) -> std::sync::Arc<crate::ShadcnModeTokens> {
        self.look.mode_tokens()
    }

    fn theme_mode(&self) -> ThemeMode {
        self.look.mode()
    }

    pub fn inspect_button_color_palette(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> ButtonInspectPalette {
        crate::inspect::controls::inspect_button_color_palette_with_stylesheet(
            &self.mode_tokens(),
            self.look.stylesheet().as_ref(),
            self.theme_mode(),
            style,
            role,
            state,
        )
    }

    pub fn inspect_button_metrics(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonInspectMetrics {
        crate::inspect::controls::inspect_button_metrics_with_stylesheet(
            &self.mode_tokens(),
            self.look.stylesheet().as_ref(),
            self.theme_mode(),
            style,
            role,
            size,
            state,
        )
    }

    pub fn inspect_button_typography_for_size(
        &self,
        size: ShadcnSize,
        role: ButtonFamilyRole,
    ) -> ButtonInspectTypography {
        crate::tables::typography::resolve_control_typography_with_stylesheet(
            &self.mode_tokens(),
            self.look.stylesheet().as_ref(),
            size,
            matches!(role, ButtonFamilyRole::Toggle { .. }),
        )
        .into()
    }

    pub fn inspect_button_elevation(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> ButtonInspectElevation {
        crate::inspect::controls::inspect_button_elevation_with_stylesheet(
            &self.mode_tokens(),
            self.look.stylesheet().as_ref(),
            self.theme_mode(),
            style,
            role,
            state,
        )
    }

    pub fn inspect_badge_color_palette(&self, variant: BadgeVariant) -> BadgeInspectPalette {
        inspect_badge_color_palette(self.look, variant)
    }

    pub fn inspect_badge_metrics(&self, variant: BadgeVariant, size: ShadcnSize) -> BadgeInspectMetrics {
        inspect_badge_metrics(&self.mode_tokens(), self.look, variant, size, self.theme_mode())
    }

    pub fn inspect_textfield_color_palette(
        &self,
        style: ShadcnTextFieldStyle,
        state: TextFieldState,
        enabled: bool,
    ) -> TextFieldInspectPalette {
        inspect_textfield_color_palette(&self.mode_tokens(), self.theme_mode(), style, state, enabled)
    }

    pub fn inspect_textfield_metrics(&self, size: ShadcnSize) -> TextFieldInspectMetrics {
        inspect_textfield_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_floating_menu_color_palette(&self, size: ShadcnSize) -> FloatingMenuInspectPalette {
        inspect_floating_menu_color_palette(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_floating_menu_metrics(&self, size: ShadcnSize) -> FloatingMenuInspectMetrics {
        inspect_floating_menu_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_resizable_panels_metrics(&self, handle_size: crate::ShadcnSize) -> ResizablePanelsInspectMetrics {
        inspect_resizable_panels_metrics(self.look.mode_tokens().as_ref(), handle_size)
    }

    pub fn inspect_selector_metrics(&self, size: ShadcnSize) -> SelectorInspectMetrics {
        inspect_selector_metrics(
            &self.mode_tokens(),
            self.theme_mode(),
            xgpui_luma::controls::selector::SelectorTriggerStyle::Outline,
            size,
        )
    }
}
