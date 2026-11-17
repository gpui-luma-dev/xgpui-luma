#[cfg(test)]
#[path = "locale_tests.rs"]
mod locale_tests;

use gpui::{
    App, Bounds, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, IntoElement, MouseDownEvent,
    MouseUpEvent, Pixels, Point, Render, SharedString, Subscription, Window, div, point, prelude::*, px, relative,
};

use super::{ContextMenuBuilder, ContextMenuRenderModel, ContextMenuTemplateHandlers};
use crate::controls::context_menu::model::ContextMenuModel;
use crate::controls::floating_menu::{FloatingMenuActivateResult, FloatingMenuState, FloatingMenuStepDirection};
use crate::infra::interaction::ControlInteraction;
use crate::infra::locale::{LayoutDirection, LocaleContext};
use crate::infra::menu_navigation::MenuNavigator;
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::state::{ControlFocusState, MenuPath};
use crate::motion::DisclosureMotion;
use crate::focus::EscapeFocus;
use crate::key_handling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenContextMenu, OpenSubmenu, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ContextMenuEvent {
    Select { item_id: SharedString, label: SharedString },
    OpenChanged { open: bool },
    Dismiss,
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
}

pub struct ContextMenu {
    model: ContextMenuModel,
    menu_position: Option<Point<Pixels>>,
    paint_position: Option<Point<Pixels>>,
    target_bounds: Option<Bounds<Pixels>>,
    menu_state: FloatingMenuState,
    presence: OverlayPresence,
    submenu_presence: OverlayPresence,
    submenu_parent: Option<usize>,
    submenu_transition: DisclosureMotion,
    submenu_transition_index: Option<usize>,
    submenu_target_open: bool,
    interaction: ControlInteraction,
    emitted_focused: bool,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
}

impl EventEmitter<ContextMenuEvent> for ContextMenu {}

