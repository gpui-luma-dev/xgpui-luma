use std::sync::Arc;

use xgpui_luma::controls::slider::{
    SliderTemplate, SliderTheme, ThemedAngularDialTemplate, ThemedCircularRingTemplate, ThemedSliderTemplate,
};
use xgpui_luma::theme::InteractionState;

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::slider::slider_look_with_stylesheet;
use crate::look::ShadcnLook;

pub fn slider_theme(theme: ShadcnLook) -> Arc<dyn SliderTheme> {
    slider_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn slider_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn SliderTheme> {
    Arc::new(ShadcnSliderTheme { theme: theme.clone(), style })
}

struct ShadcnSliderTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SliderTheme for ShadcnSliderTheme {
    fn resolve(
        &self,
        size: xgpui_luma::theme::ControlSize,
        thumb_size: Option<xgpui_luma::controls::slider::SliderThumbSize>,
        state: InteractionState,
    ) -> xgpui_luma::controls::slider::SliderLook {
        let tokens = self.theme.mode_tokens();
        slider_look_with_stylesheet(
            tokens.as_ref(),
            &self.theme.stylesheet(),
            self.theme.mode(),
            self.style,
            size,
            thumb_size,
            state,
        )
    }
}

pub fn slider_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
    slider_template_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn slider_template_with_style(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_with_style(theme, style)))
}

pub fn slider_angular_template(theme: ShadcnLook) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedAngularDialTemplate::new(slider_theme(theme)))
}

pub fn slider_circular_ring_template(theme: ShadcnLook) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedCircularRingTemplate::new(slider_theme(theme)))
}
