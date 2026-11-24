use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, FocusHandle, FocusOutEvent, Focusable, IntoElement,
    MouseDownEvent, MouseUpEvent, Render, ScrollEvent, SharedString, Subscription, Window, div, prelude::*, px,
};

use crate::controls::popup_scroll_surface::PopupScrollSurface;
use crate::controls::scrollbar::{ScrollbarEvent, ScrollbarTemplate};
use super::item_template::{SelectionPanelItemTemplate, make_selection_panel_item_template};
use super::model::{
    SelectionPanelRole, HoverActivationPolicy, SelectionPanelBuilder, SelectionPanelLookProvider, SelectionPanelItem,
    SelectionPanelItemLike, SelectionPanelModel, default_selection_panel_model,
};
use super::template::SelectionPanelTemplate;
use super::template::{
    SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler, SelectionPanelRenderModel, render_selection_panel,
};
use crate::infra::state::ControlFocusState;
use crate::key_handling::{
    ActivateControl, ControlKeyProfile, DecreaseValueLarge, IncreaseValueLarge, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectionPanelEvent {
    HoverChanged { visible_index: Option<usize> },
    ActivateRow { source_index: usize, visible_index: usize, item_id: SharedString },
    ActiveIndexChanged { visible_index: Option<usize> },
    FocusChanged { focused: bool },
    OpenChanged { open: bool },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionPanelState {
    active_visible_index: Option<usize>,
    hovered_visible_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelStepDirection {
    Previous,
    Next,
}

pub struct SelectionPanelControl<T = SelectionPanelItem>
where
    T: SelectionPanelItemLike + 'static,
{
    model: SelectionPanelModel<T>,
    state: SelectionPanelState,
    pressed_visible_index: Option<usize>,
    focus_handle: FocusHandle,
    popup_surface: PopupScrollSurface,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    emitted_focused: bool,
    _subscriptions: Vec<Subscription>,
}

impl<T> EventEmitter<SelectionPanelEvent> for SelectionPanelControl<T> where T: SelectionPanelItemLike + 'static {}

impl SelectionPanelControl<SelectionPanelItem> {
    pub fn new(id: impl Into<SharedString>, cx: &mut impl AppContext) -> Entity<Self> {
        Self::new_typed(id, cx)
    }
}

impl<T> SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub fn new_typed(id: impl Into<SharedString>, cx: &mut impl AppContext) -> Entity<Self> {
        let model = default_selection_panel_model::<T>(id);
        cx.new(|cx| Self::from_model(model, None, cx))
    }

    /// Current wheel settings. Keyboard ownership is independent.
    pub fn scroll_interaction(&self) -> crate::interaction::ScrollInteraction {
        self.model.scroll_interaction
    }

    /// Change wheel without resetting focus, selection or position.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        cx.notify();
    }

    /// Change boundary without resetting focus, selection or position.
    pub fn set_scroll_boundary_policy(
        &mut self,
        policy: crate::interaction::ScrollBoundaryPolicy,
        cx: &mut Context<Self>,
    ) {
        self.model.scroll_interaction.boundary = policy;
        cx.notify();
    }

    /// Change focus_scope without resetting focus, selection or position.
    pub fn set_wheel_focus_scope(&mut self, policy: crate::interaction::WheelFocusScope, cx: &mut Context<Self>) {
        self.model.scroll_interaction.focus_scope = policy;
        cx.notify();
    }

    /// Change hover behavior without changing focus, selection or active item.
    pub fn set_hover_activation_policy(&mut self, policy: HoverActivationPolicy, cx: &mut Context<Self>) {
        self.model.hover_activation = Some(policy);
        cx.notify();
    }

    pub(crate) fn from_builder(builder: SelectionPanelBuilder<T>, cx: &mut Context<Self>) -> Self {
        Self::from_model(builder.model, builder.initial_active_visible_index, cx)
    }

    pub(crate) fn from_model(
        mut model: SelectionPanelModel<T>,
        initial_active_visible_index: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Self {
        if model.visible_indices.is_empty() {
            model.visible_indices = (0..model.items.len()).collect();
        }

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-scroll-surface", model.id), model.scrollbar_template.clone(), cx)
                .scrollbar_visibility(model.scrollbar_visibility)
                .scrollbar_auto_hide_activate(model.scrollbar_auto_hide_activate);
        popup_surface.set_scrolling_enabled(model.scrolling);
        popup_surface.set_snap_to_rows(true);

        let subscriptions = vec![cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| {
            if let ScrollbarEvent::Change { value } = event {
                this.popup_surface.set_vertical_offset(*value, cx);
                cx.notify();
            }
        })];

        Self {
            focus_handle: cx.focus_handle().tab_stop(model.enabled),
            model,
            state: SelectionPanelState {
                active_visible_index: initial_active_visible_index,
                hovered_visible_index: None,
            },
            pressed_visible_index: None,
            popup_surface,
            focus_in_subscription: None,
            focus_out_subscription: None,
            emitted_focused: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_panel_id(&mut self, panel_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.panel_id = panel_id.into();
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.model.visible_indices = (0..self.model.items.len()).collect();
        self.clamp_state();
        cx.notify();
    }

    pub fn set_visible_indices(&mut self, visible_indices: impl IntoIterator<Item = usize>, cx: &mut Context<Self>) {
        self.model.visible_indices = visible_indices.into_iter().collect();
        self.clamp_state();
        cx.notify();
    }

    pub fn set_selected_source_index(&mut self, selected_source_index: Option<usize>, cx: &mut Context<Self>) {
        if self.model.selected_source_index == selected_source_index {
            return;
        }

        self.model.selected_source_index = selected_source_index;
        cx.notify();
    }

    pub fn set_active_visible_index(&mut self, visible_index: Option<usize>, cx: &mut Context<Self>) {
        if self.state.set_active_visible_index(self.normalize_visible_index(visible_index)) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.model.open == open {
            return;
        }
        self.model.open = open;
        cx.emit(SelectionPanelEvent::OpenChanged { open });
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;
        if !enabled {
            self.state.set_hovered_visible_index(None);
            self.pressed_visible_index = None;
            self.emit_focus_changed(false, cx);
        }
        cx.notify();
    }

    pub fn set_show_selection_marker(&mut self, show_selection_marker: bool, cx: &mut Context<Self>) {
        if self.model.show_selection_marker == show_selection_marker {
            return;
        }
        self.model.show_selection_marker = show_selection_marker;
        cx.notify();
    }

    pub fn set_scrolling(&mut self, scrolling: bool, cx: &mut Context<Self>) {
        if self.model.scrolling == scrolling {
            return;
        }
        self.model.scrolling = scrolling;
        self.popup_surface.set_scrolling_enabled(scrolling);
        cx.notify();
    }

    pub fn set_visible_row_limits(&mut self, min_visible_rows: usize, max_visible_rows: usize, cx: &mut Context<Self>) {
        let min_visible_rows = min_visible_rows.max(1);
        let max_visible_rows = max_visible_rows.max(min_visible_rows);

        if self.model.min_visible_rows == min_visible_rows && self.model.max_visible_rows == max_visible_rows {
            return;
        }

        self.model.min_visible_rows = min_visible_rows;
        self.model.max_visible_rows = max_visible_rows;
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn SelectionPanelTemplate<T>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_item_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&super::SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.set_item_template(Some(make_selection_panel_item_template(template)), cx);
    }

    pub fn set_item_template(&mut self, item_template: Option<SelectionPanelItemTemplate<T>>, cx: &mut Context<Self>) {
        self.model.item_template = item_template;
        cx.notify();
    }

    pub fn clear_item_template(&mut self, cx: &mut Context<Self>) {
        self.set_item_template(None, cx);
    }

    pub fn set_scrollbar_template(&mut self, template: std::sync::Arc<dyn ScrollbarTemplate>, cx: &mut Context<Self>) {
        self.model.scrollbar_template = template;
        cx.notify();
    }

    pub fn set_look_provider(&mut self, provider: SelectionPanelLookProvider, cx: &mut Context<Self>) {
        self.model.look_provider = provider;
        cx.notify();
    }

    pub fn ensure_visible(&self, visible_index: usize, cx: &mut Context<Self>) {
        if self.model.visible_indices.is_empty() {
            return;
        }

        self.popup_surface
            .ensure_item_visible(visible_index.min(self.model.visible_indices.len().saturating_sub(1)), cx);
    }

    fn normalize_visible_index(&self, visible_index: Option<usize>) -> Option<usize> {
        visible_index.filter(|index| *index < self.model.visible_indices.len())
    }

    fn clamp_state(&mut self) {
        self.state.active_visible_index = self.normalize_visible_index(self.state.active_visible_index);
        self.state.hovered_visible_index = self.normalize_visible_index(self.state.hovered_visible_index);
        self.pressed_visible_index = self.normalize_visible_index(self.pressed_visible_index);

        if self.model.selected_source_index.is_some_and(|source_index| source_index >= self.model.items.len()) {
            self.model.selected_source_index = None;
        }
    }

    fn can_use_visible_index(&self, visible_index: usize) -> bool {
        let Some(source_index) = self.model.visible_indices.get(visible_index).copied() else {
            return false;
        };

        self.model.enabled
            && self.model.open
            && self.model.items.get(source_index).is_some_and(SelectionPanelItemLike::is_enabled)
    }

    fn emit_activate_for_visible_index(&mut self, visible_index: usize, cx: &mut Context<Self>) {
        let Some(source_index) = self.model.visible_indices.get(visible_index).copied() else {
            return;
        };
        let Some(item) = self.model.items.get(source_index) else {
            return;
        };

        if !self.model.enabled || !item.is_enabled() {
            return;
        }

        self.model.selected_source_index = Some(source_index);
        if self.state.set_active_visible_index(Some(visible_index)) {
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
        }

        self.popup_surface.ensure_item_visible(visible_index, cx);

        cx.emit(SelectionPanelEvent::ActivateRow { source_index, visible_index, item_id: item.id().clone() });
        cx.notify();
    }

    fn move_active(&mut self, direction: SelectionPanelStepDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.state.step(self.model.visible_indices.len(), direction) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let next = if first {
            Some(0)
        } else {
            Some(self.model.visible_indices.len() - 1)
        };

        if self.state.set_active_visible_index(next) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, visible_index: usize, hovered: bool, window: &Window, cx: &mut Context<Self>) {
        if !self.can_use_visible_index(visible_index) {
            return;
        }

        let next = if hovered { Some(visible_index) } else { None };

        if self.state.set_hovered_visible_index(next) {
            let follows = self.model.hover_activation.unwrap_or(match self.model.role {
                SelectionPanelRole::Embedded => HoverActivationPolicy::PreserveActive,
                SelectionPanelRole::Popup => HoverActivationPolicy::FollowPointer,
            }) == HoverActivationPolicy::FollowPointer;
            if hovered
                && follows
                && (self.model.hover_activation.is_some() || self.focus_handle.is_focused(window))
                && self.model.open
                && self.state.set_active_visible_index(Some(visible_index))
            {
                self.popup_surface.ensure_item_visible(visible_index, cx);
                cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            }

            cx.emit(SelectionPanelEvent::HoverChanged { visible_index: self.state.hovered_visible_index() });
            cx.notify();
        }
    }

    fn handle_item_mouse_down(
        &mut self,
        visible_index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_use_visible_index(visible_index) {
            return;
        }

        self.pressed_visible_index = Some(visible_index);
        if self.state.set_active_visible_index(Some(visible_index)) {
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
        }
        self.model.pointer_focus.apply(&self.focus_handle, window, cx);
        cx.notify();
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_visible_index.take().is_some() {
            cx.notify();
        }
    }

    fn handle_item_click(
        &mut self,
        visible_index: usize,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() {
            return;
        }

        self.emit_activate_for_visible_index(visible_index, cx);
    }

    fn handle_scroll_wheel(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        let policy = self.model.scroll_interaction;
        let bar = self.popup_surface.scrollbar().read(cx).focus_handle(cx);
        if !self.model.enabled
            || !self.model.open
            || !self.model.scrolling
            || !policy.wheel.accepts(policy.focus_scope.focused(&self.focus_handle, Some(&bar), window, cx))
        {
            return;
        }
        if crate::interaction::wheel_delta(event, px(20.0), false).is_none() {
            return;
        }
        let moved = self.popup_surface.scroll_wheel(event, cx);
        if moved {
            cx.notify();
        }
        if policy.boundary.consumes(moved) {
            cx.stop_propagation();
        }
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }
        self.emitted_focused = focused;
        cx.emit(SelectionPanelEvent::FocusChanged { focused });
        true
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        self.move_active(SelectionPanelStepDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        self.move_active(SelectionPanelStepDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        self.move_active_to_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        self.move_active_to_boundary(false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        if let Some(active_visible_index) = self.state.active_visible_index() {
            self.emit_activate_for_visible_index(active_visible_index, cx);
        }
    }

    fn handle_page_down(&mut self, _: &IncreaseValueLarge, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let page = self.popup_surface.visible_row_count().max(1);
        let current = self.state.active_visible_index().unwrap_or(0);
        let next = (current + page).min(self.model.visible_indices.len() - 1);

        if self.state.set_active_visible_index(Some(next)) {
            self.popup_surface.ensure_item_visible(next, cx);
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn handle_page_up(&mut self, _: &DecreaseValueLarge, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.open || !self.focus_handle.is_focused(window) {
            cx.propagate();
            return;
        }
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let page = self.popup_surface.visible_row_count().max(1);
        let current = self.state.active_visible_index().unwrap_or(0);
        let next = current.saturating_sub(page);

        if self.state.set_active_visible_index(Some(next)) {
            self.popup_surface.ensure_item_visible(next, cx);
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    #[allow(clippy::type_complexity)]
    fn template_handlers(
        &self,
        cx: &mut Context<Self>,
    ) -> (
        Vec<SelectionPanelHoverHandler>,
        Vec<SelectionPanelMouseDownHandler>,
        Vec<SelectionPanelMouseUpHandler>,
        Vec<SelectionPanelMouseUpHandler>,
        Vec<SelectionPanelClickHandler>,
    ) {
        let hover_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, hovered, window, cx| {
                    this.handle_item_hover(visible_index, *hovered, window, cx);
                })) as SelectionPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let mouse_down_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_mouse_down(visible_index, event, window, cx);
                })) as SelectionPanelMouseDownHandler
            })
            .collect::<Vec<_>>();

        let mouse_up_handlers = (0..self.model.visible_indices.len())
            .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as SelectionPanelMouseUpHandler)
            .collect::<Vec<_>>();

        let mouse_up_out_handlers = (0..self.model.visible_indices.len())
            .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as SelectionPanelMouseUpHandler)
            .collect::<Vec<_>>();

        let click_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(visible_index, event, window, cx);
                })) as SelectionPanelClickHandler
            })
            .collect::<Vec<_>>();

        (hover_handlers, mouse_down_handlers, mouse_up_handlers, mouse_up_out_handlers, click_handlers)
    }

    pub fn scrollbar(&self) -> Entity<crate::controls::scrollbar::Scrollbar> {
        self.popup_surface.scrollbar()
    }
}

