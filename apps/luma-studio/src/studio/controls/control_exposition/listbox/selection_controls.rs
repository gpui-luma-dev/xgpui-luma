//! One runtime selection policy shared by every ListBox exposition example.

use xgpui_luma::controls::selector::SelectorEventExt;
use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::checkbox::{Checkbox};
use xgpui_luma::controls::listbox::{SelectionMode, SelectionPolicy};
use xgpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::vstack;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, LumaTypographyExt, ShadcnTextSize};
use xgpui_luma_look_shadcn::prelude::*;

pub(super) const DEFAULT_POLICY: SelectionPolicy =
    SelectionPolicy { mode: SelectionMode::Extended, toggle_off: true, selection_follows_active: false };

type ApplyPolicy = Box<dyn Fn(SelectionPolicy, &mut Context<SelectionControls>)>;

const MODES: [(&str, &str, SelectionMode); 5] = [
    ("none", "No selection", SelectionMode::None),
    ("required", "Single · required", SelectionMode::SingleRequired),
    ("single", "Single · optional", SelectionMode::SingleAllowNone),
    ("multiple", "Multiple · toggle", SelectionMode::Multiple),
    ("extended", "Extended · range", SelectionMode::Extended),
];

pub(super) struct SelectionControls {
    look: Arc<ShadcnLook>,
    policy: SelectionPolicy,
    mode: Entity<Selector>,
    toggle_off: Checkbox,
    follows_active: Checkbox,
    apply: ApplyPolicy,
    _subscriptions: Vec<Subscription>,
}

impl SelectionControls {
    pub fn new(
        look: Arc<ShadcnLook>,
        apply: impl Fn(SelectionPolicy, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let mode = shadcn::Selector::new("listbox-selection-mode")
            .look(&look)
            .label("Selection mode")
            .items(MODES.map(|(id, label, _)| SelectorItem::new(id).label(label)))
            .selected_id("extended")
            .spawn(cx);
        let toggle_off = shadcn::Checkbox::new("listbox-toggle-off")
            .look(&look)
            .checked(DEFAULT_POLICY.toggle_off)
            .content(|_, _| div().child("Toggle off selected item").into_any_element())
            .spawn(cx);
        let follows_active = shadcn::Checkbox::new("listbox-follows-active")
            .look(&look)
            .enabled(false)
            .content(|_, _| div().child("Selection follows navigation").into_any_element())
            .spawn(cx);
        let subscriptions = {
            let retained_subscriptions = Vec::new();
            mode.clone().on_event(cx, |this, event, cx| {
                if let SelectorEvent::Change { item_id, .. } = event
                    && let Some((_, _, mode)) = MODES.iter().find(|(id, _, _)| *id == item_id.as_ref())
                {
                    this.policy.mode = *mode;
                    this.apply_policy(cx);
                }
            });
            toggle_off.clone().on_value_changed(cx, |this, checked, cx| {
                this.policy.toggle_off = checked;
                this.apply_policy(cx);
            });
            follows_active.clone().on_value_changed(cx, |this, checked, cx| {
                this.policy.selection_follows_active = checked;
                this.apply_policy(cx);
            });
            retained_subscriptions
        };
        Self {
            look,
            policy: DEFAULT_POLICY,
            mode,
            toggle_off,
            follows_active,
            apply: Box::new(apply),
            _subscriptions: subscriptions,
        }
    }

    fn apply_policy(&mut self, cx: &mut Context<Self>) {
        (self.apply)(self.policy, cx);
        self.toggle_off.update(cx, |control, cx| {
            control
                .set_enabled(matches!(self.policy.mode, SelectionMode::SingleAllowNone | SelectionMode::Extended), cx)
        });
        self.follows_active.update(cx, |control, cx| {
            control.set_enabled(
                matches!(self.policy.mode, SelectionMode::SingleAllowNone | SelectionMode::SingleRequired),
                cx,
            )
        });
        cx.notify();
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.mode.update(cx, |_, cx| cx.notify());
        self.toggle_off.update(cx, |_, cx| cx.notify());
        self.follows_active.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for SelectionControls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let hint = match self.policy.mode {
                SelectionMode::None => {
                    "Click or arrows move the active item. Enter or double-click activates; nothing is selected."
                }
                SelectionMode::SingleRequired => "One enabled item must remain selected in each list.",
                SelectionMode::SingleAllowNone => {
                    "Select one item or clear it. Toggle-off and navigation selection are optional."
                }
                SelectionMode::Multiple => {
                    "Click or Space toggles items independently. Cmd/Ctrl+A selects all; Cmd/Ctrl+Shift+A clears."
                }
                SelectionMode::Extended if self.policy.toggle_off => {
                    "Click selects; click a checked item to uncheck it. Cmd/Ctrl-click toggles; Shift-click or Shift+arrows selects a range. Enter selects and activates. Cmd/Ctrl+Shift+A clears."
                }
                SelectionMode::Extended => {
                    "Click selects one; Cmd/Ctrl-click toggles; Shift-click or Shift+arrows selects a range. Cmd/Ctrl+Shift adds a range. Cmd/Ctrl+A selects all; Cmd/Ctrl+Shift+A clears."
                }
            };
            vstack! { gap=8.0;
                div().child("Selection policies · All examples")
                    .typography_style(self.look.typography_scale(ShadcnTextSize::Sm)).text_color(self.look.chrome().title_text),
                div().w(px(250.0)).child(self.mode.clone()),
                div().flex().flex_wrap().gap(px(12.0)).child(self.toggle_off.clone()).child(self.follows_active.clone()),
                div().typography_style(self.look.typography_scale(ShadcnTextSize::Xs)).text_color(self.look.chrome().muted_text).child(hint),
            }
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use xgpui_luma::controls::checkbox::CheckboxEvent;
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use gpui::TestAppContext;

    #[test]
    fn selection_controls_publish_the_full_policy_on_each_setting_change() {
        let mut app = TestAppContext::single();
        let policies = Rc::new(RefCell::new(Vec::new()));
        let target = policies.clone();
        let (view, cx) = app.add_window_view(|_, cx| {
            SelectionControls::new(
                Arc::new(ShadcnLook::built_in()),
                move |policy, _| target.borrow_mut().push(policy),
                cx,
            )
        });
        cx.update(|_, app| {
            assert_eq!(view.read(app).policy, DEFAULT_POLICY);
            view.read(app)
                .toggle_off
                .clone()
                .update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: false }));
        });
        cx.run_until_parked();
        assert_eq!(policies.borrow().last(), Some(&SelectionPolicy { toggle_off: false, ..DEFAULT_POLICY }));
        for (id, label, mode) in MODES {
            cx.update(|_, app| {
                view.read(app)
                    .mode
                    .clone()
                    .update(app, |_, cx| cx.emit(SelectorEvent::Change { item_id: id.into(), label: label.into() }));
            });
            cx.run_until_parked();
            assert_eq!(policies.borrow().last().unwrap().mode, mode);
        }
        cx.update(|_, app| {
            view.read(app)
                .toggle_off
                .clone()
                .update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }));
        });
        cx.run_until_parked();
        assert_eq!(policies.borrow().last(), Some(&DEFAULT_POLICY));
    }
}
