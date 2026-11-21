use std::sync::Arc;

use xgpui_luma::controls::slider::{
    SliderTemplate, SliderTheme, StyledSliderTheme, ThemedAngularDialTemplate, ThemedCircularRingTemplate,
    ThemedSliderTemplate,
};

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

pub fn slider_theme(theme: ShadcnLook) -> Arc<dyn SliderTheme> {
    slider_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

/// The Shadcn slider drawn from `.lss` (`style/slider.lss`) at the look's size; secondary uses the
/// secondary foreground, every other style the primary.
pub fn slider_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn SliderTheme> {
    let size = theme.size();
    let binding = xgpui_luma::style::StyleBinding::new(
        "slider",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || theme.style_theme())),
    );
    let binding = if style == ShadcnButtonStyle::Secondary {
        binding.variant("secondary")
    } else {
        binding
    };
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(StyledSliderTheme::new(binding))
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
