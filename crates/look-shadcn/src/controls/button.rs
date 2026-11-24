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
