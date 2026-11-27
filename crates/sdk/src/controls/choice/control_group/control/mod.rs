use crate::infra::focus_report::ReportsFocus as _;
mod input;
mod render;
mod selection;

use std::collections::HashMap;

use gpui::{App, Bounds, Context, EventEmitter, Focusable, Pixels, ScrollHandle, SharedString, Subscription, point, px};

use super::model::{
    ControlGroupBuilder, ControlGroupFocusStrategy, ControlGroupItemLike, ControlGroupModel, ControlGroupStateMode,
    ControlSelectionMode,
};
use crate::motion::{VisualTransition};
use crate::theme::observe_theme_revision;

use self::selection::{enabled_item_index_by_id, normalize_active_id, normalize_model, selected_ids_contain};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ControlGroupEvent {
    Activate {
        activated_id: SharedString,
    },
    Change {
        changed_id: SharedString,
        selected: bool,
        selected_ids: Vec<SharedString>,
    },
    ItemBoundsChanged {
        item_id: SharedString,
        bounds: Bounds<Pixels>,
    },
    FocusChanged {
        focused: bool,
    },
    ItemFocused {
        item_id: SharedString,
    },
    /// The pointer moved onto (`hovered`) or off an item.
    ItemHoverChanged {
        item_id: SharedString,
        hovered: bool,
    },
    /// The primary button went down (`pressed`) or up on an item.
    ItemPressedChanged {
        item_id: SharedString,
        pressed: bool,
    },
}

pub struct ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    model: ControlGroupModel<T>,
    focus_handle: gpui::FocusHandle,
    focus_subscriptions: crate::infra::focus_report::FocusSubscriptions,
    item_focus_subscriptions: Vec<Subscription>,
    item_focus_subscription_keys: Vec<(SharedString, gpui::FocusHandle)>,
    focus_report: crate::infra::focus_report::FocusReport,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
    /// The items last reported hovered and pressed (see `sync_item_visual_events`).
    emitted_hovered: Option<SharedString>,
    emitted_pressed: Option<SharedString>,
    item_bounds: Vec<Option<Bounds<Pixels>>>,
    selection_transitions: HashMap<SharedString, VisualTransition>,
    scroll_handle: Option<ScrollHandle>,
    /// Maps each item's drawn state (`set_item_state_override`).
    item_state_override: Option<crate::infra::state::ItemStateOverride<SharedString>>,
}

impl<T> EventEmitter<ControlGroupEvent> for ControlGroupControl<T> where T: ControlGroupItemLike + 'static {}