impl SelectionPanelState {
    pub fn active_visible_index(&self) -> Option<usize> {
        self.active_visible_index
    }

    pub fn hovered_visible_index(&self) -> Option<usize> {
        self.hovered_visible_index
    }

    pub fn set_hovered_visible_index(&mut self, hovered_visible_index: Option<usize>) -> bool {
        if self.hovered_visible_index == hovered_visible_index {
            return false;
        }
        self.hovered_visible_index = hovered_visible_index;
        true
    }

    pub fn set_active_visible_index(&mut self, active_visible_index: Option<usize>) -> bool {
        if self.active_visible_index == active_visible_index {
            return false;
        }
        self.active_visible_index = active_visible_index;
        true
    }

    pub fn step(&mut self, visible_len: usize, direction: SelectionPanelStepDirection) -> bool {
        if visible_len == 0 {
            return false;
        }

        let Some(current) = self.active_visible_index else {
            return self.set_active_visible_index(Some(0));
        };

        let next = match direction {
            SelectionPanelStepDirection::Previous => current.saturating_sub(1),
            SelectionPanelStepDirection::Next => (current + 1).min(visible_len - 1),
        };

        self.set_active_visible_index(Some(next))
    }
}

impl<T> Focusable for SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.contains_focused(window, cx) {
            window.blur(cx);
        }
        self.clamp_state();
        if self.focus_in_subscription.is_none() {
            self.focus_in_subscription = Some(cx.on_focus(&self.focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            self.focus_out_subscription = Some(cx.on_focus_out(&self.focus_handle, window, Self::handle_focus_out));
        }

        let look = (self.model.look_provider)();
        let panel_min_width = look.min_width;

        let item_count = self.model.visible_indices.len();
        let min_visible_rows = self.model.min_visible_rows.max(1);
        let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
        let visible_rows = item_count.max(1).clamp(min_visible_rows, max_visible_rows) as f32;
        let viewport_height = px(look.item_height * visible_rows);
        let row_height = px(look.item_height);
        let content_top_padding = px(0.0);
        let allow_scrolling = self.model.scrolling && item_count > max_visible_rows;

        self.popup_surface.set_scrolling_enabled(allow_scrolling);
        self.popup_surface.configure(item_count.max(1), row_height, content_top_padding, viewport_height);
        self.popup_surface.sync(cx);

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let (item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks) =
            self.template_handlers(cx);

        let rows = render_selection_panel(
            self.model.template.clone(),
            &SelectionPanelRenderModel {
                panel_id: &self.model.panel_id,
                control_id: &self.model.id,
                items: &self.model.items,
                visible_indices: &self.model.visible_indices,
                selected_source_index: self.model.selected_source_index,
                active_visible_index: self.state.active_visible_index,
                hovered_visible_index: self.state.hovered_visible_index,
                pressed_visible_index: self.pressed_visible_index,
                open: self.model.open,
                enabled: self.model.enabled,
                focus,
                item_template: self.model.item_template.as_ref(),
                look: look.clone(),
                show_selection_marker: self.model.show_selection_marker,
                icons: &self.model.icons,
                show_panel_chrome: false,
            },
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
            cx,
        );

        let rows_content = if allow_scrolling {
            self.popup_surface.render_without_wheel(rows.into_any_element())
        } else {
            rows.into_any_element()
        };

        let content_width = panel_min_width
            + if allow_scrolling {
                crate::theme::stylesheet::SharedGeometry::sdk().selection_scrollbar_width
            } else {
                0.0
            };

        let panel_shell = div()
            .min_w(px(content_width))
            .p(px(look.padding))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow)
            .block_mouse_except_scroll()
            .child(rows_content);

        div()
            .id(self.model.id.clone())
            .track_focus(&self.focus_handle)
            .key_context(ControlKeyProfile::Selector.context())
            .on_scroll(cx.listener(Self::handle_scroll_wheel))
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_page_up))
            .on_action(cx.listener(Self::handle_page_down))
            .on_action(cx.listener(Self::handle_activate_control))
            .child(panel_shell)
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn wheel_policy_dispatch_matrix() {
    crate::interaction_tests::matrix(
        |policy, cx| {
            SelectionPanelBuilder::new("matrix-panel")
                .items((0..100).map(|i| SelectionPanelItem::new(i.to_string())))
                .visible_row_limits(4, 4)
                .wheel_scroll_policy(policy.wheel)
                .scroll_boundary_policy(policy.boundary)
                .spawn(cx)
        },
        |view, _cx| view.popup_surface.vertical_offset().as_f32(),
        |view, cx| {
            view.popup_surface.set_vertical_offset(100_000.0, cx);
            cx.notify();
        },
    );
}

