//! Radix split actions using the SDK's shared popup engine and independent face states.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::{
    popup_menu::{PopupMenuBuilder, PopupMenuTriggerStyle},
    split_button::*,
};
use crate::{Look, PopupMenuVariant, Tone};

/// Live Radix theme shared with Popup Menu, including independent primary/disclosure paint.
/// Notify affected owners after editing the supplied Look.
pub fn split_button_theme(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn SplitButtonTheme> {
    crate::popup_menu_theme(look, variant, tone)
}
/// SDK split-face template with Radix trigger and floating-menu chrome.
pub fn split_button_template(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn SplitButtonTemplate> {
    Arc::new(ThemedSplitButtonTemplate::new(split_button_theme(look, variant, tone)))
}
/// Themed SDK builder retaining primary-only disabling, one focus target, menu
/// keyboard navigation, presenters, placement, size and semantic callback APIs.
/// Solid starts with a Primary trigger; Soft starts with Secondary. Override
/// `.trigger_style(...)` for Outline/Ghost or replace `.template(...)` as needed.
/// Icons and motion defaults are captured; theme colors and geometry resolve live.
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma::infra::menu_item::MenuItem;
/// use xgpui_luma_look_radix::{Look, PopupMenuVariant, Tone, split_button_builder};
/// fn save<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     split_button_builder(look, "save", PopupMenuVariant::Solid, Tone::Accent)
///         .label("Save").primary_enabled(false)
///         .items([MenuItem::new("save-as").label("Save as")]).spawn(cx);
/// }
/// ```
pub fn split_button_builder(
    look: &Look,
    id: impl Into<SharedString>,
    variant: PopupMenuVariant,
    tone: Tone,
) -> SplitButtonBuilder {
    let popup = PopupMenuBuilder::new(id)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().popup_menu)
        .trigger_style(match variant {
            PopupMenuVariant::Solid => PopupMenuTriggerStyle::Primary,
            PopupMenuVariant::Soft => PopupMenuTriggerStyle::Secondary,
        });
    SplitButtonBuilder::from_popup_builder(popup).template(split_button_template(look, variant, tone))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Accent, Gray, SemanticRole};
    use xgpui_luma::controls::popup_menu::PopupMenuTriggerMetrics;
    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
    #[test]
    fn split_faces_and_menu_follow_the_bound_look() {
        let base = Look::built_in();
        let fork = base.fork();
        let theme = split_button_theme(&fork, PopupMenuVariant::Solid, Tone::Accent);
        let before = split_button_theme(&base, PopupMenuVariant::Solid, Tone::Accent).resolve(
            PopupMenuTriggerStyle::Primary,
            Default::default(),
            Default::default(),
        );
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            fork.set_mode(mode);
            for accent in [Accent::Blue, Accent::Lime] {
                fork.set_palettes(accent, Gray::Auto);
                for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                    for bits in 0..16 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let palette = theme.resolve(
                            PopupMenuTriggerStyle::Primary,
                            PopupMenuTriggerMetrics { size, ..Default::default() },
                            state,
                        );
                        if state.disabled {
                            assert_eq!(
                                palette.trigger_foreground,
                                fork.resolve_role(SemanticRole::MutedForeground).hsla()
                            );
                        } else {
                            assert_eq!(palette.trigger_foreground, Tone::Accent.contrast(&fork));
                        }
                        assert_eq!(palette.floating_menu.item_hover_background, Tone::Accent.step(&fork, 9));
                    }
                }
            }
        }
        let unchanged = split_button_theme(&base, PopupMenuVariant::Solid, Tone::Accent).resolve(
            PopupMenuTriggerStyle::Primary,
            Default::default(),
            Default::default(),
        );
        assert_eq!(unchanged.trigger_background, before.trigger_background);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod interaction_tests {
    use super::*;
    use gpui::{Context, Entity, Focusable, Render, TestAppContext, Window, div, prelude::*};
    use xgpui_luma::infra::menu_item::MenuItem;
    use xgpui_luma::focus::LumaFocusScopeExt;
    use std::{cell::RefCell, rc::Rc};
    struct Harness {
        split: Entity<SplitButton>,
        scope: gpui::FocusHandle,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().luma_focus_scope(&self.scope).flex().flex_col().items_start().child(self.split.clone())
        }
    }
    #[test]
    fn radix_split_button_keeps_keyboard_selection_and_primary_only_disabling() {
        let mut app = TestAppContext::single();
        let events = Rc::new(RefCell::new(Vec::new()));
        let capture = events.clone();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            xgpui_luma::focus::bind_default_focus_keys(cx);
            let split = split_button_builder(&Look::built_in(), "save", PopupMenuVariant::Solid, Tone::Accent)
                .label("Save")
                .primary_enabled(false)
                .items([MenuItem::new("disabled").enabled(false), MenuItem::new("save-as").label("Save as")])
                .on_event(cx, move |_: &mut Harness, event, _| match event {
                    SplitButtonEvent::ActionClick => capture.borrow_mut().push("action"),
                    SplitButtonEvent::Select { .. } => capture.borrow_mut().push("select"),
                    SplitButtonEvent::Dismiss => capture.borrow_mut().push("dismiss"),
                    _ => {}
                })
                .spawn(cx);
            split.read(cx).focus_handle(cx).focus(window, cx);
            Harness { split, scope: cx.focus_handle() }
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("enter space down enter");
        assert_eq!(*events.borrow(), ["select"]);
        cx.simulate_keystrokes("down escape");
        assert_eq!(*events.borrow(), ["select", "dismiss"]);
        cx.update(|window, app| {
            let split = view.read(app).split.clone();
            assert!(split.read(app).focus_handle(app).is_focused(window));
            split.update(app, |split, cx| split.set_primary_enabled(true, cx));
        });
        cx.simulate_keystrokes("enter");
        assert_eq!(*events.borrow(), ["select", "dismiss", "action"]);
        cx.update(|_, app| {
            let split = view.read(app).split.clone();
            split.update(app, |split, cx| split.set_enabled(false, cx));
        });
        cx.simulate_keystrokes("enter down space");
        assert_eq!(*events.borrow(), ["select", "dismiss", "action"]);
    }
}
