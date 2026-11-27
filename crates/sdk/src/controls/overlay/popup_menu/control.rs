use crate::infra::focus_report::ReportsFocus as _;
use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use gpui::{
    App, Bounds, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, IntoElement, MouseDownEvent,
    MouseUpEvent, Pixels, Render, SharedString, Window, div, prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;

use crate::controls::button::ControlIcon;

use super::{PopupMenuBuilder, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplateHandlers, MenuPath};
use crate::motion::{DisclosureMotion, VisualTransition};
use crate::controls::popup_menu::model::PopupMenuModel;
use crate::controls::floating_menu::{FloatingMenuActivateResult, FloatingMenuState, FloatingMenuStepDirection};
use crate::infra::interaction::ControlInteraction;
use crate::infra::menu_navigation::MenuNavigator;
use crate::motion::popup_lifecycle::PopupLifecycle;
use crate::controls::floating_menu::FloatingMenuHighlight;
use crate::infra::state::ControlFocusState;
use crate::focus::EscapeFocus;
use crate::key_handling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PopupMenuEvent {
    ActionClick,
    Select { item_id: SharedString, label: SharedString },
    OpenChanged { open: bool },
    Dismiss,
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
}

pub struct PopupMenu {
    model: PopupMenuModel,
    attachments: AttachmentHost,
    lifecycle: PopupLifecycle,
    menu_state: FloatingMenuState,
    highlight_from: Option<MenuPath>,
    highlight_to: Option<MenuPath>,
    highlight_transition: VisualTransition,
    disclosure_transition: DisclosureMotion,
    submenu_transition: DisclosureMotion,
    submenu_transition_index: Option<usize>,
    submenu_target_open: bool,
    interaction: ControlInteraction,
    focus_report: crate::infra::focus_report::FocusReport,
    split_state: crate::controls::split_button::SplitButtonState,
    focus_subscriptions: crate::infra::focus_report::FocusSubscriptions,
}

impl EventEmitter<PopupMenuEvent> for PopupMenu {}

impl PopupMenu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopupMenuBuilder {
        PopupMenuBuilder::new(id)
    }

    pub fn motion_enabled(&self) -> bool {
        self.model.motion.enabled()
    }
    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.motion.set_enabled(enabled);
        self.lifecycle.set_motion_enabled(enabled);
        self.disclosure_transition.set_motion_enabled(enabled);
        self.submenu_transition.set_motion_enabled(enabled);
        if !enabled {
            self.highlight_transition.finish();
        }
        cx.notify();
    }

    crate::motion_snapshot!(super::PopupMenuMotion, model.motion);

    pub(crate) fn from_builder(builder: PopupMenuBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion.enabled();
        let enabled = builder.model.enabled;
        let tab_stop = builder.model.tab_stop;

        Self {
            model: builder.model,
            attachments: AttachmentHost::with_accessible_role(gpui::Role::Button),
            lifecycle: PopupLifecycle::with_motion(motion_enabled, motion.presence.clone()),
            menu_state: FloatingMenuState::default(),
            highlight_from: None,
            highlight_to: None,
            highlight_transition: VisualTransition::with_transition(1.0, motion.highlight.enabled(motion_enabled)),
            disclosure_transition: DisclosureMotion::with_transition(0.0, motion_enabled, motion.disclosure.clone()),
            submenu_transition: DisclosureMotion::with_transition(0.0, motion_enabled, motion.submenu.clone()),
            submenu_transition_index: None,
            submenu_target_open: false,
            interaction: ControlInteraction::new_with_tab_stop(enabled, tab_stop, cx),
            focus_report: Default::default(),
            split_state: Default::default(),
            focus_subscriptions: Default::default(),
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    /// Replace the trigger face presenter and clear the icon-only preset.
    pub fn set_presenter(
        &mut self,
        content: crate::infra::presenter::ControlPresenter<super::PopupMenuTriggerModel>,
        cx: &mut Context<Self>,
    ) {
        self.model.content = content;
        self.model.icon_only = false;
        self.model.icon = None;
        cx.notify();
    }

    /// Icon-only face preset: [`super::icon_content`] + square chrome (`icon_only`).
    ///
    /// Not a parallel field beside the presenter. Mirrors [`super::PopupMenuBuilder::icon`].
    /// Lucide glyphs also fill `icon`; SVG paths render through the presenter.
    pub fn set_icon(&mut self, icon: impl Into<ControlIcon>, cx: &mut Context<Self>) {
        let icon = icon.into();
        self.model.content = super::icon_content(icon.clone());
        self.model.icon_only = true;
        self.model.icon = match icon {
            ControlIcon::Lucide(icon) => Some(icon),
            ControlIcon::SvgPath(_) => None,
        };
        cx.notify();
    }

    /// Trailing adornment on a labeled trigger (distinct from the icon face preset).
    pub fn set_end_icon(&mut self, icon: Option<LucideIcon>, cx: &mut Context<Self>) {
        self.model.end_icon = icon;
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

    /// Whether the control takes input.
    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let changed = self.model.enabled != enabled;
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        self.focus_subscriptions.reset();
        if !enabled {
            self.split_state = Default::default();
            self.close_menu(cx);
            self.emit_focus_changed(false, cx);
        }
        if changed {
            cx.emit(PopupMenuEvent::EnabledChanged { enabled });
        }
        cx.notify();
    }

    /// Draw the trigger as `:invalid`.
    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.model.invalid != invalid {
            self.model.invalid = invalid;
            cx.notify();
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.model.invalid
    }

    /// Disable only the primary face of a split trigger, preserving menu access.
    pub fn set_primary_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.primary_enabled = enabled;
        if !enabled {
            self.split_state.primary = Default::default();
        }
        cx.notify();
    }

    pub(crate) fn split_state(&self, window: &Window) -> crate::controls::split_button::SplitButtonState {
        let mut state = self.split_state.resolved(
            self.model.enabled,
            self.model.primary_enabled,
            self.interaction.render_state(self.model.enabled, window).focused,
        );
        // Both faces of an invalid split trigger draw `:invalid`.
        state.primary.invalid = self.model.invalid;
        state.disclosure.invalid = self.model.invalid;
        state
    }

    pub(crate) fn attachment_state(&self, window: &Window) -> crate::theme::InteractionState {
        let mut state = self.interaction.render_state(self.model.enabled, window);
        state.disabled |= self.lifecycle.is_open();
        state
    }

    pub fn set_tab_stop(&mut self, tab_stop: bool, cx: &mut Context<Self>) {
        self.model.tab_stop = tab_stop;
        self.interaction.set_tab_stop(self.model.enabled, tab_stop);
        self.focus_subscriptions.reset();
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::PopupMenuTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_placement(&mut self, placement: PopupMenuPlacement, cx: &mut Context<Self>) {
        self.model.placement = placement;
        cx.notify();
    }

    pub fn set_trigger_style(&mut self, style: super::PopupMenuTriggerStyle, cx: &mut Context<Self>) {
        self.model.trigger_style = style;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> PopupMenuRenderModel<'a> {
        PopupMenuRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            content: self.model.content.clone(),
            items: &self.model.items,
            open: self.lifecycle.is_open(),
            disclosure_progress: self.disclosure_transition.progress(),
            presence: self.lifecycle.presence(),
            submenu_transition: self.submenu_transition_index.map(|index| (index, self.submenu_transition.progress())),
            trigger_bounds: self.lifecycle.trigger_bounds(),
            placement: self.model.placement,
            trigger_style: self.model.trigger_style,
            icon_only: self.model.icon_only,
            icon: self.model.icon,
            end_icon: self.model.end_icon,
            disclosure_icons: &self.model.disclosure_icons,
            full_width: self.model.full_width,
            without_elevation: self.model.without_elevation,
            split: self.model.split,
            split_state: self.split_state(window),
            trigger_radius_override: None,
            open_submenu: self.menu_state.open_submenu(),
            active_path: self.menu_state.active_path(),
            highlight: self.highlight_model(),
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, self.interaction.focus_handle(), window),
            state: self.render_state(window),
        }
    }

    /// The trigger's interaction state, with `invalid` from the model.
    fn render_state(&self, window: &Window) -> crate::theme::InteractionState {
        let mut state = self.interaction.render_state(self.model.enabled, window);
        state.invalid = self.model.invalid;
        state
    }

    fn highlight_model(&self) -> Option<FloatingMenuHighlight> {
        Some(FloatingMenuHighlight {
            from: self.highlight_from?,
            to: self.highlight_to?,
            progress: self.highlight_transition.progress(),
        })
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

    fn sync_highlight(&mut self, previous: Option<MenuPath>) {
        let next = self.menu_state.active_path();
        if previous == next {
            return;
        }

        let same_surface = matches!(
            (previous, next),
            (Some(MenuPath::Root(_)), Some(MenuPath::Root(_)))
                | (Some(MenuPath::Submenu { .. }), Some(MenuPath::Submenu { .. }))
        );
        if same_surface {
            self.highlight_from = previous;
            self.highlight_to = next;
            self.highlight_transition = VisualTransition::with_transition(
                0.0,
                self.model.motion.profile().highlight.enabled(self.model.motion.enabled()),
            );
            self.highlight_transition.set_target(1.0);
        } else {
            self.highlight_from = next;
            self.highlight_to = next;
            self.highlight_transition.snap_to(1.0);
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> PopupMenuTemplateHandlers {
        let item_paths = self.menu_state.item_click_paths(&self.model.items);

        PopupMenuTemplateHandlers {
            split_faces: Some(super::template::SplitFaceHandlers {
                primary_hover: Box::new(cx.listener(|this, hovered, _, cx| {
                    this.split_state.primary.hovered = *hovered;
                    cx.notify();
                })),
                disclosure_hover: Box::new(cx.listener(|this, hovered, _, cx| {
                    this.split_state.disclosure.hovered = *hovered;
                    cx.notify();
                })),
                primary_down: Box::new(cx.listener(|this, _, _, cx| {
                    this.split_state.primary.pressed = this.model.enabled && this.model.primary_enabled;
                    cx.notify();
                })),
                disclosure_down: Box::new(cx.listener(|this, _, _, cx| {
                    this.split_state.disclosure.pressed = this.model.enabled;
                    cx.notify();
                })),
            }),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
            action_click: Box::new(cx.listener(Self::handle_action_click)),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_hover: Box::new(cx.listener(Self::handle_hover)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            trigger_mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            trigger_mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            root_mouse_down_out: Box::new(cx.listener(Self::handle_mouse_down_out)),
            item_hovers: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_item_hover(index, *hovered, cx);
                    })) as _
                })
                .collect(),
            submenu_hovers: self
                .model
                .items
                .iter()
                .enumerate()
                .map(|(parent, item)| {
                    item.submenu_items()
                        .iter()
                        .enumerate()
                        .map(|(child, _)| {
                            Box::new(cx.listener(move |this, hovered, _window, cx| {
                                if *hovered {
                                    let previous = this.menu_state.active_path();
                                    if this.menu_state.hover_submenu_item(&this.model.items, parent, child) {
                                        this.sync_highlight(previous);
                                        cx.notify();
                                    }
                                }
                            })) as _
                        })
                        .collect()
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

    fn close_menu(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.lifecycle.is_open();
        let previous = self.menu_state.active_path();
        self.lifecycle.close();
        self.disclosure_transition.set_target(0.0);
        self.menu_state.clear();
        self.sync_highlight(previous);
        if was_open {
            cx.emit(PopupMenuEvent::OpenChanged { open: false });
        }
        was_open
    }

    fn dismiss_menu(&mut self, cx: &mut Context<Self>) -> bool {
        let closed = self.close_menu(cx);
        if closed {
            cx.emit(PopupMenuEvent::Dismiss);
        }
        closed
    }

    /// Closes the popup and emits the normal dismissal events when it was open.
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if self.dismiss_menu(cx) {
            cx.notify();
        }
    }

    fn open_menu_with(&mut self, active_path: Option<MenuPath>, cx: &mut Context<Self>) -> bool {
        let was_open = self.lifecycle.is_open();
        let previous = self.menu_state.active_path();
        let selection_changed = self.menu_state.open_with(active_path);
        let changed = !was_open || selection_changed;
        self.lifecycle.open();
        self.disclosure_transition.set_target(1.0);
        self.sync_highlight(previous);
        if !was_open {
            cx.emit(PopupMenuEvent::OpenChanged { open: true });
        }
        changed
    }

    fn select_item_at_path(&mut self, path: &[usize], cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        if let Some((item_id, label)) = self.menu_state.select_at_path(&self.model.items, path) {
            self.close_menu(cx);
            cx.emit(PopupMenuEvent::Select { item_id, label });
            return true;
        }

        false
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.lifecycle.set_trigger_bounds(*bounds);
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.model.enabled {
            if self.lifecycle.is_open() {
                self.dismiss_menu(cx);
            } else {
                self.open_menu_with(None, cx);
            }
            cx.notify();
        }
    }

    fn handle_action_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !event.is_keyboard() && self.model.enabled {
            self.activate_primary(cx);
        }
    }

    pub(crate) fn activate_primary(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.primary_enabled {
            return;
        }
        self.dismiss_menu(cx);
        cx.emit(PopupMenuEvent::ActionClick);
        cx.notify();
    }

    fn handle_item_click(&mut self, path: &[usize], event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.select_item_at_path(path, cx) {
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !hovered || !self.lifecycle.is_open() {
            return;
        }

        let previous = self.menu_state.active_path();
        if self.menu_state.hover_root_item(&self.model.items, index) {
            self.sync_highlight(previous);
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.emit(PopupMenuEvent::HoverChanged { hovered: *hovered });
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_down(self.model.enabled, window, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.split_state.primary.pressed = false;
        self.split_state.disclosure.pressed = false;
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_mouse_down_out(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open()
            && self.model.dismiss.click_outside
            && !self.lifecycle.is_inside_trigger(event.position)
        {
            self.dismiss_menu(cx);
            cx.notify();
        }
    }

    fn open_menu_at_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let navigator = MenuNavigator::new(&self.model.items);
        let active_path = if first {
            navigator.first_root()
        } else {
            navigator.last_root()
        }
        .map(MenuPath::Root);

        if self.open_menu_with(active_path, cx) {
            cx.notify();
        }
    }

    fn step_active_item(&mut self, direction: FloatingMenuStepDirection, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.lifecycle.is_open() {
            return;
        }

        let previous = self.menu_state.active_path();
        if self.menu_state.step(&self.model.items, direction) {
            self.sync_highlight(previous);
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.lifecycle.is_open() {
            return;
        }

        let previous = self.menu_state.active_path();
        if self.menu_state.move_to_boundary(&self.model.items, first) {
            self.sync_highlight(previous);
            cx.notify();
        }
    }

    fn open_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.lifecycle.is_open() {
            return;
        }

        let previous = self.menu_state.active_path();
        if self.menu_state.open_active_submenu(&self.model.items) {
            self.sync_highlight(previous);
            cx.notify();
        }
    }

    fn close_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.lifecycle.is_open() {
            return;
        }

        let previous = self.menu_state.active_path();
        if self.menu_state.close_active_submenu() {
            self.sync_highlight(previous);
            cx.notify();
        }
    }

    fn activate_active_item(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if !self.lifecycle.is_open() {
            self.open_menu_at_boundary(true, cx);
            return;
        }

        let previous = self.menu_state.active_path();
        match self.menu_state.activate(&self.model.items) {
            FloatingMenuActivateResult::None => {}
            FloatingMenuActivateResult::OpenedSubmenu => {
                self.sync_highlight(previous);
                cx.notify();
            }
            FloatingMenuActivateResult::Select { item_id, label } => {
                self.close_menu(cx);
                cx.emit(PopupMenuEvent::Select { item_id, label });
                cx.notify();
            }
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.step_active_item(FloatingMenuStepDirection::Previous, cx);
        } else if self.model.tab_stop {
            self.open_menu_at_boundary(false, cx);
        } else {
            cx.propagate();
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.step_active_item(FloatingMenuStepDirection::Next, cx);
        } else if self.model.tab_stop {
            self.open_menu_at_boundary(true, cx);
        } else {
            cx.propagate();
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.move_active_to_boundary(true, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.move_active_to_boundary(false, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.open_active_submenu(cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() {
            self.close_active_submenu(cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.split && !self.lifecycle.is_open() {
            self.activate_primary(cx);
        } else {
            self.activate_active_item(cx);
        }
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.lifecycle.is_open() && self.model.dismiss.escape {
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
        self.split_state = Default::default();
        self.interaction.handle_mouse_up();
        let focused = self.emit_focus_changed(false, cx);
        let closed = self.model.dismiss.focus_loss && self.dismiss_menu(cx);
        if focused || closed {
            cx.notify();
        }
    }
}

impl Focusable for PopupMenu {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_submenu_transition_target();
        let focus_handle = self.interaction.focus_handle().clone();
        self.focus_subscriptions
            .ensure(&focus_handle, window, cx, Self::handle_focus_in, Self::handle_focus_out);

        if self.lifecycle.sync() {
            self.lifecycle.schedule_frame(window, cx);
        }

        if self.disclosure_transition.sync() {
            self.disclosure_transition.schedule_frame(window, cx);
        }

        if self.submenu_transition.sync() {
            self.submenu_transition.schedule_frame(window, cx);
        }

        if self.highlight_transition.sync() {
            self.highlight_transition.schedule_frame(window, cx);
        }

        let mut tip_state = self.interaction.render_state(self.model.enabled, window);
        tip_state.disabled |= self.lifecycle.is_open();
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);
        let root = self.model.template.render(&model, handlers, window, cx);
        let root = self.attachments.render(root, tip_state, cx);

        div()
            .child(
                root.track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::Menu.context())
                    .on_action(cx.listener(Self::handle_escape_focus))
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

impl AttachmentTarget for PopupMenu {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
    fn attachment_anchor(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        self.lifecycle.trigger_bounds().unwrap_or(bounds)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod dismiss_tests {
    use gpui::{Entity, TestAppContext};

    use super::*;
    use crate::infra::menu_item::MenuItem;
    use crate::infra::popup_dismiss::PopupDismiss;

    fn open_menu_with_policy(
        cx: &mut TestAppContext,
        dismiss: PopupDismiss,
    ) -> (Entity<PopupMenu>, &mut gpui::VisualTestContext) {
        let (menu, cx) = cx.add_window_view(|_, cx| {
            PopupMenu::from_builder(PopupMenu::new("menu").items([MenuItem::new("one")]).dismiss(dismiss), cx)
        });
        menu.update(cx, |menu, cx| {
            menu.open_menu_with(None, cx);
        });
        (menu, cx)
    }

    #[gpui::test]
    fn escape_and_focus_loss_follow_the_policy(cx: &mut TestAppContext) {
        let (menu, cx) = open_menu_with_policy(cx, PopupDismiss::NEVER);
        menu.update_in(cx, |menu, window, cx| {
            menu.handle_escape_focus(&EscapeFocus, window, cx);
            menu.handle_focus_out(FocusOutEvent { blurred: cx.focus_handle().downgrade() }, window, cx);
            assert!(menu.lifecycle.is_open(), "NEVER keeps the menu open");
        });

        menu.update_in(cx, |menu, window, cx| {
            menu.model.dismiss = PopupDismiss::LIGHT;
            menu.handle_escape_focus(&EscapeFocus, window, cx);
            assert!(!menu.lifecycle.is_open(), "Escape closes under LIGHT");
        });
    }
}

impl crate::infra::focus_report::ReportsFocus for PopupMenu {
    type FocusEvent = PopupMenuEvent;

    fn focus_report(&mut self) -> &mut crate::infra::focus_report::FocusReport {
        &mut self.focus_report
    }

    fn focus_event(focused: bool) -> PopupMenuEvent {
        PopupMenuEvent::FocusChanged { focused }
    }

    fn focus_enabled(&self) -> bool {
        self.model.enabled
    }
}