#[cfg(all(test, feature = "test-support"))]
mod interaction_tests {
    use super::*;
    use gpui::{MouseMoveEvent, TestAppContext, point};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn scrollbar_visibility_and_timed_reveal_follow_accepted_wheel_input() {
        use crate::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarVisibility};
        use crate::controls::scrollbar::{ThemedScrollbarTemplate, default_scrollbar_theme};
        use crate::interaction::WheelScrollPolicy;
        for visibility in
            [ScrollbarVisibility::AlwaysVisible, ScrollbarVisibility::Hidden, ScrollbarVisibility::AutoHide]
        {
            for wheel in [WheelScrollPolicy::Pointer, WheelScrollPolicy::PassThrough] {
                let mut app = TestAppContext::single();
                let (panel, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    SelectionPanelControl::from_builder(
                        SelectionPanelBuilder::new("visibility-panel")
                            .items((0..20).map(|i| SelectionPanelItem::new(i.to_string())))
                            .selected_source_index(Some(0))
                            .visible_row_limits(4, 4)
                            .wheel_scroll_policy(wheel)
                            .scrollbar_visibility(visibility)
                            .scrollbar_auto_hide_activate(ScrollbarAutoHideActivate::Move)
                            .scrollbar_template(std::sync::Arc::new(
                                ThemedScrollbarTemplate::new(default_scrollbar_theme())
                                    .with_modifier(|root, _| root.debug_selector(|| "panel-scrollbar".into())),
                            )),
                        cx,
                    )
                });
                cx.run_until_parked();
                let always = visibility == ScrollbarVisibility::AlwaysVisible;
                assert_eq!(cx.debug_bounds("panel-scrollbar").is_some(), always);
                cx.simulate_event(MouseMoveEvent { position: point(px(35.0), px(55.0)), ..Default::default() });
                cx.run_until_parked();
                assert_eq!(cx.debug_bounds("panel-scrollbar").is_some(), always);
                crate::interaction_tests::wheel(cx, 0.0, -30.0);
                let accepted = wheel == WheelScrollPolicy::Pointer;
                assert_eq!(
                    cx.debug_bounds("panel-scrollbar").is_some(),
                    always || (accepted && visibility == ScrollbarVisibility::AutoHide)
                );
                cx.update(|_, app| {
                    let panel = panel.read(app);
                    assert_eq!(panel.popup_surface.vertical_offset(), px(if accepted { 30.0 } else { 0.0 }));
                    assert_eq!(panel.model.selected_source_index, Some(0));
                });
                if accepted && visibility == ScrollbarVisibility::AutoHide {
                    // Chrome deadlines use real time; the test executor separately drives the timer.
                    std::thread::sleep(std::time::Duration::from_millis(1300));
                    cx.executor().advance_clock(std::time::Duration::from_millis(1300));
                    cx.run_until_parked();
                    assert!(cx.debug_bounds("panel-scrollbar").is_none());
                }
            }
        }
    }

    #[test]
    fn embedded_hover_preserves_active_selection_position_and_focus_click_acts_once() {
        let mut app = TestAppContext::single();
        let events = Rc::new(RefCell::new(Vec::new()));
        let (panel, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            SelectionPanelControl::from_builder(
                SelectionPanelBuilder::new("hover-panel")
                    .items((0..20).map(|i| SelectionPanelItem::new(i.to_string())))
                    .active_visible_index(Some(0))
                    .selected_source_index(Some(0))
                    .visible_row_limits(4, 4),
                cx,
            )
        });
        let _subscription = cx.update(|_, app| {
            app.subscribe(&panel, {
                let events = events.clone();
                move |_, event, _| events.borrow_mut().push(event.clone())
            })
        });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: point(px(35.0), px(55.0)), ..Default::default() });
        cx.run_until_parked();
        cx.update(|window, app| {
            let panel = panel.read(app);
            assert!(!panel.focus_handle.is_focused(window));
            assert_eq!(panel.state.active_visible_index(), Some(0));
            assert_eq!(panel.model.selected_source_index, Some(0));
            assert_eq!(panel.popup_surface.vertical_offset(), px(0.0));
            assert!(panel.state.hovered_visible_index().is_some());
        });
        assert!(!events.borrow().iter().any(|e| matches!(e, SelectionPanelEvent::ActiveIndexChanged { .. })));
        cx.simulate_click(point(px(35.0), px(55.0)), Default::default());
        cx.run_until_parked();
        cx.update(|window, app| assert!(panel.read(app).focus_handle.is_focused(window)));
        assert_eq!(events.borrow().iter().filter(|e| matches!(e, SelectionPanelEvent::ActivateRow { .. })).count(), 1);
        assert_eq!(
            events
                .borrow()
                .iter()
                .filter(|e| matches!(e, SelectionPanelEvent::ActiveIndexChanged { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn role_defaults_explicit_hover_override_and_pointer_focus_override() {
        for override_policy in
            [None, Some(HoverActivationPolicy::PreserveActive), Some(HoverActivationPolicy::FollowPointer)]
        {
            let mut app = TestAppContext::single();
            let (panel, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                let mut builder = SelectionPanelBuilder::new("popup-panel")
                    .items((0..5).map(|i| SelectionPanelItem::new(i.to_string())))
                    .active_visible_index(Some(0))
                    .role(SelectionPanelRole::Popup)
                    .pointer_focus_policy(crate::interaction::PointerFocusPolicy::Preserve);
                if let Some(policy) = override_policy {
                    builder = builder.hover_activation_policy(policy);
                }
                SelectionPanelControl::from_builder(builder, cx)
            });
            cx.run_until_parked();
            cx.update(|window, app| panel.read(app).focus_handle.clone().focus(window, app));
            cx.run_until_parked();
            cx.simulate_event(MouseMoveEvent { position: point(px(35.0), px(55.0)), ..Default::default() });
            cx.run_until_parked();
            cx.update(|_, app| {
                assert_eq!(
                    panel.read(app).state.active_visible_index() != Some(0),
                    override_policy != Some(HoverActivationPolicy::PreserveActive)
                )
            });
            let elsewhere = cx.update(|window, app| {
                let handle = app.focus_handle();
                handle.focus(window, app);
                handle
            });
            cx.run_until_parked();
            cx.simulate_click(point(px(35.0), px(55.0)), Default::default());
            cx.run_until_parked();
            cx.update(|window, app| {
                assert!(!panel.read(app).focus_handle.is_focused(window));
                assert!(elsewhere.is_focused(window));
            });
        }
    }
}
