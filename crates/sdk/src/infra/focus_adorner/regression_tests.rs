use gpui::IntoElement;

struct FocusBorderCheckboxTheme;
impl crate::controls::checkbox::CheckboxTheme for FocusBorderCheckboxTheme {
    fn resolve(
        &self,
        checked: bool,
        state: crate::theme::InteractionState,
        size: crate::theme::ControlSize,
    ) -> crate::controls::checkbox::CheckboxPalette {
        let mut palette = crate::controls::checkbox::default_checkbox_theme().resolve(checked, state, size);
        palette.control_border = state.focused.then_some(gpui::blue());
        palette
    }
    fn metrics(&self) -> crate::theme::MetricTokens {
        crate::theme::MetricTokens::default()
    }
}
#[test]
fn custom_checkbox_focus_border_is_stable() {
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
    for size in [crate::theme::ControlSize::Sm, crate::theme::ControlSize::Md, crate::theme::ControlSize::Lg] {
        for role in [
            crate::controls::button_family::ButtonFamilyRole::Text,
            crate::controls::button_family::ButtonFamilyRole::Icon,
        ] {
            let template =
                crate::controls::checkbox::ThemedCheckboxTemplate::new(std::sync::Arc::new(FocusBorderCheckboxTheme));
            super::testing::assert_stable_layout(move |state, selected, window, cx| {
                template
                    .render(
                        &ButtonRenderModel {
                            data: crate::controls::checkbox::CheckboxData::new(selected),
                            state,
                            size,
                            role,
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

struct FocusBorderRadioTheme;
impl crate::controls::radio_button::RadioButtonTheme for FocusBorderRadioTheme {
    fn resolve(
        &self,
        checked: bool,
        state: crate::theme::InteractionState,
        size: crate::theme::ControlSize,
    ) -> crate::controls::radio_button::RadioButtonPalette {
        let mut palette = crate::controls::radio_button::default_radio_button_theme().resolve(checked, state, size);
        palette.control_border = state.focused.then_some(gpui::blue());
        palette
    }
    fn metrics(&self) -> crate::theme::MetricTokens {
        crate::theme::MetricTokens::default()
    }
}
#[test]
fn custom_radio_focus_border_is_stable() {
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
    for size in [crate::theme::ControlSize::Sm, crate::theme::ControlSize::Md, crate::theme::ControlSize::Lg] {
        for role in [
            crate::controls::button_family::ButtonFamilyRole::Text,
            crate::controls::button_family::ButtonFamilyRole::Icon,
        ] {
            let template = crate::controls::radio_button::ThemedRadioButtonTemplate::new(std::sync::Arc::new(
                FocusBorderRadioTheme,
            ));
            super::testing::assert_stable_layout(move |state, selected, window, cx| {
                template
                    .render(
                        &ButtonRenderModel {
                            data: crate::controls::radio_button::RadioButtonData::new(selected),
                            state,
                            size,
                            role,
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

#[test]
fn textfield_nonempty_placeholder_is_stable() {
    super::testing::assert_textfield_layout(crate::controls::textfield::default_textfield_theme());
}

#[test]
fn textarea_nonempty_placeholder_is_stable() {
    super::testing::assert_textarea_layout(crate::controls::textarea::default_textarea_theme());
}
