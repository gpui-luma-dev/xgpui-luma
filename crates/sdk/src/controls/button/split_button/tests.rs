use super::*;
use gpui::{
    AppContext, Context, Entity, FocusHandle, Focusable, Render, TestAppContext, VisualTestContext, Window, div,
    prelude::*,
};
use crate::infra::menu_item::MenuItem;
use crate::focus::LumaFocusScopeExt;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

struct Page {
    split: Entity<SplitButton>,
    next: FocusHandle,
    scope: FocusHandle,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .luma_focus_scope(&self.scope)
            .flex()
            .flex_col()
            .items_start()
            .child(self.split.clone())
            .child(div().id("next").track_focus(&self.next).child("Next"))
    }
}
struct Probe(Arc<Mutex<SplitButtonState>>);
impl SplitButtonTemplate for Probe {
    fn render(
        &self,
        model: &SplitButtonRenderModel<'_>,
        handlers: SplitButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> gpui::Stateful<gpui::Div> {
        *self.0.lock().unwrap() = model.split_state;
        default_split_button_template().render(model, handlers, window, cx)
    }
}
type Log = Rc<RefCell<Vec<String>>>;
fn setup(
    app: &mut TestAppContext,
    primary_enabled: bool,
    probe: Arc<Mutex<SplitButtonState>>,
) -> (Entity<Page>, Log, &mut VisualTestContext) {
    let log: Log = Default::default();
    let capture = log.clone();
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        crate::focus::bind_default_focus_keys(cx);
        let split = SplitButton::new("test-split")
            .label("Save")
            .primary_enabled(primary_enabled)
            .items([
                MenuItem::new("disabled").enabled(false),
                MenuItem::new("save-as").label("Save as"),
                MenuItem::new("export"),
            ])
            .template(Arc::new(Probe(probe)))
            .on_event(cx, move |_: &mut Page, event, _| {
                capture.borrow_mut().push(match event {
                    SplitButtonEvent::ActionClick => "action".into(),
                    SplitButtonEvent::Select { item_id, .. } => format!("select:{item_id}"),
                    SplitButtonEvent::OpenChanged { open } => format!("open:{open}"),
                    SplitButtonEvent::Dismiss => "dismiss".into(),
                    _ => return,
                })
            })
            .spawn(cx);
        split.read(cx).focus_handle(cx).focus(window, cx);
        Page { split, next: cx.focus_handle().tab_stop(true), scope: cx.focus_handle() }
    });
    cx.run_until_parked();
    (page, log, cx)
}

#[test]
fn keyboard_distinguishes_primary_selection_and_dismissal_and_restores_focus() {
    let mut app = TestAppContext::single();
    let (page, log, cx) = setup(&mut app, true, Default::default());
    cx.simulate_keystrokes("enter space down enter");
    assert_eq!(*log.borrow(), ["action", "action", "open:true", "open:false", "select:save-as"]);
    cx.update(|window, app| assert!(page.read(app).split.read(app).focus_handle(app).is_focused(window)));
    log.borrow_mut().clear();
    cx.simulate_keystrokes("up enter");
    assert_eq!(*log.borrow(), ["open:true", "open:false", "select:export"]);
    log.borrow_mut().clear();
    cx.simulate_keystrokes("up escape");
    assert_eq!(*log.borrow(), ["open:true", "open:false", "dismiss"]);
    cx.update(|window, app| assert!(page.read(app).split.read(app).focus_handle(app).is_focused(window)));
    log.borrow_mut().clear();
    cx.simulate_keystrokes("down tab");
    cx.update(|window, app| assert!(page.read(app).next.is_focused(window)));
    assert_eq!(*log.borrow(), ["open:true", "open:false", "dismiss"]);
    cx.simulate_keystrokes("shift-tab enter");
    assert_eq!(log.borrow().last().unwrap(), "action");
}

#[test]
fn primary_disable_preserves_keyboard_menu_and_global_disable_blocks_both() {
    let mut app = TestAppContext::single();
    let (page, log, cx) = setup(&mut app, false, Default::default());
    cx.simulate_keystrokes("enter space down enter");
    assert_eq!(*log.borrow(), ["open:true", "open:false", "select:save-as"]);
    let split = cx.update(|_, app| page.read(app).split.clone());
    log.borrow_mut().clear();
    cx.simulate_keystrokes("down");
    split.update(cx, |split, cx| split.set_enabled(false, cx));
    cx.simulate_keystrokes("enter down space");
    assert_eq!(*log.borrow(), ["open:true", "open:false"]);
    split.update(cx, |split, cx| {
        split.set_enabled(true, cx);
        split.set_primary_enabled(true, cx);
        split.activate(cx);
    });
    assert_eq!(log.borrow().last().unwrap(), "action");
}

