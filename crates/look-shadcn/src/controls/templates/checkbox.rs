use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::{CheckboxData, CheckboxTheme, ThemedCheckboxTemplate};
use xgpui_luma::theme::{InteractionState};

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::checkbox::{checkbox_look_with_stylesheet, checkbox_scale_with_stylesheet};
use crate::look::ShadcnLook;

struct ShadcnStyledCheckboxTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl CheckboxTheme for ShadcnStyledCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> xgpui_luma::controls::checkbox::CheckboxPalette {
        let size = self.theme.size();
        let tokens = self.theme.mode_tokens();
        checkbox_look_with_stylesheet(
            &crate::LookContext::new(tokens.as_ref(), self.theme.mode(), state),
            self.theme.stylesheet().as_ref(),
            self.style,
            checked,
            size,
        )
    }

    fn scale(&self, scale_factor: f32) -> xgpui_luma::controls::checkbox::CheckboxScale {
        let size = self.theme.size();
        let tokens = self.theme.mode_tokens();
        checkbox_scale_with_stylesheet(
            &crate::LookContext::new(tokens.as_ref(), self.theme.mode(), InteractionState::default()),
            self.theme.stylesheet().as_ref(),
            size,
            scale_factor,
        )
        .0
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn checkbox_theme(theme: ShadcnLook) -> Arc<dyn CheckboxTheme> {
    checkbox_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn checkbox_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn CheckboxTheme> {
    Arc::new(ShadcnStyledCheckboxTheme { theme: theme.clone(), style })
}

pub fn checkbox_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<CheckboxData>> {
    Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_with_style(theme, style)))
}
