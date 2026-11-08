use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::switch::{SwitchData, SwitchTheme, ThemedSwitchTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::switch::{switch_look, switch_scale_with_stylesheet};
use crate::look::ShadcnLook;

struct ShadcnStyledSwitchTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SwitchTheme for ShadcnStyledSwitchTheme {
    fn resolve(
        &self,
        on: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::switch::SwitchPalette {
        let tokens = self.theme.mode_tokens();
        switch_look(tokens.as_ref(), self.theme.mode(), self.style, on, state, size)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> xgpui_luma::controls::switch::SwitchScale {
        let tokens = self.theme.mode_tokens();
        switch_scale_with_stylesheet(
            tokens.as_ref(),
            &self.theme.stylesheet(),
            self.theme.mode(),
            self.style,
            size,
            None,
            scale_factor,
        )
    }
}

pub fn switch_theme(theme: ShadcnLook) -> Arc<dyn SwitchTheme> {
    switch_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn switch_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn SwitchTheme> {
    Arc::new(ShadcnStyledSwitchTheme { theme: theme.clone(), style })
}

pub fn switch_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_with_style(theme, style)))
}
