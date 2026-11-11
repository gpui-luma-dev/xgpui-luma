//! Rendered geometry regressions for look variants, not just palette values.
use gpui::IntoElement;
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::checkbox::{CheckboxData, ThemedCheckboxTemplate};
use xgpui_luma::controls::radio_button::{RadioButtonData, ThemedRadioButtonTemplate};
use xgpui_luma::controls::switch::{SwitchData, ThemedSwitchTemplate};
use xgpui_luma::infra::focus_adorner::testing::assert_stable_layout;
use xgpui_luma::theme::{ControlSize, ThemeMode};
use crate::*;

#[test]
fn all_button_variants_keep_control_and_sibling_bounds_across_states() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        for variant in [
            ButtonVariant::Classic,
            ButtonVariant::Solid,
            ButtonVariant::Soft,
            ButtonVariant::Surface,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::GhostQuiet,
            ButtonVariant::Page,
        ] {
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for compact in [false, true] {
                    let look = Look::built_in();
                    look.set_mode(mode);
                    let template = DefaultButtonTemplate::new(button_family_theme(&look, variant));
                    assert_stable_layout(move |state, selected, window, cx| {
                        let model = ButtonRenderModel::<()> {
                            id: "variant".into(),
                            size,
                            compact,
                            role: ButtonFamilyRole::Toggle { selected },
                            state,
                            ..Default::default()
                        };
                        template.render(&model, window, cx).into_any_element()
                    });
                }
            }
        }
    }
}

#[test]
fn choice_indicators_reserve_focus_space_in_every_state() {
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        for variant in [CheckboxVariant::Classic, CheckboxVariant::Surface, CheckboxVariant::Soft] {
            let look = Look::built_in();
            let template = ThemedCheckboxTemplate::new(checkbox_theme(&look, variant));
            assert_stable_layout(move |state, selected, window, cx| {
                template
                    .render(
                        &ButtonRenderModel { data: CheckboxData::new(selected), state, size, ..Default::default() },
                        window,
                        cx,
                    )
                    .into_any_element()
            });
        }
        for variant in [RadioVariant::Classic, RadioVariant::Surface, RadioVariant::Soft] {
            let look = Look::built_in();
            let template = ThemedRadioButtonTemplate::new(radio_theme(&look, variant));
            assert_stable_layout(move |state, selected, window, cx| {
                template
                    .render(
                        &ButtonRenderModel { data: RadioButtonData::new(selected), state, size, ..Default::default() },
                        window,
                        cx,
                    )
                    .into_any_element()
            });
        }
        for variant in [SwitchVariant::Classic, SwitchVariant::Surface, SwitchVariant::Soft] {
            let look = Look::built_in();
            let template = ThemedSwitchTemplate::new(switch_theme_with(&look, variant, Paint::accent()));
            assert_stable_layout(move |state, selected, window, cx| {
                template
                    .render(
                        &ButtonRenderModel { data: SwitchData::new(selected), state, size, ..Default::default() },
                        window,
                        cx,
                    )
                    .into_any_element()
            });
        }
    }
}

#[test]
fn text_and_popup_triggers_keep_their_focus_space() {
    use xgpui_luma::infra::focus_adorner::testing::*;
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let look = Look::built_in();
        look.set_mode(mode);
        for variant in [TextFieldVariant::Classic, TextFieldVariant::Surface, TextFieldVariant::Soft] {
            assert_textfield_layout(textfield_theme_with(&look, variant));
            assert_textarea_layout(textarea_theme_with(&look, variant));
        }
        assert_selector_layout(selector_theme(&look));
        assert_popup_layout(popup_menu_theme(&look, PopupMenuVariant::Solid, Tone::Accent));
    }
}
