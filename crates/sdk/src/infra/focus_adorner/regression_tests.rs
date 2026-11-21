use gpui::IntoElement;

#[test]
fn custom_checkbox_focus_border_is_stable() {
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
    use xgpui_luma_style::{NoNativeTokens, Source, Theme};
    // An app layer paints a border on the focused control, as the old custom-theme test did.
    let sheet = crate::style::stylesheet()
        .with_source(
            "app",
            Source::new("app/focus.lss", "checkbox { part root { :focused { border = srgb(0, 0, 1) } } }"),
        )
        .expect("compiles");
    let theme = Theme::bind(&sheet, std::sync::Arc::new(NoNativeTokens), "light").expect("binds");
    for role in [
        crate::controls::button_family::ButtonFamilyRole::Text,
        crate::controls::button_family::ButtonFamilyRole::Icon,
    ] {
        let template = crate::controls::checkbox::StyledCheckboxTemplate::new(theme.clone());
        super::testing::assert_stable_layout(move |state, selected, window, cx| {
            template
                .render(
                    &ButtonRenderModel {
                        data: crate::controls::checkbox::CheckboxData::new(selected),
                        state,
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

#[test]
fn custom_radio_focus_border_is_stable() {
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
    use xgpui_luma_style::{NoNativeTokens, Source, Theme};
    let sheet = crate::style::stylesheet()
        .with_source("app", Source::new("app/focus.lss", "radio { part root { :focused { border = srgb(0, 0, 1) } } }"))
        .expect("compiles");
    let theme = Theme::bind(&sheet, std::sync::Arc::new(NoNativeTokens), "light").expect("binds");
    for role in [
        crate::controls::button_family::ButtonFamilyRole::Text,
        crate::controls::button_family::ButtonFamilyRole::Icon,
    ] {
        let template = crate::controls::radio_button::StyledRadioButtonTemplate::new(theme.clone());
        super::testing::assert_stable_layout(move |state, selected, window, cx| {
            template
                .render(
                    &ButtonRenderModel {
                        data: crate::controls::radio_button::RadioButtonData::new(selected),
                        state,
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

#[test]
fn textfield_nonempty_placeholder_is_stable() {
    super::testing::assert_textfield_layout(crate::controls::textfield::default_textfield_theme());
}

#[test]
fn textarea_nonempty_placeholder_is_stable() {
    super::testing::assert_textarea_layout(crate::controls::textarea::default_textarea_theme());
}