impl ContextMenu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ContextMenuBuilder {
        ContextMenuBuilder::new(id)
    }

    crate::presence_policy!(presence, submenu_presence, submenu_transition);

    crate::motion_snapshot!(super::ContextMenuMotion, model.motion);

    pub(crate) fn from_builder(builder: ContextMenuBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion.enabled();
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            menu_position: None,
            paint_position: None,
            target_bounds: None,
            menu_state: FloatingMenuState::default(),
            presence: OverlayPresence::with_motion(false, motion_enabled, motion.presence.clone()),
            submenu_presence: OverlayPresence::with_motion(false, motion_enabled, motion.submenu.clone()),
            submenu_parent: None,
            submenu_transition: DisclosureMotion::with_transition(0.0, motion_enabled, motion.disclosure.clone()),
            submenu_transition_index: None,
            submenu_target_open: false,
            interaction: ControlInteraction::new(enabled, cx),
            emitted_focused: false,
            focus_in_subscription: None,
            focus_out_subscription: None,
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = crate::infra::menu_item::MenuItem>,
        cx: &mut Context<Self>,
    ) {
        self.model.items = items.into_iter().collect();
        self.close_menu(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let changed = self.model.enabled != enabled;
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;
        if !enabled {
            self.close_menu(cx);
            self.emit_focus_changed(false, cx);
        }
        if changed {
            cx.emit(ContextMenuEvent::EnabledChanged { enabled });
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::ContextMenuTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> ContextMenuRenderModel<'a> {
        ContextMenuRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            target_content: self.model.target_content.as_ref(),
            items: &self.model.items,
            menu_position: self
                .menu_position
                .or_else(|| self.presence.should_paint().then_some(self.paint_position).flatten()),
            presence: self.presence.clone(),
            open_submenu: self
                .menu_state
                .open_submenu()
                .or_else(|| self.submenu_presence.should_paint().then_some(self.submenu_parent).flatten()),
            active_path: self.menu_state.active_path(),
            submenu_presence: self.submenu_presence.clone(),
            submenu_transition: self.submenu_transition_index.map(|index| (index, self.submenu_transition.progress())),
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, self.interaction.focus_handle(), window),
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ContextMenuTemplateHandlers {
        let item_paths = self.menu_state.item_click_paths(&self.model.items);

        ContextMenuTemplateHandlers {
            target_bounds: Box::new(cx.listener(Self::handle_target_bounds)),
            target_aux_click: Box::new(cx.listener(Self::handle_target_aux_click)),
            target_hover: Box::new(cx.listener(Self::handle_hover)),
            target_mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            target_mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            target_mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            root_mouse_down_out: Box::new(cx.listener(Self::handle_mouse_down_out)),
            item_hovers: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_item_hover(index, *hovered, cx);
                    })) as _
                })
                .collect(),
            item_clicks: item_paths
                .into_iter()
                .map(|path| {
                    Box::new(cx.listener(move |this, event, window, cx| {
                        this.handle_item_click(&path, event, window, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn open_menu_at(&mut self, position: Point<Pixels>, active_path: Option<MenuPath>, cx: &mut Context<Self>) -> bool {
        let was_open = self.menu_position.is_some();
        let state_changed = self.menu_state.open_with(active_path);
        let changed = self.menu_position != Some(position) || state_changed;

        self.menu_position = Some(position);
        self.paint_position = Some(position);
        self.presence.set_open(true);
        self.sync_submenu_presence();
        if !was_open {
            cx.emit(ContextMenuEvent::OpenChanged { open: true });
        }

        changed
    }

    fn close_menu(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.menu_position.is_some();
        self.menu_position = None;
        self.menu_state.clear();
        self.presence.set_open(false);
        self.sync_submenu_presence();
        if was_open {
            cx.emit(ContextMenuEvent::OpenChanged { open: false });
        }
        was_open
    }

    fn sync_submenu_presence(&mut self) {
        let next_parent = self.menu_state.open_submenu();
        if let Some(next) = next_parent {
            if self.submenu_parent.is_some_and(|old| old != next) {
                self.submenu_presence.snap_open(false);
            }
            self.submenu_parent = Some(next);
        }
        self.submenu_presence.set_open(next_parent.is_some());
        if !self.submenu_presence.should_paint() {
            self.submenu_parent = None;
        }
    }

    fn sync_submenu_transition_target(&mut self) {
        let next = self.menu_state.open_submenu();
        let next_open = next.is_some();
        if next_open != self.submenu_target_open {
            self.submenu_target_open = next_open;
            self.submenu_transition.set_target(if next_open { 1.0 } else { 0.0 });
        }
        if let Some(index) = next {
            self.submenu_transition_index = Some(index);
        }
    }

    fn dismiss_menu(&mut self, cx: &mut Context<Self>) -> bool {
        let closed = self.close_menu(cx);
        if closed {
            cx.emit(ContextMenuEvent::Dismiss);
        }
        closed
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }
        self.emitted_focused = focused;
        cx.emit(ContextMenuEvent::FocusChanged { focused });
        true
    }

    fn keyboard_menu_position(&self) -> Point<Pixels> {
        self.target_bounds
            .map(|bounds| point(bounds.left(), bounds.bottom()))
            .unwrap_or_else(|| point(px(0.0), px(0.0)))
    }

    fn select_item_at_path(&mut self, path: &[usize], cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        if let Some((item_id, label)) = self.menu_state.select_at_path(&self.model.items, path) {
            self.close_menu(cx);
            cx.emit(ContextMenuEvent::Select { item_id, label });
            return true;
        }

        false
    }

    fn handle_target_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.target_bounds = Some(*bounds);
    }

    fn handle_target_aux_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled
            && event.is_right_click()
            && let Some(position) = event.mouse_position()
        {
            self.open_menu_at(position, None, cx);
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, path: &[usize], event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_none() {
            return;
        }
        if event.is_keyboard() {
            return;
        }

        if self.select_item_at_path(path, cx) {
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if self.menu_position.is_none() {
            return;
        }
        if !hovered || self.menu_position.is_none() {
            return;
        }

        if self.menu_state.hover_root_item(&self.model.items, index) {
            self.sync_submenu_presence();
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.emit(ContextMenuEvent::HoverChanged { hovered: *hovered });
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_secondary_mouse_down(self.model.enabled) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_mouse_down_out(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.dismiss_menu(cx) {
            cx.notify();
        }
    }

    fn open_keyboard_menu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let navigator = MenuNavigator::new(&self.model.items);
        let active_path = navigator.first_root().map(MenuPath::Root);

        if self.open_menu_at(self.keyboard_menu_position(), active_path, cx) {
            cx.notify();
        }
    }

    fn step_active_item(&mut self, direction: FloatingMenuStepDirection, cx: &mut Context<Self>) {
        if !self.model.enabled || self.menu_position.is_none() {
            return;
        }

        if self.menu_state.step(&self.model.items, direction) {
            self.sync_submenu_presence();
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || self.menu_position.is_none() {
            return;
        }

        if self.menu_state.move_to_boundary(&self.model.items, first) {
            self.sync_submenu_presence();
            cx.notify();
        }
    }

    fn open_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || self.menu_position.is_none() {
            return;
        }

        if self.menu_state.open_active_submenu(&self.model.items) {
            self.sync_submenu_presence();
            cx.notify();
        }
    }

    fn close_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || self.menu_position.is_none() {
            return;
        }

        if self.menu_state.close_active_submenu() {
            self.sync_submenu_presence();
            cx.notify();
        }
    }

    fn activate_active_item(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || self.menu_position.is_none() {
            return;
        }

        match self.menu_state.activate(&self.model.items) {
            FloatingMenuActivateResult::None => {}
            FloatingMenuActivateResult::OpenedSubmenu => {
                self.sync_submenu_presence();
                cx.notify();
            }
            FloatingMenuActivateResult::Select { item_id, label } => {
                self.close_menu(cx);
                cx.emit(ContextMenuEvent::Select { item_id, label });
                cx.notify();
            }
        }
    }

    fn handle_open_context_menu(&mut self, _: &OpenContextMenu, _window: &mut Window, cx: &mut Context<Self>) {
        self.open_keyboard_menu(cx);
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.step_active_item(FloatingMenuStepDirection::Previous, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.step_active_item(FloatingMenuStepDirection::Next, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.move_active_to_boundary(true, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.move_active_to_boundary(false, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            if LocaleContext::current(cx).direction == LayoutDirection::Rtl {
                self.close_active_submenu(cx);
            } else {
                self.open_active_submenu(cx);
            }
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            if LocaleContext::current(cx).direction == LayoutDirection::Rtl {
                self.open_active_submenu(cx);
            } else {
                self.close_active_submenu(cx);
            }
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.activate_active_item(cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_position.is_some() {
            self.dismiss_menu(cx);
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let focused = self.emit_focus_changed(false, cx);
        let closed = self.dismiss_menu(cx);
        if focused || closed {
            cx.notify();
        }
    }
}

impl Focusable for ContextMenu {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for ContextMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_submenu_transition_target();
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.interaction.focus_handle().clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.interaction.focus_handle().clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        self.presence.sync();
        self.presence.schedule_frame(window, cx);
        self.submenu_presence.sync();
        self.submenu_presence.schedule_frame(window, cx);
        if self.submenu_transition.sync() {
            self.submenu_transition.schedule_frame(window, cx);
        }

        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        let mut root = div();
        if self.model.target_content.is_some() {
            root = root.w_full().min_h(relative(1.0));
        }

        root.child(
            self.model
                .template
                .render(&model, handlers, window, cx)
                .track_focus(self.interaction.focus_handle())
                .key_context(ControlKeyProfile::ContextMenu.context())
                .on_action(cx.listener(Self::handle_escape_focus))
                .on_action(cx.listener(Self::handle_open_context_menu))
                .on_action(cx.listener(Self::handle_select_previous_item))
                .on_action(cx.listener(Self::handle_select_next_item))
                .on_action(cx.listener(Self::handle_select_first_item))
                .on_action(cx.listener(Self::handle_select_last_item))
                .on_action(cx.listener(Self::handle_open_submenu))
                .on_action(cx.listener(Self::handle_close_submenu))
                .on_action(cx.listener(Self::handle_activate_control)),
        )
        .into_any_element()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod motion_tests {
    use super::*;
    use crate::infra::menu_item::MenuItem;
    use crate::motion::{Transition, Easing};
    use gpui::{AppContext, TestAppContext};
    use std::time::Duration;

    #[test]
    fn custom_exit_retains_paint_position_and_submenu_after_logical_dismissal() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let entity = cx.new(|cx| {
                ContextMenu::from_builder(
                    ContextMenu::new("exit").items([MenuItem::new("parent").submenu([MenuItem::new("child")])]).motion(
                        |m| {
                            m.presence.exit = Transition::timed(Duration::from_secs(1), Easing::Linear);
                            m.submenu.exit = Transition::timed(Duration::from_millis(500), Easing::Linear);
                        },
                    ),
                    cx,
                )
            });
            entity.update(cx, |menu, cx| {
                menu.presence.elapse_for_test(Duration::ZERO);
                let position = point(px(20.), px(30.));
                menu.open_menu_at(position, None, cx);
                menu.presence.elapse_for_test(Duration::from_millis(200));
                menu.presence.sync();
                menu.submenu_presence.elapse_for_test(Duration::ZERO);
                menu.menu_state.hover_root_item(&menu.model.items, 0);
                menu.sync_submenu_presence();
                menu.submenu_presence.elapse_for_test(Duration::from_millis(200));
                menu.submenu_presence.sync();
                assert!(menu.close_menu(cx));
                assert!(menu.menu_position.is_none());
                assert_eq!(menu.paint_position, Some(position));
                assert_eq!(menu.submenu_parent, Some(0));
                assert!(menu.presence.should_paint() && menu.submenu_presence.should_paint());
                menu.handle_item_hover(0, true, cx);
                assert!(menu.menu_state.open_submenu().is_none());
                menu.submenu_presence.elapse_for_test(Duration::from_millis(500));
                menu.submenu_presence.sync();
                assert!(!menu.submenu_presence.should_paint());
                menu.presence.elapse_for_test(Duration::from_secs(1));
                menu.presence.sync();
                assert!(!menu.presence.should_paint());
            });
        });
    }

    #[test]
    fn default_close_is_immediate_and_disabled_policy_survives_open() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let entity = cx.new(|cx| ContextMenu::from_builder(ContextMenu::new("default-exit"), cx));
            entity.update(cx, |menu, cx| {
                menu.open_menu_at(point(px(20.), px(30.)), None, cx);
                menu.close_menu(cx);
                assert!(!menu.presence.should_paint());
                menu.set_motion_enabled(false, cx);
                menu.open_menu_at(point(px(20.), px(30.)), None, cx);
                assert_eq!(menu.presence.opacity(), 1.);
                assert!(!menu.presence.is_animating());
            });
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod dispatch_tests {
    use super::*;
    use crate::infra::menu_item::MenuItem;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn clicking_menu_outside_target_selects_before_dismissal_and_outside_click_still_closes() {
        let mut app = TestAppContext::single();
        let (menu, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            ContextMenu::from_builder(
                ContextMenu::new("context-menu-regression")
                    .label("Target")
                    .item(MenuItem::new("copy").label("Copy"))
                    .motion_enabled(false),
                cx,
            )
        });
        let selected = Rc::new(RefCell::new(Vec::new()));
        let _subscription = cx.update(|_, cx| {
            let selected = selected.clone();
            cx.subscribe(&menu, move |_, event: &ContextMenuEvent, _| {
                if let ContextMenuEvent::Select { item_id, .. } = event {
                    selected.borrow_mut().push(item_id.clone());
                }
            })
        });
        let position = point(px(300.0), px(160.0));
        cx.update(|_, cx| {
            menu.update(cx, |menu, cx| {
                menu.open_menu_at(position, None, cx);
                cx.notify();
            })
        });
        cx.run_until_parked();
        // The item is deliberately outside the target's bounds. Dismissing
        // against the target would close on mouse-down and lose this click.
        cx.simulate_click(point(px(320.0), px(180.0)), Default::default());
        cx.run_until_parked();
        assert_eq!(*selected.borrow(), vec![SharedString::from("copy")]);
        cx.update(|_, cx| {
            assert!(menu.read(cx).menu_position.is_none());
            menu.update(cx, |menu, cx| {
                menu.open_menu_at(position, None, cx);
                cx.notify();
            });
        });
        cx.run_until_parked();
        cx.simulate_click(point(px(600.0), px(400.0)), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert!(menu.read(cx).menu_position.is_none()));
        assert_eq!(selected.borrow().len(), 1);
    }
}
