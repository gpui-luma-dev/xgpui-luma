use crate::infra::trigger::{TriggerToggle, toggle_on_click, toggle_on_press};
use crate::infra::focus_report::ReportsFocus as _;
use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use std::sync::Arc;

use gpui::{
    App, Bounds, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, IntoElement, MouseDownEvent,
    MouseUpEvent, Pixels, Render, SharedString, Window, div, prelude::*,
};

use super::{SelectorBuilder, SelectorPlacement, SelectorRenderModel, SelectorTemplateHandlers};
use crate::infra::interaction::ControlInteraction;
use crate::controls::selector::model::{
    SelectorItemTemplate, SelectorModel, SelectorItem, SelectorItemLike, SelectorItemsTemplate,
    normalize_selector_items,
};
use crate::infra::state::ControlFocusState;
use crate::focus::EscapeFocus;
use crate::key_handling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use crate::theme::observe_theme_revision;
use crate::motion::overlay_presence::OverlayPresence;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SelectorEvent {
    Change { item_id: SharedString, label: SharedString },
    FocusChanged { focused: bool },
    OpenChanged { open: bool },
    Dismiss,
}

pub struct Selector<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    model: SelectorModel<T>,
    attachments: AttachmentHost,
    open: bool,
    presence: OverlayPresence,
    trigger_bounds: Option<Bounds<Pixels>>,
    selected_index: Option<usize>,
    active_index: Option<usize>,
    interaction: ControlInteraction,
    focus_subscriptions: crate::infra::focus_report::FocusSubscriptions,
    focus_report: crate::infra::focus_report::FocusReport,
    popup_scroll: gpui::ScrollHandle,
    popup_geometry: Option<crate::controls::selector_list::SelectorPopupGeometry>,
    initialize_popup_scroll: bool,
    popup_viewport: Option<gpui::Size<gpui::Pixels>>,
}

impl<T> EventEmitter<SelectorEvent> for Selector<T> where T: SelectorItemLike + 'static {}

impl Selector<SelectorItem> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SelectorBuilder<SelectorItem> {
        SelectorBuilder::new(id)
    }
}

