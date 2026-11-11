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

fn composite_item(
    look: &Look,
    icon: bool,
    state: xgpui_luma::theme::InteractionState,
    selected: bool,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui::Stateful<gpui::Div> {
    use xgpui_luma::controls::control_group::*;
    let id = "group".into();
    let item = ControlGroupItem::new("item").label("Item");
    let stylesheet = look.common_stylesheet();
    let model = ControlGroupItemRenderModel {
        group_id: &id,
        stylesheet: &stylesheet,
        layout: ControlGroupLayout::Vertical,
        item: &item,
        index: 0,
        sibling_count: 1,
        selected,
        selection_progress: if selected { 1. } else { 0. },
        active: state.focused,
        enabled: !state.disabled,
        state: ControlGroupItemState {
            active: state.focused,
            focus_visible: state.focused,
            disabled: state.disabled,
            hovered: state.hovered,
            pressed: state.pressed,
            selected,
        },
        selection_mode: ControlSelectionMode::Multiple,
        foreground: None,
    };
    let template = if icon {
        crate::icon_group_item_template(look, ControlSize::Md)
    } else {
        crate::control_group_item_template(look, ControlSize::Md)
    };
    template(&model, None, window, cx)
}

#[test]
fn composites_respect_transparent_and_custom_focus_recipes_without_replacing_face_borders() {
    use gpui::{Context, Render, Window};
    use xgpui_luma::theme::InteractionState;
    struct Composite {
        look: Look,
        icon: bool,
        state: InteractionState,
    }
    impl Render for Composite {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            composite_item(&self.look, self.icon, self.state, false, window, cx)
        }
    }
    for icon in [false, true] {
        for recipe in ["none", "tone.6"] {
            let look = Look::built_in().fork();
            look.set_button_recipes(&format!("[[button_recipes.rules]]\nbackground='transparent'\nforeground='role.foreground'\nborder='gray.7'\nfocus_border='{recipe}'")).unwrap();
            let palette = crate::button_family_theme(&look, ButtonVariant::Ghost).resolve(
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState::default(),
            );
            let expected_focus = palette.focus_border;
            let expected_face = palette.border.unwrap();
            let focus_role = look.resolve_role(SemanticRole::Focus).hsla();
            let mut app = gpui::TestAppContext::single();
            let paint_look = look.clone();
            let (view, cx) =
                app.add_window_view(|_, _| Composite { look: paint_look, icon, state: InteractionState::default() });
            for (focused, disabled) in [(false, false), (true, false), (true, true), (false, false)] {
                view.update(cx, |view, cx| {
                    view.state.focused = focused;
                    view.state.disabled = disabled;
                    cx.notify();
                });
                cx.run_until_parked();
                cx.update(|window, _| {
                    let quads = window.painted_quads();
                    assert!(quads.iter().any(|q| q.border_color == expected_face), "face border survives focus");
                    assert!(
                        !quads.iter().any(|q| q.border_color == focus_role),
                        "hard-coded focus role must not bypass recipe"
                    );
                    if expected_focus.a > 0. {
                        assert_eq!(quads.iter().any(|q| q.border_color == expected_focus), focused && !disabled);
                    }
                });
            }
            assert_stable_layout(move |state, selected, window, cx| {
                composite_item(&look, icon, state, selected, window, cx).into_any_element()
            });
        }
    }
}
