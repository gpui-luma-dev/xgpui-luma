use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    App, Context, EventEmitter, FocusOutEvent, Focusable, FocusHandle, IntoElement, KeyDownEvent, Render, SharedString,
    Subscription, Window, div, prelude::*,
};

use super::{
    AccordionBuilder, AccordionItem, AccordionItemRenderModel, AccordionModel, AccordionRenderModel,
    AccordionSelectionMode, AccordionTemplateHandlers,
};
use crate::motion::DisclosureMotion;
use crate::infra::state::{CompositeItemState, ControlFocusState};
use crate::key_handling::{ActivateControl, SelectFirstItem, SelectLastItem};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AccordionEvent {
    DragDrop(super::AccordionDragEvent),
    ExpandedChanged { item_id: SharedString, expanded: bool },
    FocusChanged { focused: bool },
    ItemFocused { item_id: SharedString },
    ItemHoverChanged { item_id: SharedString, hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccordionDirection {
    Previous,
    Next,
}

#[derive(Clone, Copy)]
enum AccordionKeyAction {
    Previous,
    Next,
    First,
    Last,
    Activate,
}

pub struct AccordionControl {
    pub(super) hovered_drag_handle: Option<SharedString>,
    pub(super) drag_tooltips: std::collections::HashMap<SharedString, crate::infra::attachments::TooltipHandle>,
    pub(super) active_drag: Option<super::drag::AccordionDrag>,
    pub(super) drop_handler: Option<super::drag::DropHandler>,
    pub(super) drop_layout: crate::motion::DropLayoutMotion<SharedString>,
    pub(super) item_bounds: super::geometry::ItemBounds,
    pub(super) external_scroll: bool,
    pub(super) drag_scroll: gpui::ScrollHandle,
    pub(super) drag_scroll_pending: std::rc::Rc<std::cell::Cell<bool>>,
    pub(super) model: AccordionModel,
    focus_handle: FocusHandle,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    expanded_ids: HashSet<SharedString>,
    pub(super) transitions: Vec<DisclosureMotion>,
    pub(super) content_heights_px: Vec<f32>,
    pub(super) focused_item_index: Option<usize>,
    pub(super) hovered_item_index: Option<usize>,
    pub(super) pressed_item_index: Option<usize>,
    emitted_focused: bool,
}

impl EventEmitter<AccordionEvent> for AccordionControl {}

impl AccordionControl {
    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn motion_enabled(&self) -> bool {
        self.model.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.motion_enabled = enabled;
        for transition in &mut self.transitions {
            transition.set_motion_enabled(enabled);
        }
        if !enabled {
            self.drop_layout.cancel();
        }
        cx.notify();
    }
    crate::motion_snapshot!(super::AccordionMotion, model.motion);

    pub(crate) fn from_builder(builder: AccordionBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let enabled = builder.model.enabled;
        let mut expanded_ids = HashSet::new();

        for item in &builder.model.items {
            if item.initially_expanded {
                expanded_ids.insert(item.id.clone());
                if builder.model.selection_mode == AccordionSelectionMode::Single {
                    break;
                }
            }
        }

        let transitions = builder
            .model
            .items
            .iter()
            .map(|item| {
                let initial = if expanded_ids.contains(&item.id) { 1.0 } else { 0.0 };
                DisclosureMotion::with_transition(initial, builder.model.motion_enabled, motion.expansion.clone())
            })
            .collect();
        let content_heights_px = vec![0.0; builder.model.items.len()];

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            hovered_drag_handle: None,
            drag_tooltips: Default::default(),
            external_scroll: false,
            active_drag: None,
            drop_handler: None,
            drop_layout: Default::default(),
            item_bounds: Default::default(),
            drag_scroll: Default::default(),
            drag_scroll_pending: Default::default(),
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            focus_in_subscription: None,
            focus_out_subscription: None,
            expanded_ids,
            transitions,
            content_heights_px,
            focused_item_index: None,
            hovered_item_index: None,
            pressed_item_index: None,
            emitted_focused: false,
        }
    }

    pub fn is_expanded(&self, item_id: &SharedString) -> bool {
        self.expanded_ids.contains(item_id)
    }

    pub fn set_template(&mut self, template: Arc<dyn super::AccordionTemplate>, cx: &mut Context<Self>) {
        self.drop_layout.cancel();
        self.drag_tooltips.clear();
        self.hovered_drag_handle = None;
        self.model.template = template;
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
            self.drag_tooltips.clear();
            self.hovered_drag_handle = None;
            self.cancel_drag(cx);
            self.drop_layout.cancel();
            if let Some(index) = self.hovered_item_index {
                cx.emit(AccordionEvent::ItemHoverChanged {
                    item_id: self.model.items[index].id.clone(),
                    hovered: false,
                });
            }
            self.focused_item_index = None;
            self.hovered_item_index = None;
            self.pressed_item_index = None;
            self.emit_focus_changed(false, cx);
        }
        cx.emit(AccordionEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn toggle_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.model.enabled || index >= self.model.items.len() {
            return;
        }

        let item = &self.model.items[index];
        if !item.enabled {
            return;
        }

        self.drop_layout.cancel();
        let id = item.id.clone();
        for event in apply_toggle(&mut self.expanded_ids, self.model.selection_mode, self.model.collapsible, id) {
            if let AccordionEvent::ExpandedChanged { item_id, expanded } = &event
                && let Some(item_index) = self.model.items.iter().position(|item| item.id == *item_id)
                && let Some(transition) = self.transitions.get_mut(item_index)
            {
                transition.set_target(if *expanded { 1.0 } else { 0.0 });
            }
            cx.emit(event);
        }

        cx.notify();
    }

    pub(crate) fn set_content_height(&mut self, index: usize, height: f32, cx: &mut Context<Self>) {
        let Some(slot) = self.content_heights_px.get_mut(index) else {
            return;
        };
        let height = height.max(0.0);
        let settled_open = self
            .transitions
            .get(index)
            .is_some_and(|transition| !transition.is_animating() && transition.progress() >= 1.0 - f32::EPSILON);

        if settled_open {
            // Fully open: accept the natural height (corrects oversized caches).
            if (height - *slot).abs() <= 0.5 {
                return;
            }
        } else if height <= *slot + 0.5 {
            // Mid-animation: only grow so clipped frames cannot shrink the cache.
            return;
        }

        *slot = height;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> AccordionRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);

        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let item_enabled = self.model.enabled && item.enabled;
                let expanded = self.expanded_ids.contains(&item.id);
                let has_keyboard_focus = focus.focused && self.focused_item_index == Some(index);
                let progress = self
                    .transitions
                    .get(index)
                    .map(|transition| transition.progress())
                    .unwrap_or_else(|| if expanded { 1.0 } else { 0.0 });
                let content_height_px = self.content_heights_px.get(index).copied().unwrap_or(0.0);

                AccordionItemRenderModel {
                    id: &item.id,
                    trigger: &item.trigger,
                    content: &item.content,
                    expanded,
                    progress,
                    content_height_px,
                    enabled: item_enabled,
                    state: CompositeItemState {
                        hovered: item_enabled && self.hovered_item_index == Some(index),
                        pressed: item_enabled && self.pressed_item_index == Some(index),
                        disabled: !item_enabled,
                        selected: expanded,
                        active: item_enabled && has_keyboard_focus,
                        focus_visible: item_enabled && focus.focus_visible && has_keyboard_focus,
                    },
                }
            })
            .collect();

        AccordionRenderModel {
            id: &self.model.id,
            items,
            selection_mode: self.model.selection_mode,
            collapsible: self.model.collapsible,
            enabled: self.model.enabled,
            item_dividers: self.model.item_dividers,
            content_padding_y: self.model.content_padding_y,
            content_padding_top: self.model.content_padding_top,
            content_padding_bottom: self.model.content_padding_bottom,
            trigger_min_height: self.model.trigger_min_height,
            trigger_padding_y: self.model.trigger_padding_y,
            disclosure_icons: &self.model.disclosure_icons,
            focus,
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.items.get(index).is_some_and(|item| item.enabled) {
            return;
        }

        if hovered {
            if self.hovered_item_index != Some(index) {
                self.hovered_item_index = Some(index);
                cx.emit(AccordionEvent::ItemHoverChanged {
                    item_id: self.model.items[index].id.clone(),
                    hovered: true,
                });
                cx.notify();
            }
        } else if self.hovered_item_index == Some(index) {
            self.hovered_item_index = None;
            if self.pressed_item_index == Some(index) {
                self.pressed_item_index = None;
            }
            cx.emit(AccordionEvent::ItemHoverChanged { item_id: self.model.items[index].id.clone(), hovered: false });
            cx.notify();
        }
    }

    fn template_handlers(&mut self, cx: &mut Context<Self>) -> AccordionTemplateHandlers {
        AccordionTemplateHandlers {
            drag: self.drag_bindings(cx),
            trigger_hovers: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, hovered, _, cx| {
                        this.handle_item_hover(idx, *hovered, cx);
                    })) as _
                })
                .collect(),
            trigger_mouse_downs: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, window, cx| {
                        if this.model.enabled && this.model.items[idx].enabled {
                            this.pressed_item_index = Some(idx);
                            this.set_focused_item_index(Some(idx), cx);
                            this.focus_handle.focus(window, cx);
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_mouse_ups: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        if this.pressed_item_index == Some(idx) {
                            this.pressed_item_index = None;
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_clicks: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        this.toggle_item(idx, cx);
                    })) as _
                })
                .collect(),
            content_height_reports: (0..self.model.items.len())
                .map(|idx| {
                    let entity = cx.entity().clone();
                    Box::new(move |height: f32, _: &mut Window, cx: &mut App| {
                        // Heights are measured while the frame is drawn, and a notify then does not
                        // schedule a redraw; apply the height after the frame so the content's new
                        // height (e.g. after rows were filtered out) shows without waiting for input.
                        let entity = entity.clone();
                        cx.defer(move |cx| {
                            entity.update(cx, |this, cx| this.set_content_height(idx, height, cx));
                        });
                    }) as _
                })
                .collect(),
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let shortcut = modifiers.platform || modifiers.control;

        let action = match (key, shortcut) {
            ("left", false) => Some(AccordionKeyAction::Previous),
            ("right", false) => Some(AccordionKeyAction::Next),
            ("up", false) => Some(AccordionKeyAction::Previous),
            ("down", false) => Some(AccordionKeyAction::Next),
            ("left", true) | ("a", true) => Some(AccordionKeyAction::First),
            ("right", true) | ("e", true) => Some(AccordionKeyAction::Last),
            ("home", false) => Some(AccordionKeyAction::First),
            ("end", false) => Some(AccordionKeyAction::Last),
            ("enter", false) | ("space", false) => Some(AccordionKeyAction::Activate),
            _ => None,
        };

        let Some(action) = action else {
            return;
        };

        match action {
            AccordionKeyAction::Previous => self.move_focus(AccordionDirection::Previous, cx),
            AccordionKeyAction::Next => self.move_focus(AccordionDirection::Next, cx),
            AccordionKeyAction::First => self.handle_first(&crate::key_handling::SelectFirstItem, window, cx),
            AccordionKeyAction::Last => self.handle_last(&crate::key_handling::SelectLastItem, window, cx),
            AccordionKeyAction::Activate => self.handle_activate(&ActivateControl, window, cx),
        }
        window.prevent_default();
        cx.stop_propagation();
    }

    fn move_focus(&mut self, direction: AccordionDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(next) = next_enabled_index(&self.model.items, self.focused_item_index, direction) {
            self.set_focused_item_index(Some(next), cx);
            cx.notify();
        }
    }

    fn handle_first(&mut self, _: &SelectFirstItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = first_enabled_index(&self.model.items) {
            self.set_focused_item_index(Some(idx), cx);
            cx.notify();
        }
    }

    fn handle_last(&mut self, _: &SelectLastItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = last_enabled_index(&self.model.items) {
            self.set_focused_item_index(Some(idx), cx);
            cx.notify();
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.focused_item_index {
            self.toggle_item(idx, cx);
        }
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

    pub(super) fn set_focused_item_index(&mut self, focused_item_index: Option<usize>, cx: &mut Context<Self>) {
        if self.focused_item_index == focused_item_index {
            return;
        }

        self.focused_item_index = focused_item_index;
        if let Some(index) = focused_item_index {
            cx.emit(AccordionEvent::ItemFocused { item_id: self.model.items[index].id.clone() });
        }
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(AccordionEvent::FocusChanged { focused });
        true
    }
}

impl Focusable for AccordionControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AccordionControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut was_animating = false;
        let mut is_animating = false;
        for transition in &mut self.transitions {
            was_animating |= transition.is_animating();
            let animating = transition.sync();
            is_animating |= animating || transition.is_animating();
            transition.schedule_frame(window, cx);
        }
        if was_animating || is_animating {
            cx.notify();
        }

        if self.focus_in_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let handlers = self.template_handlers(cx);
        let model = self.render_model(window);

        let root = div().w_full();
        let root = if self.model.reordering && !self.external_scroll {
            root.flex_1().min_h(gpui::px(0.)).flex().flex_col()
        } else {
            root
        };
        root.child(
            self.model
                .template
                .render(&model, handlers, window, cx)
                .key_context(crate::key_handling::ControlKeyProfile::Navigation.context())
                .track_focus(&self.focus_handle)
                .on_key_down(cx.listener(Self::handle_key_down)),
        )
        .into_any_element()
    }
}

