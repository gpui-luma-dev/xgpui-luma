use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::{CheckboxData, CheckboxTheme, StyledCheckboxTemplate};
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

/// The Shadcn checkbox drawn from `.lss` (`style/checkbox.lss`), following the look's mode and catalog.
pub fn checkbox_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<CheckboxData>> {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledCheckboxTemplate::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    Arc::new(template)
}

/// The `.lss` variant for a style; primary is the base.
pub(crate) fn checkbox_variant(style: ShadcnButtonStyle) -> Option<&'static str> {
    match style {
        ShadcnButtonStyle::Primary => None,
        ShadcnButtonStyle::Secondary => Some("secondary"),
        ShadcnButtonStyle::Outline => Some("outline"),
        ShadcnButtonStyle::Ghost => Some("ghost"),
        ShadcnButtonStyle::ContentOnly => Some("content_only"),
    }
}