#[test]
fn pointer_faces_have_independent_feedback_and_activation() {
    let mut app = TestAppContext::single();
    let probe = Arc::new(Mutex::new(SplitButtonState::default()));
    let (page, log, cx) = setup(&mut app, true, probe.clone());
    let primary = cx.debug_bounds("test-split/primary").unwrap().center();
    let disclosure = cx.debug_bounds("test-split/disclosure").unwrap().center();
    cx.simulate_mouse_move(primary, None, Default::default());
    cx.run_until_parked();
    assert!(probe.lock().unwrap().primary.hovered);
    assert!(!probe.lock().unwrap().disclosure.hovered);
    cx.simulate_mouse_down(primary, gpui::MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert!(probe.lock().unwrap().primary.pressed);
    assert!(!probe.lock().unwrap().disclosure.pressed);
    cx.simulate_mouse_up(primary, gpui::MouseButton::Left, Default::default());
    assert_eq!(*log.borrow(), ["action"]);
    cx.simulate_mouse_down(disclosure, gpui::MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert!(!probe.lock().unwrap().primary.pressed);
    assert!(probe.lock().unwrap().disclosure.pressed);
    cx.simulate_mouse_up(disclosure, gpui::MouseButton::Left, Default::default());
    assert_eq!(*log.borrow(), ["action", "open:true"]);
    let split = cx.update(|_, app| page.read(app).split.clone());
    split.update(cx, |split, cx| {
        split.dismiss(cx);
        split.set_primary_enabled(false, cx);
    });
    log.borrow_mut().clear();
    cx.simulate_click(primary, Default::default());
    assert!(log.borrow().is_empty());
    cx.simulate_click(disclosure, Default::default());
    assert_eq!(*log.borrow(), ["open:true"]);
    cx.simulate_click(disclosure, Default::default());
    assert_eq!(*log.borrow(), ["open:true", "open:false", "dismiss"]);
    split.update(cx, |split, cx| split.set_enabled(false, cx));
    log.borrow_mut().clear();
    cx.simulate_click(primary, Default::default());
    cx.simulate_click(disclosure, Default::default());
    assert!(log.borrow().is_empty());
    cx.run_until_parked();
    assert!(probe.lock().unwrap().primary.disabled);
    assert!(probe.lock().unwrap().disclosure.disabled);
}

#[test]
fn builder_and_entity_callbacks_remain_ordered_and_allow_reentry() {
    struct Owner {
        control: Entity<SplitButton>,
        log: Vec<&'static str>,
    }
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let control = SplitButton::new("callbacks")
                .on_action_click(cx, |owner: &mut Owner, cx| {
                    owner.log.push("first");
                    owner.control.update(cx, |split, cx| split.set_enabled(false, cx));
                })
                .on_action_click(cx, |owner, _| owner.log.push("second"))
                .spawn(cx)
                .on_enabled_changed(cx, |owner, _, _| owner.log.push("disabled"));
            Owner { control, log: Vec::new() }
        })
    });
    let control = app.read(|cx| owner.read(cx).control.clone());
    app.update(|cx| control.update(cx, |control, cx| control.activate(cx)));
    app.update(|cx| control.update(cx, |control, cx| control.activate(cx)));
    app.read(|cx| assert_eq!(owner.read(cx).log, ["first", "second", "disabled"]));
}

#[test]
fn popup_builder_migration_preserves_existing_and_specialized_callbacks() {
    struct Owner {
        log: Vec<&'static str>,
    }
    let app = TestAppContext::single();
    let owner = app.update(|cx| cx.new(|_| Owner { log: Vec::new() }));
    let control = app.update(|cx| {
        owner.update(cx, |_, cx| {
            let popup = crate::controls::popup_menu::PopupMenu::new("migrated")
                .on_action_click(cx, |owner: &mut Owner, _| owner.log.push("legacy"));
            SplitButtonBuilder::from_popup_builder(popup)
                .on_action_click(cx, |owner, _| owner.log.push("specialized"))
                .spawn(cx)
        })
    });
    app.update(|cx| control.update(cx, |control, cx| control.activate(cx)));
    app.read(|cx| {
        let log = &owner.read(cx).log;
        assert_eq!(log.len(), 2);
        assert!(log.contains(&"legacy"));
        assert!(log.contains(&"specialized"));
    });
}

#[test]
fn split_button_text_is_empty_until_supplied() {
    let bare = SplitButtonBuilder::new("bare");
    assert!(bare.model.popup.split_text.usage_hint.is_empty());
    assert!(bare.model.popup.split_text.primary_unavailable.is_empty());
    assert!(bare.model.popup.split_text.menu_label.is_none());

    let text = SplitButtonText {
        usage_hint: "Entrée".into(),
        primary_unavailable: "Indisponible".into(),
        menu_label: Some(Arc::new(|label| format!("Menu {label}").into())),
    };
    let built = SplitButtonBuilder::new("text").text(text);
    assert_eq!(built.model.popup.split_text.usage_hint, "Entrée");
    assert_eq!(built.model.popup.split_text.primary_unavailable, "Indisponible");
    assert_eq!((built.model.popup.split_text.menu_label.as_ref().unwrap())("Save"), "Menu Save");
}