pub(crate) fn first_enabled_index(items: &[AccordionItem]) -> Option<usize> {
    items.iter().position(|item| item.enabled)
}

pub(crate) fn last_enabled_index(items: &[AccordionItem]) -> Option<usize> {
    items.iter().rposition(|item| item.enabled)
}

pub(crate) fn next_enabled_index(
    items: &[AccordionItem],
    current_index: Option<usize>,
    direction: AccordionDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        AccordionDirection::Previous => len - 1,
        AccordionDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(idx) => (idx + step) % len,
        None if direction == AccordionDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].enabled {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

pub(crate) fn apply_toggle(
    expanded_ids: &mut HashSet<SharedString>,
    selection_mode: AccordionSelectionMode,
    collapsible: bool,
    item_id: SharedString,
) -> Vec<AccordionEvent> {
    let was_expanded = expanded_ids.contains(&item_id);
    let mut events = Vec::new();

    match selection_mode {
        AccordionSelectionMode::Single => {
            if was_expanded {
                if collapsible {
                    expanded_ids.remove(&item_id);
                    events.push(AccordionEvent::ExpandedChanged { item_id, expanded: false });
                }
            } else {
                for prev_id in expanded_ids.drain() {
                    events.push(AccordionEvent::ExpandedChanged { item_id: prev_id, expanded: false });
                }
                expanded_ids.insert(item_id.clone());
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: true });
            }
        }
        AccordionSelectionMode::Multiple => {
            if was_expanded {
                expanded_ids.remove(&item_id);
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: false });
            } else {
                expanded_ids.insert(item_id.clone());
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: true });
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::{
        AccordionDirection, AccordionSelectionMode, apply_toggle, first_enabled_index, last_enabled_index,
        next_enabled_index,
    };
    use crate::controls::accordion::{AccordionContent, AccordionItem, AccordionTrigger};
    use gpui::{SharedString, div, prelude::*};

    fn sample_items() -> Vec<AccordionItem> {
        vec![
            AccordionItem::new(
                "one",
                AccordionTrigger::new("One"),
                AccordionContent::custom(|_, _| div().child("Content one").into_any_element()),
            ),
            AccordionItem::new(
                "two",
                AccordionTrigger::new("Two"),
                AccordionContent::custom(|_, _| div().child("Content two").into_any_element()),
            )
            .enabled(false),
            AccordionItem::new(
                "three",
                AccordionTrigger::new("Three"),
                AccordionContent::custom(|_, _| div().child("Content three").into_any_element()),
            ),
        ]
    }

    #[test]
    fn navigation_skips_disabled_items_and_wraps() {
        let items = sample_items();

        assert_eq!(next_enabled_index(&items, Some(0), AccordionDirection::Next), Some(2));
        assert_eq!(next_enabled_index(&items, Some(2), AccordionDirection::Next), Some(0));
        assert_eq!(next_enabled_index(&items, Some(0), AccordionDirection::Previous), Some(2));
    }

    #[test]
    fn first_and_last_enabled_index_skip_disabled_items() {
        let items = sample_items();

        assert_eq!(first_enabled_index(&items), Some(0));
        assert_eq!(last_enabled_index(&items), Some(2));
    }

    #[test]
    fn single_mode_collapses_other_items_when_expanding() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Single, true, SharedString::from("three"));

        assert_eq!(expanded.len(), 1);
        assert!(expanded.contains(&SharedString::from("three")));
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|event| matches!(event, super::AccordionEvent::ExpandedChanged { item_id, expanded: false } if item_id.as_ref() == "one")));
        assert!(events.iter().any(|event| matches!(event, super::AccordionEvent::ExpandedChanged { item_id, expanded: true } if item_id.as_ref() == "three")));
    }

    #[test]
    fn single_mode_respects_non_collapsible() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Single, false, SharedString::from("one"));

        assert_eq!(expanded.len(), 1);
        assert!(events.is_empty());
    }

    #[test]
    fn multiple_mode_toggles_independently() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Multiple, true, SharedString::from("three"));

        assert_eq!(expanded.len(), 2);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::AccordionEvent::ExpandedChanged { expanded: true, .. }));
    }

    /// Mirrors [`AccordionControl::handle_item_hover`] index bookkeeping.
    fn apply_accordion_item_hover(hovered_item_index: &mut Option<usize>, index: usize, hovered: bool) {
        if hovered {
            if *hovered_item_index != Some(index) {
                *hovered_item_index = Some(index);
            }
        } else if *hovered_item_index == Some(index) {
            *hovered_item_index = None;
        }
    }

    #[test]
    fn hover_false_only_clears_matching_index() {
        let mut hovered = Some(1usize);

        apply_accordion_item_hover(&mut hovered, 0, false);
        assert_eq!(hovered, Some(1));

        apply_accordion_item_hover(&mut hovered, 1, false);
        assert_eq!(hovered, None);

        apply_accordion_item_hover(&mut hovered, 2, true);
        assert_eq!(hovered, Some(2));
    }
}
