use serde::Deserialize;
use std::collections::HashMap;

use crate::size::ShadcnSize;

use crate::provenance::ResolvedColor;

/// Button-local corner radius presets (shadcn/Shadcn `radius` prop).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonRadiusPreset {
    None,
    Small,
    Medium,
    Large,
    Full,
}

impl ButtonRadiusPreset {
    pub const ALL: [Self; 5] = [Self::None, Self::Small, Self::Medium, Self::Large, Self::Full];

    /// The `.lss` `radius` attribute value.
    pub fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Full => "full",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No radius",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Full => "Full",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
    ContentOnly,
}

#[derive(Clone, Debug)]
pub struct ButtonColorPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

impl ButtonColorPalette {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: None,
        }
    }
}

// Mirrors the public semantic resolver with an additional selected stylesheet.

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;

    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn text_and_popup_focus_geometry_is_stable() {
        use xgpui_luma::infra::focus_adorner::testing::*;
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
            look.set_mode(mode);
            for theme in [
                look.textfield_theme(),
                look.input_textfield_theme(),
                look.surface_textfield_theme(),
                look.primary_textfield_theme(),
            ] {
                assert_textfield_layout(theme);
            }
            for theme in [
                look.textarea_theme(),
                look.input_textarea_theme(),
                look.surface_textarea_theme(),
                look.primary_textarea_theme(),
            ] {
                assert_textarea_layout(theme);
            }
            assert_selector_layout(look.selector_theme());
            assert_search_selector_layout(look.selector_theme());
            assert_popup_layout(look.popup_menu_theme());
        }
    }

    #[test]
    fn choice_focus_geometry_is_stable_across_styles() {
        use xgpui_luma::controls::button::ButtonRenderModel;
        use xgpui_luma::infra::focus_adorner::testing::assert_stable_layout;
        use gpui::IntoElement;
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
            look.set_mode(mode);
            for style in [
                ShadcnButtonStyle::Primary,
                ShadcnButtonStyle::Secondary,
                ShadcnButtonStyle::Outline,
                ShadcnButtonStyle::Ghost,
                ShadcnButtonStyle::ContentOnly,
            ] {
                let template = super::super::templates::checkbox_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::checkbox::CheckboxData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
                let template = super::super::templates::radio_button_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::radio_button::RadioButtonData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
                let template = super::super::templates::switch_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::switch::SwitchData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
            }
        }
    }
}

/// Button metrics. Colors and shadows come from `button.lss`, so a sheet that sets rules for them is
/// an error rather than silently ignored.
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
}

impl ButtonStylesheet {
    /// Native bundled rules in their original priority order.
    /// Native bundled metric-token references.
    pub fn native_default() -> Self {
        Self {
            tokens: [("sm", "metrics.control.sm"), ("md", "metrics.control.md"), ("lg", "metrics.control.lg")]
                .into_iter()
                .map(|(key, height)| {
                    (
                        key.into(),
                        ButtonMetricTokensRule { height: Some(height.into()), corner_radius: Some("radius".into()) },
                    )
                })
                .collect(),
            ..Default::default()
        }
    }

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ShadcnSize::Sm => "sm",
            ShadcnSize::Md => "md",
            ShadcnSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
}

/// Shadcn token references retained independently of common pixel dimensions.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonMetricTokensRule {
    pub height: Option<String>,
    pub corner_radius: Option<String>,
}

/// Native common geometry owned by this control.
pub(crate) fn native_button_stylesheet() -> xgpui_luma::theme::stylesheet::ButtonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ButtonStylesheet {
        sizes: [("sm", 12.0, 12.0, 14.0), ("md", 16.0, 14.0, 16.0), ("lg", 20.0, 16.0, 18.0)]
            .into_iter()
            .map(|(key, padding, font, icon)| {
                (
                    key.into(),
                    ButtonGeometryStylesheet {
                        padding_x: Some(padding),
                        font_size: Some(font),
                        icon_size: Some(icon),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}