impl<T> ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ControlGroupBuilder<T> {
        ControlGroupBuilder::new(id)
    }

    crate::motion_snapshot!(super::ControlGroupMotion, model.motion);

    pub(crate) fn from_builder(builder: ControlGroupBuilder<T>, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        normalize_model(&mut builder.model);
        let selection_transitions = builder
            .model
            .items
            .iter()
            .map(|item| {
                let selected = selected_ids_contain(builder.model.effective_selected_ids(), item.id());
                (
                    item.id().clone(),
                    VisualTransition::with_transition(if selected { 1.0 } else { 0.0 }, motion.selection.clone()),
                )
            })
            .collect();
        let tab_stop = builder.model.enabled && builder.model.tab_stop;
        let scrollable = builder.model.scrollable;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(tab_stop),
            focus_subscriptions: Default::default(),
            item_focus_subscriptions: Vec::new(),
            item_focus_subscription_keys: Vec::new(),
            focus_report: Default::default(),
            hovered_item: None,
            pressed_item: None,
            emitted_hovered: None,
            emitted_pressed: None,
            item_bounds: Vec::new(),
            selection_transitions,
            scroll_handle: scrollable.then(ScrollHandle::new),
            item_state_override: None,
        }
    }
    pub fn selected_ids(&self) -> &[SharedString] {
        self.model.effective_selected_ids()
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.model.effective_selected_id()
    }

    pub fn active_id(&self) -> Option<&SharedString> {
        self.model.active_id.as_ref()
    }

    pub fn selection_mode(&self) -> ControlSelectionMode {
        self.model.selection_mode
    }

    pub fn state_mode(&self) -> ControlGroupStateMode {
        self.model.state_mode
    }

    pub fn set_state_mode(&mut self, mode: ControlGroupStateMode, cx: &mut Context<Self>) {
        self.model.state_mode = mode;
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        cx.notify();
    }

    pub fn set_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        cx.notify();
    }

    pub fn set_selection_mode(&mut self, selection_mode: ControlSelectionMode, cx: &mut Context<Self>) {
        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        cx.notify();
    }

    pub fn set_layout(&mut self, layout: super::model::ControlGroupLayout, cx: &mut Context<Self>) {
        self.model.layout = layout;
        cx.notify();
    }

    pub fn scroll_vertical_by(&self, delta: Pixels, cx: &mut Context<Self>) -> bool {
        let Some(scroll_handle) = self.scroll_handle.as_ref() else {
            return false;
        };

        let current = (-scroll_handle.offset().y.as_f32()).max(0.0);
        let maximum = scroll_handle.max_offset().y.as_f32().max(0.0);
        let target = (current + delta.as_f32()).clamp(0.0, maximum);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        scroll_handle.set_offset(point(px(0.0), px(-target)));
        cx.notify();
        true
    }

    pub fn vertical_scroll_offset(&self) -> Pixels {
        self.scroll_handle
            .as_ref()
            .map_or(px(0.0), |scroll_handle| (-scroll_handle.offset().y).max(px(0.0)))
    }

    pub fn vertical_scroll_max_offset(&self) -> Pixels {
        self.scroll_handle
            .as_ref()
            .map_or(px(0.0), |scroll_handle| scroll_handle.max_offset().y.max(px(0.0)))
    }

    pub fn set_vertical_scroll_offset(&self, offset: Pixels, cx: &mut Context<Self>) {
        let Some(scroll_handle) = self.scroll_handle.as_ref() else {
            return;
        };
        let maximum = scroll_handle.max_offset().y.max(px(0.0));
        let target = offset.clamp(px(0.0), maximum);
        scroll_handle.set_offset(point(px(0.0), -target));
        cx.notify();
    }

    pub fn set_focus_strategy(&mut self, focus_strategy: ControlGroupFocusStrategy, cx: &mut Context<Self>) {
        if self.model.focus_strategy == focus_strategy {
            return;
        }
        self.model.focus_strategy = focus_strategy;
        self.clear_item_focus_subscriptions();
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_item = None;
        self.pressed_item = None;
        self.item_bounds.clear();
        self.clear_item_focus_subscriptions();
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        self.sync_item_visual_events(cx);
        cx.notify();
    }

    /// Draw each item in the state `map` gives it, from its id and the state input gives it, or as input gives it
    /// (`None`). Only the look changes; selection, input and events follow the real state (#397).
    pub fn set_item_state_override(
        &mut self,
        map: Option<crate::infra::state::ItemStateOverride<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.item_state_override = map;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled && self.model.tab_stop);
        self.clear_focus_subscriptions();
        self.clear_item_focus_subscriptions();

        if !enabled {
            self.hovered_item = None;
            self.pressed_item = None;
            self.emit_focus_changed(false, cx);
        }
        self.sync_item_visual_events(cx);

        cx.notify();
    }

    pub fn set_tab_stop(&mut self, tab_stop: bool, cx: &mut Context<Self>) {
        self.model.tab_stop = tab_stop;
        self.focus_handle = self.focus_handle.clone().tab_stop(self.model.enabled && tab_stop);
        self.clear_focus_subscriptions();
        cx.notify();
    }

    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let active_id = active_id.into();
        if enabled_item_index_by_id(&self.model.items, Some(&active_id)).is_some() {
            self.model.active_id = Some(active_id);
            cx.notify();
        }
    }

    pub fn clear_active(&mut self, cx: &mut Context<Self>) {
        self.model.active_id = None;
        normalize_active_id(&mut self.model);
        cx.notify();
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.set_managed_selected_ids(selected_ids);
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        cx.notify();
    }

    pub fn clear_managed_selected_ids(&mut self, cx: &mut Context<Self>) {
        self.model.clear_managed_selected_ids();
        normalize_model(&mut self.model);
        self.sync_selection_transitions();
        cx.notify();
    }

    pub fn motion_enabled(&self) -> bool {
        self.model.animated_selection
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.set_animated_selection(enabled, cx);
    }

    pub fn set_animated_selection(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.model.animated_selection == motion_enabled {
            return;
        }

        self.model.animated_selection = motion_enabled;
        if !motion_enabled {
            let selected_ids = self.model.effective_selected_ids();
            for (id, transition) in &mut self.selection_transitions {
                transition.snap_to(if selected_ids_contain(selected_ids, id) {
                    1.0
                } else {
                    0.0
                });
            }
        }
        cx.notify();
    }

    pub fn set_item_template(
        &mut self,
        item_template: Option<crate::controls::control_group::ControlGroupItemTemplate<T>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_template = item_template;
        cx.notify();
    }

    pub fn set_item_element_template(
        &mut self,
        item_element_template: Option<crate::controls::control_group::ControlGroupItemElementTemplate<T>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_element_template = item_element_template;
        cx.notify();
    }

    pub fn set_template(
        &mut self,
        template: crate::controls::control_group::ControlGroupTemplate<T>,
        cx: &mut Context<Self>,
    ) {
        self.model.template = template;
        cx.notify();
    }
}

impl<T> Focusable for ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> crate::infra::focus_report::ReportsFocus for ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    type FocusEvent = ControlGroupEvent;

    fn focus_report(&mut self) -> &mut crate::infra::focus_report::FocusReport {
        &mut self.focus_report
    }

    fn focus_event(focused: bool) -> ControlGroupEvent {
        ControlGroupEvent::FocusChanged { focused }
    }

    fn focus_enabled(&self) -> bool {
        self.model.enabled
    }
}