impl<T> Selector<T>
where
    T: SelectorItemLike + 'static,
{
    /// Typed constructor for custom selector item models.
    #[allow(clippy::new_ret_no_self)]
    pub fn new_typed(id: impl Into<SharedString>) -> SelectorBuilder<T> {
        SelectorBuilder::new(id)
    }

    crate::presence_policy!(presence);

    crate::motion_snapshot!(super::SelectorMotion, model.motion);

    pub(crate) fn from_builder(builder: SelectorBuilder<T>, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion.enabled();
        let enabled = builder.model.enabled;
        let tab_stop = builder.model.tab_stop;
        let selected_index = builder
            .initial_selected_id
            .as_ref()
            .and_then(|selected_id| {
                builder.model.items.iter().position(|item| item.id() == selected_id && item.is_enabled())
            })
            .or_else(|| builder.model.items.iter().position(|item| item.is_selected() && item.is_enabled()));

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            attachments: AttachmentHost::with_accessible_role(gpui::Role::ComboBox),
            open: false,
            presence: OverlayPresence::with_motion(false, motion_enabled, motion.presence.clone()),
            trigger_bounds: None,
            selected_index,
            active_index: None,
            interaction: ControlInteraction::new_with_tab_stop(enabled, tab_stop, cx),
            focus_subscriptions: Default::default(),
            focus_report: Default::default(),
            popup_scroll: gpui::ScrollHandle::new(),
            popup_geometry: None,
            initialize_popup_scroll: false,
            popup_viewport: None,
        }
    }

    /// Current independent popup wheel settings.
    pub fn scroll_interaction(&self) -> crate::interaction::ScrollInteraction {
        self.model.scroll_interaction
    }

    /// Update wheel eligibility without rebuilding the popup or changing its selection/offset.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        cx.notify();
    }

    /// Update boundary behavior independently of wheel eligibility.
    pub fn set_scroll_boundary_policy(
        &mut self,
        policy: crate::interaction::ScrollBoundaryPolicy,
        cx: &mut Context<Self>,
    ) {
        self.model.scroll_interaction.boundary = policy;
        cx.notify();
    }

    /// Update which real focus owners qualify, without transferring focus.
    pub fn set_wheel_focus_scope(&mut self, policy: crate::interaction::WheelFocusScope, cx: &mut Context<Self>) {
        self.model.scroll_interaction.focus_scope = policy;
        cx.notify();
    }

    fn handle_popup_scroll_wheel(&mut self, event: &gpui::ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        let policy = self.model.scroll_interaction;
        if !self.model.enabled
            || !self.open
            || !policy.wheel.accepts(policy.focus_scope.focused(self.interaction.focus_handle(), None, window, cx))
        {
            return;
        }
        let Some(delta) = crate::interaction::wheel_delta(event, window.line_height(), false) else {
            return;
        };
        // Popup padding contains applicable input by default, but only its row viewport moves.
        let moved = self.popup_scroll.bounds().contains(&event.position)
            && crate::interaction::scroll_handle_by(&self.popup_scroll, delta, false);
        if moved {
            cx.notify();
        }
        if policy.boundary.consumes(moved) {
            cx.stop_propagation();
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        let previous_selection = self.selected_id().cloned();
        self.model.items = normalize_selector_items(items);
        self.selected_index = previous_selection.as_ref().and_then(|id| self.index_by_id(id));
        self.close_menu();
        cx.notify();
    }

    /// Whether the control takes input.
    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        self.focus_subscriptions.reset();
        if !enabled {
            self.close_menu();
            self.emit_focus_changed(false, cx);
        }
        cx.notify();
    }

    /// Closes the popup and emits the normal dismissal events when it was open.
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if self.close_menu_with_event(true, cx) {
            cx.notify();
        }
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.model.invalid != invalid {
            self.model.invalid = invalid;
            cx.notify();
        }
    }

    pub fn set_tab_stop(&mut self, tab_stop: bool, cx: &mut Context<Self>) {
        self.model.tab_stop = tab_stop;
        self.interaction.set_tab_stop(self.model.enabled, tab_stop);
        self.focus_subscriptions.reset();
        cx.notify();
    }

    /// Choose how the popup opens. Takes effect on the next opening.
    pub fn set_opening_mode(&mut self, mode: super::SelectorOpeningMode, cx: &mut Context<Self>) {
        self.model.opening_mode = mode;
        self.popup_geometry = None;
        self.initialize_popup_scroll = true;
        cx.notify();
    }

    pub fn set_placement(&mut self, placement: SelectorPlacement, cx: &mut Context<Self>) {
        self.model.placement = placement;
        cx.notify();
    }

    pub fn set_item_template(&mut self, template: Option<SelectorItemTemplate<T>>, cx: &mut Context<Self>) {
        self.model.item_template = template;
        cx.notify();
    }

    pub fn set_panel_template(
        &mut self,
        template: std::sync::Arc<dyn SelectorItemsTemplate<T>>,
        cx: &mut Context<Self>,
    ) {
        self.model.panel_template = template;
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::SelectorTemplate<T>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.selected_index.and_then(|index| self.model.items.get(index)).map(SelectorItemLike::id)
    }

    pub fn set_selected_id(&mut self, item_id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        let item_id = item_id.into();
        let changed = self.select_by_id(&item_id, false, cx);
        if changed {
            cx.notify();
        }
        changed
    }

    fn render_model<'a>(&'a self, window: &Window) -> SelectorRenderModel<'a, T> {
        let mut interaction = self.interaction.render_state(self.model.enabled, window);
        interaction.invalid = self.model.invalid;
        SelectorRenderModel {
            id: &self.model.id,
            label: self.trigger_label(),
            selected_index: self.selected_index,
            items: &self.model.items,
            open: self.open,
            presence: self.presence.clone(),
            popup_scroll: Some(&self.popup_scroll),
            trigger_bounds: self.trigger_bounds,
            placement: self.model.placement,
            opening_mode: self.model.opening_mode,
            popup_geometry: self.popup_geometry,
            initialize_popup_scroll: self.initialize_popup_scroll,
            active_path: self.active_index.map(crate::controls::selector_list::SelectorPath::Item),
            enabled: self.model.enabled,
            trigger_style: self.model.trigger_style,
            icons: &self.model.icons,
            without_elevation: self.model.without_elevation,
            item_template: self.model.item_template.as_ref(),
            panel_template: Some(self.model.panel_template.as_ref()),
            focus: ControlFocusState::from_focus_handle(self.model.enabled, self.interaction.focus_handle(), window),
            state: interaction,
            visual_state: crate::controls::selector::SelectorVisualState {
                interaction,
                open: self.open,
                selected: self.selected_index.is_some(),
                invalid: self.model.invalid,
            },
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> SelectorTemplateHandlers {
        let entity = cx.entity();
        let hover_entity = entity.clone();
        let mouse_down_entity = entity.clone();
        let click_entity = entity.clone();

        SelectorTemplateHandlers {
            popup_scroll_wheel: Some(Box::new(cx.listener(Self::handle_popup_scroll_wheel))),
            popup_geometry: Some(Box::new(cx.listener(Self::handle_popup_geometry))),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_hover: Box::new(cx.listener(Self::handle_hover)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            trigger_mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            trigger_mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            root_mouse_down_out: Box::new(cx.listener(Self::handle_mouse_down_out)),
            on_item_hover: Arc::new(move |model_index, hovered, _window, app| {
                hover_entity.update(app, |this, cx| {
                    this.handle_item_hover(model_index, *hovered, cx);
                });
            }),
            on_item_mouse_down: Arc::new(move |model_index, _event, _window, app| {
                mouse_down_entity.update(app, |this, cx| {
                    this.handle_item_mouse_down(model_index, cx);
                });
            }),
            on_item_click: Arc::new(move |model_index, event, _window, app| {
                click_entity.update(app, |this, cx| {
                    this.handle_item_click(model_index, event, cx);
                });
            }),
        }
    }

    fn trigger_label(&self) -> &SharedString {
        selected_trigger_label(&self.model.label, &self.model.items, self.selected_index)
    }

    fn index_by_id(&self, item_id: &SharedString) -> Option<usize> {
        self.model.items.iter().position(|item| item.id() == item_id && self.is_selectable_item(item))
    }

    fn is_selectable_item(&self, item: &T) -> bool {
        item.is_enabled()
    }

    fn selectable_indices(&self) -> Vec<usize> {
        self.model
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| self.is_selectable_item(item).then_some(index))
            .collect()
    }

    fn first_selectable_index(&self) -> Option<usize> {
        self.model.items.iter().position(|item| self.is_selectable_item(item))
    }

    fn last_selectable_index(&self) -> Option<usize> {
        self.model.items.iter().rposition(|item| self.is_selectable_item(item))
    }

    fn step_selectable_index(&self, current: Option<usize>, forward: bool) -> Option<usize> {
        let indices = self.selectable_indices();
        if indices.is_empty() {
            return None;
        }

        match current.and_then(|current| indices.iter().position(|index| *index == current)) {
            Some(position) => {
                let next = if forward {
                    (position + 1) % indices.len()
                } else if position == 0 {
                    indices.len() - 1
                } else {
                    position - 1
                };
                Some(indices[next])
            }
            None => {
                if forward {
                    indices.first().copied()
                } else {
                    indices.last().copied()
                }
            }
        }
    }

    fn handle_popup_geometry(
        &mut self,
        geometry: &crate::controls::selector_list::SelectorPopupGeometry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.open || self.model.opening_mode != super::SelectorOpeningMode::Centered {
            return;
        }
        if self.popup_geometry != Some(*geometry) || self.popup_viewport != Some(window.viewport_size()) {
            self.popup_geometry = Some(*geometry);
            self.popup_viewport = Some(window.viewport_size());
            self.initialize_popup_scroll = true;
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        } else {
            self.initialize_popup_scroll = false;
        }
    }

    fn close_menu(&mut self) {
        self.open = false;
        // The native popup viewport previously discarded its offset when unmounted.
        self.popup_scroll.set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
        self.presence.set_open(false);
        self.active_index = None;
    }

    fn close_menu_with_event(&mut self, dismiss: bool, cx: &mut Context<Self>) -> bool {
        let was_open = self.open;
        self.close_menu();
        if was_open {
            cx.emit(SelectorEvent::OpenChanged { open: false });
            if dismiss {
                cx.emit(SelectorEvent::Dismiss);
            }
        }
        was_open
    }

    fn open_menu_with_active(&mut self, active_index: Option<usize>, cx: &mut Context<Self>) -> bool {
        let next_active =
            active_index.filter(|index| self.model.items.get(*index).is_some_and(|item| self.is_selectable_item(item)));
        let changed = !self.open || self.active_index != next_active;
        let open_changed = !self.open;
        self.open = true;
        self.presence.set_open(true);
        self.active_index = next_active;
        if open_changed {
            self.popup_geometry = None;
            self.initialize_popup_scroll = true;
            cx.emit(SelectorEvent::OpenChanged { open: true });
        }
        changed
    }

    fn open_with_default_active(&mut self, cx: &mut Context<Self>) -> bool {
        let active = self
            .selected_index
            .filter(|index| self.model.items.get(*index).is_some_and(|item| self.is_selectable_item(item)))
            .or_else(|| self.first_selectable_index());
        self.open_menu_with_active(active, cx)
    }

    fn select_index(&mut self, index: usize, emit: bool, cx: &mut Context<Self>) -> bool {
        let Some(item) = self.model.items.get(index) else {
            return false;
        };
        if !self.is_selectable_item(item) {
            return false;
        }

        let item_id = item.id().clone();
        let label = item.label().clone();
        let changed = self.selected_index != Some(index);
        self.selected_index = Some(index);
        let closed = if emit {
            self.close_menu_with_event(false, cx)
        } else {
            self.close_menu();
            false
        };

        if emit && changed {
            cx.emit(SelectorEvent::Change { item_id, label });
        }

        changed || closed
    }

    fn select_by_id(&mut self, item_id: &SharedString, emit: bool, cx: &mut Context<Self>) -> bool {
        let Some(index) = self.index_by_id(item_id) else {
            return false;
        };

        self.select_index(index, emit, cx)
    }

    fn activate_active_item(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if !self.open {
            if self.open_with_default_active(cx) {
                cx.notify();
            }
            return;
        }

        if let Some(active_index) = self.active_index
            && self.select_index(active_index, true, cx)
        {
            cx.notify();
        }
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(*bounds);
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let toggle = toggle_on_click(self.model.open_on, event, self.model.enabled, self.open);
        self.apply_trigger_toggle(toggle, cx);
    }

    fn apply_trigger_toggle(&mut self, toggle: Option<TriggerToggle>, cx: &mut Context<Self>) {
        match toggle {
            Some(TriggerToggle::Open) => {
                self.open_with_default_active(cx);
            }
            Some(TriggerToggle::Close) => {
                self.close_menu_with_event(true, cx);
            }
            None => return,
        }
        cx.notify();
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.select_index(index, true, cx) {
            cx.notify();
        }
    }

    fn handle_item_mouse_down(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.select_index(index, true, cx) {
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !hovered || !self.open || !self.model.items.get(index).is_some_and(|item| self.is_selectable_item(item)) {
            return;
        }

        if self.active_index != Some(index) {
            self.active_index = Some(index);
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_down(self.model.enabled, window, cx) {
            cx.notify();
        }
        let toggle = toggle_on_press(self.model.open_on, self.model.enabled, self.open);
        self.apply_trigger_toggle(toggle, cx);
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_mouse_down_out(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open && self.model.dismiss.click_outside {
            self.close_menu_with_event(true, cx);
            cx.notify();
        }
    }

    fn scroll_active_into_view(&self) {
        if self.model.opening_mode == super::SelectorOpeningMode::Centered
            && let Some(index) = self.active_index
        {
            self.popup_scroll.scroll_to_item(index);
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.open {
            let next = self.step_selectable_index(self.active_index, false);
            if self.active_index != next {
                self.active_index = next;
                self.scroll_active_into_view();
                cx.notify();
            }
        } else if self.model.tab_stop && self.open_menu_with_active(self.last_selectable_index(), cx) {
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.open {
            let next = self.step_selectable_index(self.active_index, true);
            if self.active_index != next {
                self.active_index = next;
                self.scroll_active_into_view();
                cx.notify();
            }
        } else if self.model.tab_stop && self.open_menu_with_active(self.first_selectable_index(), cx) {
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            let next = self.first_selectable_index();
            if self.active_index != next {
                self.active_index = next;
                self.scroll_active_into_view();
                cx.notify();
            }
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            let next = self.last_selectable_index();
            if self.active_index != next {
                self.active_index = next;
                self.scroll_active_into_view();
                cx.notify();
            }
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_active_item(cx);
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open && self.model.dismiss.escape {
            self.close_menu_with_event(true, cx);
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let mut changed = self.emit_focus_changed(false, cx);
        if self.open && self.model.dismiss.focus_loss {
            changed |= self.close_menu_with_event(true, cx);
        }
        if changed {
            cx.notify();
        }
    }
}

impl<T> Focusable for Selector<T>
where
    T: SelectorItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl<T> Render for Selector<T>
where
    T: SelectorItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.presence.sync();
        self.presence.schedule_frame(window, cx);
        let focus_handle = self.interaction.focus_handle().clone();
        self.focus_subscriptions
            .ensure(&focus_handle, window, cx, Self::handle_focus_in, Self::handle_focus_out);

        let mut tip_state = self.interaction.render_state(self.model.enabled, window);
        tip_state.disabled |= self.open;
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);
        let root = self.model.template.render(&model, handlers, window, cx);
        let root = self.attachments.render(root, tip_state, cx);

        div()
            .child(
                root.track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::Selector.context())
                    .on_action(cx.listener(Self::handle_escape_focus))
                    .on_action(cx.listener(Self::handle_select_previous_item))
                    .on_action(cx.listener(Self::handle_select_next_item))
                    .on_action(cx.listener(Self::handle_select_first_item))
                    .on_action(cx.listener(Self::handle_select_last_item))
                    .on_action(cx.listener(Self::handle_activate_control)),
            )
            .into_any_element()
    }
}

fn selected_trigger_label<'a, T>(
    fallback_label: &'a SharedString,
    items: &'a [T],
    selected_index: Option<usize>,
) -> &'a SharedString
where
    T: SelectorItemLike + 'static,
{
    selected_index.and_then(|index| items.get(index)).map_or(fallback_label, SelectorItemLike::label)
}

impl<T: SelectorItemLike + 'static> AttachmentTarget for Selector<T> {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
    fn attachment_anchor(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        self.trigger_bounds.unwrap_or(bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<SelectorItem> {
        vec![
            SelectorItem::new("first").label("First"),
            SelectorItem::new("second").label("Second"),
            SelectorItem::new("third").label("Third"),
        ]
    }

    #[test]
    fn normalize_preserves_disabled_items() {
        let filtered = normalize_selector_items(vec![
            SelectorItem::new("first").label("First"),
            SelectorItem::new("disabled").label("Disabled").enabled(false),
            SelectorItem::new("last").label("Last"),
        ]);

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].id().as_ref(), "first");
        assert_eq!(filtered[1].id().as_ref(), "disabled");
        assert_eq!(filtered[2].id().as_ref(), "last");
        assert!(!filtered[1].is_enabled());
    }

    #[test]
    fn builder_stores_initial_selected_id() {
        let builder = Selector::new("selector").items(items()).selected_id("second");

        assert_eq!(builder.initial_selected_id.as_ref().map(|id| id.as_ref()), Some("second"));
    }

    #[test]
    fn builder_leaves_selector_unselected_without_a_valid_selection() {
        let builder = Selector::new("selector").items(items());

        assert!(builder.initial_selected_id.is_none());
        assert!(builder.model.items.iter().all(|item| !item.is_selected()));
    }

    #[test]
    fn item_template_is_installed() {
        let builder = Selector::new("selector")
            .items(items())
            .with_item_template(|item, _cx| gpui::div().child(item.item.id().clone()));

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn selected_trigger_prefers_item_label_over_id() {
        let fallback = SharedString::from("Theme");
        let items = vec![SelectorItem::new("retro-arcade").label("Retro Arcade")];

        let label = selected_trigger_label(&fallback, &items, Some(0));
        assert_eq!(label.as_ref(), "Retro Arcade");
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "wheel_tests.rs"]
mod wheel_tests;

impl<T> crate::infra::focus_report::ReportsFocus for Selector<T>
where
    T: SelectorItemLike + 'static,
{
    type FocusEvent = SelectorEvent;

    fn focus_report(&mut self) -> &mut crate::infra::focus_report::FocusReport {
        &mut self.focus_report
    }

    fn focus_event(focused: bool) -> SelectorEvent {
        SelectorEvent::FocusChanged { focused }
    }

    fn focus_enabled(&self) -> bool {
        self.model.enabled
    }
}
