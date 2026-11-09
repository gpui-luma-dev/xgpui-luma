#[cfg(all(test, feature = "test-support"))]
#[path = "click_tests.rs"]
mod click_tests;

use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent,
    MouseDownEvent, Pixels, Render, ScrollEvent, SharedString, Subscription, TextRun, Window, div, font, prelude::*,
    px,
};

use crate::controls::selector_list::{SelectorItemsPanelLook, SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::motion::DisclosureMotion;
use crate::controls::selector::SelectorVisualState;
use crate::infra::interaction::ControlInteraction;

use crate::controls::popup_scroll_surface::PopupScrollSurface;
use crate::motion::overlay_presence::OverlayPresence;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::textfield::TextFieldState;
use crate::theme::observe_theme_revision;

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::item_template::SearchSelectorItemTemplate;
use super::model::{SearchSelectorBuilder, SearchSelectorPopupLookProvider};
use super::panel_template::{SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate};
use super::template::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers,
};
use super::text_selection::{self, TextSelectionEvent};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SearchSelectorEvent {
    Change { query: SharedString },
    Select { item_id: SharedString, label: SharedString },
    Complete { item_id: SharedString, label: SharedString },
    Clear,
    FocusChanged { focused: bool },
    OpenChanged { open: bool },
    Dismiss,
}

pub type SearchSelector = Entity<SearchSelectorControl>;

pub struct SearchSelectorControl {
    popup_search_textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    model: super::model::SearchSelectorModel,
    attachments: AttachmentHost,
    behavior: SelectionBehavior,
    interaction: ControlInteraction,
    trigger_bounds: Option<Bounds<Pixels>>,
    presence: OverlayPresence,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    committed_selection: Option<usize>,
    trigger_toggle_pending: bool,
    disclosure_transition: DisclosureMotion,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SearchSelectorEvent> for SearchSelectorControl {}

impl SearchSelectorControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
    ) -> SearchSelectorBuilder {
        SearchSelectorBuilder::new(id, items)
    }

    crate::presence_policy!(presence, disclosure_transition);

    crate::motion_snapshot!(super::SearchSelectorMotion, model.motion);

    pub(crate) fn from_builder(builder: SearchSelectorBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion.enabled();
        let model = builder.model;
        let enabled = model.enabled;
        let geometry = model.stylesheet.resolve_geometry(model.size.stylesheet_key(), Default::default());
        let padding_reduction = geometry.search_padding_reduction;
        let height_floor = geometry.search_input_height_floor;

        let popup_search_textfield = text_selection::new(format!("{}-popup-search", model.id))
            .placeholder(model.search_placeholder.clone())
            .enabled(model.enabled)
            .size(model.size)
            .full_width(true)
            .clean_on_escape(model.clean_on_escape)
            .propagate_home_end_to_parent(true)
            .look_override(move |mut look| {
                look.padding_y = (look.padding_y - padding_reduction).max(0.0);
                look.background = gpui::hsla(0.0, 0.0, 0.0, 0.0);
                look.typography.line_height = look.typography.size;
                look.min_height = (look.typography.line_height + (look.padding_y * 2.0)).max(height_floor);
                look
            })
            .template(model.textfield_template.clone())
            .spawn(cx);

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-popup-surface", model.id), model.scrollbar_template.clone(), cx);
        popup_surface.set_scrolling_enabled(model.scrolling);

        let subscriptions = vec![
            cx.subscribe(&popup_search_textfield, |this, _, event: &crate::controls::textfield::TextFieldEvent, cx| {
                if let Some(event) = text_selection::map_event(event) {
                    this.handle_popup_search_event(event, cx);
                }
            }),
            cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| {
                if let ScrollbarEvent::Change { value } = event {
                    this.popup_surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            }),
            observe_theme_revision(cx, |_, cx| cx.notify()),
        ];

        let committed_selection = builder
            .initial_selected_id
            .as_ref()
            .and_then(|selected_id| model.items.iter().position(|item| item.id == *selected_id && item.enabled));

        Self {
            popup_search_textfield,
            popup_surface,
            model,
            attachments: AttachmentHost::with_accessible_role(gpui::Role::ComboBox),
            behavior: SelectionBehavior::new(),
            interaction: ControlInteraction::new(enabled, cx),
            trigger_bounds: None,
            presence: OverlayPresence::with_motion(false, motion_enabled, motion.presence.clone()),
            last_event: SharedString::from("none"),
            last_keyboard_event: SharedString::from("none"),
            committed_selection,
            trigger_toggle_pending: false,
            disclosure_transition: DisclosureMotion::with_transition(0.0, motion_enabled, motion.disclosure.clone()),
            _subscriptions: subscriptions,
        }
    }

    fn index_by_id(&self, item_id: &SharedString) -> Option<usize> {
        self.model.items.iter().position(|item| item.id == *item_id && item.enabled)
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.committed_selection.and_then(|index| self.model.items.get(index)).map(|item| &item.id)
    }

    pub fn set_selected_id(&mut self, item_id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        let item_id = item_id.into();
        let next = self.index_by_id(&item_id);
        let changed = self.committed_selection != next;
        self.committed_selection = next;
        if changed {
            cx.notify();
        }
        changed
    }

    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
        cx: &mut Context<Self>,
    ) {
        let previous_selection = self.selected_id().cloned();
        self.model.items = items.into_iter().collect();
        self.committed_selection = previous_selection.as_ref().and_then(|id| self.index_by_id(id));
        self.behavior.apply(SelectionEvent::Blur, &self.model.items);
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.placeholder = placeholder.into();
        cx.notify();
    }

    pub fn set_size(&mut self, size: crate::theme::ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_size(size, cx));
        cx.notify();
    }

    fn handle_popup_search_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        match event {
            TextSelectionEvent::Change { value } => {
                let was_open = self.behavior.state.open;
                self.behavior.set_query(value.clone(), &self.model.items);
                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.state.open = true;
                }
                self.emit_open_changed_if_needed(was_open, false, cx);
                self.last_event = SharedString::from("search:change");

                if self.behavior.state.filtered.is_empty() {
                    self.clear_committed_selection(cx);
                }

                cx.emit(SearchSelectorEvent::Change { query: value.into() });
                self.sync_popup_highlight_visibility(cx);
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("search:submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                let was_focused = self.behavior.state.focused;
                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Focus, &self.model.items);
                self.emit_focus_changed_if_needed(was_focused, cx);
                self.emit_open_changed_if_needed(was_open, false, cx);
                if self.behavior.state.query.trim().is_empty() {
                    self.open_popup_with_all_items(cx);
                } else {
                    self.set_popup_open(true, cx);
                    self.sync_popup_highlight_visibility(cx);
                    cx.notify();
                }
            }
            TextSelectionEvent::FocusLeave => {
                let was_focused = self.behavior.state.focused;
                let was_open = self.behavior.state.open;
                if self.trigger_toggle_pending && was_open {
                    // Trigger mouse-down moves focus away from the popup before the
                    // trigger click arrives. Preserve the open state so that click
                    // can perform the intended close instead of reopening it.
                    self.behavior.state.focused = false;
                } else {
                    self.behavior.apply(SelectionEvent::Blur, &self.model.items);
                }
                self.emit_focus_changed_if_needed(was_focused, cx);
                if was_open && !self.trigger_toggle_pending {
                    self.emit_open_changed_if_needed(was_open, true, cx);
                }
                cx.notify();
            }
        }
    }

    fn handle_submit_result(&mut self, result: SubmitResult, cx: &mut Context<Self>) {
        match result {
            SubmitResult::None => {
                self.last_event = SharedString::from("submit:no-match");
                self.clear_committed_selection(cx);
                cx.notify();
            }
            SubmitResult::Select { index, exact_complete } => {
                self.select_item(index, exact_complete, cx);
            }
        }
    }

    fn clear_committed_selection(&mut self, cx: &mut Context<Self>) {
        self.committed_selection = None;
        cx.emit(SearchSelectorEvent::Clear);
    }

    fn clear_popup_search(&mut self, cx: &mut Context<Self>) {
        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.model.items[index];
        let item_id = item.id.clone();
        let label = item.label.clone();
        let was_open = self.behavior.state.open;

        self.behavior.select_index(index);
        self.committed_selection = Some(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.clear_popup_search(cx);
        self.emit_open_changed_if_needed(was_open, false, cx);

        if exact_complete {
            cx.emit(SearchSelectorEvent::Complete { item_id, label });
        } else {
            cx.emit(SearchSelectorEvent::Select { item_id, label });
        }
        cx.notify();
    }

    fn close_popup(&mut self, cx: &mut Context<Self>) {
        let was_open = self.behavior.state.open;
        self.behavior.state.open = false;
        self.behavior.state.highlighted_filtered = None;
        self.clear_popup_search(cx);
        self.emit_open_changed_if_needed(was_open, true, cx);
        cx.notify();
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let enabled_item = self
            .behavior
            .state
            .filtered
            .get(index)
            .and_then(|item_index| self.model.items.get(*item_index))
            .is_some_and(|item| item.enabled);

        if hovered && enabled_item && self.behavior.state.open {
            self.behavior.state.highlighted_filtered = Some(index);
            self.sync_popup_highlight_visibility(cx);
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() || !self.behavior.state.open {
            return;
        }

        if let Some(item_index) = self.behavior.state.filtered.get(index).copied()
            && self.model.items.get(item_index).is_some_and(|item| item.enabled)
        {
            self.select_item(item_index, false, cx);
            self.restore_trigger_focus(window, cx);
        }
    }

    fn popup_page_size(&self) -> usize {
        self.popup_surface.visible_row_count().clamp(1, 14)
    }

    fn move_highlight_page(&mut self, forward: bool, page_size: usize) {
        if self.behavior.state.filtered.is_empty() {
            return;
        }

        let len = self.behavior.state.filtered.len();
        let step = page_size.max(1);
        let current = self.behavior.state.highlighted_filtered.unwrap_or(if forward { 0 } else { len - 1 });
        let next = if forward {
            (current + step).min(len - 1)
        } else {
            current.saturating_sub(step)
        };

        self.behavior.state.highlighted_filtered = Some(next);
        self.behavior.state.open = true;
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.last_keyboard_event = SharedString::from(format!("key:{}", event.keystroke.key));

        if event.keystroke.modifiers.control || event.keystroke.modifiers.secondary(cx) {
            return;
        }

        match event.keystroke.key.as_str() {
            "escape" => {
                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Escape, &self.model.items);
                self.emit_open_changed_if_needed(was_open, true, cx);
                self.restore_trigger_focus(window, cx);
                cx.notify();
                window.prevent_default();
                cx.stop_propagation();
            }
            "down" | "arrowdown" => {
                if !self.behavior.state.open {
                    self.open_popup_with_all_items(cx);
                    self.focus_popup_search(window, cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    return;
                }

                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MoveNext, &self.model.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "up" | "arrowup" if self.behavior.state.open && !self.behavior.state.filtered.is_empty() => {
                self.behavior.apply(SelectionEvent::MovePrevious, &self.model.items);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "pagedown" | "page_down" | "pgdown" if !self.behavior.state.filtered.is_empty() => {
                self.set_popup_open(true, cx);
                self.move_highlight_page(true, self.popup_page_size());
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "pageup" | "page_up" | "pgup" if !self.behavior.state.filtered.is_empty() => {
                self.set_popup_open(true, cx);
                self.move_highlight_page(false, self.popup_page_size());
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "home" if !self.behavior.state.filtered.is_empty() => {
                self.set_popup_open(true, cx);
                self.behavior.state.highlighted_filtered = Some(0);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "end" if !self.behavior.state.filtered.is_empty() => {
                self.set_popup_open(true, cx);
                self.behavior.state.highlighted_filtered = Some(self.behavior.state.filtered.len() - 1);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "enter" if self.behavior.state.open => {
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
                window.prevent_default();
                cx.stop_propagation();
            }
            _ => {}
        }
        if !self.behavior.state.open {
            self.restore_trigger_focus(window, cx);
        }
    }

    fn open_popup_with_all_items(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.model.items.is_empty() {
            let was_open = self.behavior.state.open;
            self.behavior.state.filtered.clear();
            self.behavior.state.highlighted_filtered = None;
            self.behavior.state.open = false;
            self.emit_open_changed_if_needed(was_open, false, cx);
            return;
        }

        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
        self.behavior.state.query = SharedString::default();
        self.behavior.state.filtered = (0..self.model.items.len()).collect();
        self.behavior.state.highlighted_filtered =
            self.model.items.iter().enumerate().find_map(|(index, item)| item.enabled.then_some(index));
        self.set_popup_open(true, cx);
        self.behavior.state.status = SelectionStatus::Searching;
        self.sync_popup_highlight_visibility(cx);
        cx.notify();
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.trigger_toggle_pending = false;
        if !self.model.enabled || event.is_keyboard() {
            return;
        }

        if self.behavior.state.open {
            self.close_popup(cx);
            cx.stop_propagation();
            return;
        }

        self.open_popup_with_all_items(cx);
        self.focus_popup_search(window, cx);
        cx.stop_propagation();
    }

    fn handle_trigger_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_trigger_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.trigger_toggle_pending = self.behavior.state.open;
        if self.interaction.handle_mouse_down(self.model.enabled, window, cx) {
            cx.notify();
        }
    }

    fn handle_trigger_mouse_up(&mut self, _event: &gpui::MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_trigger_mouse_up_out(
        &mut self,
        _event: &gpui::MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.trigger_toggle_pending = false;
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(*bounds);
    }

    /// Update popup wheel without rebuilding its editor or selection.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        cx.notify();
    }

    /// Update popup boundary without rebuilding its editor or selection.
    pub fn set_scroll_boundary_policy(
        &mut self,
        policy: crate::interaction::ScrollBoundaryPolicy,
        cx: &mut Context<Self>,
    ) {
        self.model.scroll_interaction.boundary = policy;
        cx.notify();
    }

    /// Update popup focus_scope without rebuilding its editor or selection.
    pub fn set_wheel_focus_scope(&mut self, policy: crate::interaction::WheelFocusScope, cx: &mut Context<Self>) {
        self.model.scroll_interaction.focus_scope = policy;
        cx.notify();
    }

    fn handle_popup_scroll_wheel(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        use gpui::Focusable;
        if !self.model.enabled || !self.behavior.state.open || self.behavior.state.filtered.is_empty() {
            return;
        }
        let policy = self.model.scroll_interaction;
        let owner = self.popup_search_textfield.read(cx).focus_handle(cx);
        let bar = self.popup_surface.scrollbar().read(cx).focus_handle(cx);
        if !policy.wheel.accepts(policy.focus_scope.focused(&owner, Some(&bar), window, cx)) {
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

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_enabled(enabled, cx));
        if !enabled {
            let was_open = self.behavior.state.open;
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.clear_popup_search(cx);
            self.emit_open_changed_if_needed(was_open, false, cx);
        }
        cx.notify();
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.model.invalid != invalid {
            self.model.invalid = invalid;
            cx.notify();
        }
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn SearchSelectorTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    #[deprecated(note = "Prefer set_panel_template(...) + set_item_template(...)")]
    pub fn set_items_template(
        &mut self,
        template: std::sync::Arc<dyn SearchSelectorItemsTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.items_template = template;
        cx.notify();
    }

    pub fn set_panel_template(
        &mut self,
        template: std::sync::Arc<dyn SearchSelectorPanelTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.panel_template = template;
        cx.notify();
    }

    pub fn set_popup_look_provider(&mut self, provider: SearchSelectorPopupLookProvider, cx: &mut Context<Self>) {
        self.model.popup_look_provider = provider;
        cx.notify();
    }

    pub fn set_item_template(
        &mut self,
        template: Option<SearchSelectorItemTemplate<super::behavior::SelectionItem>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_template = template;
        cx.notify();
    }

    fn sync_popup_highlight_visibility(&self, cx: &mut Context<Self>) {
        if !self.behavior.state.open {
            return;
        }

        if let Some(visible_index) = self.behavior.state.highlighted_filtered {
            self.popup_surface.ensure_item_visible(visible_index, cx);
        }
    }

    fn sync_disabled_state(&mut self, cx: &mut Context<Self>) {
        if self.model.enabled || !self.behavior.state.open {
            return;
        }

        self.close_popup(cx);
    }

    fn set_popup_open(&mut self, open: bool, cx: &mut Context<Self>) -> bool {
        let was_open = self.behavior.state.open;
        self.behavior.state.open = open;
        self.emit_open_changed_if_needed(was_open, false, cx)
    }

    fn emit_open_changed_if_needed(&mut self, was_open: bool, dismissed: bool, cx: &mut Context<Self>) -> bool {
        if was_open == self.behavior.state.open {
            return false;
        }

        self.presence.set_open(self.behavior.state.open);
        self.disclosure_transition.set_target(if self.behavior.state.open { 1.0 } else { 0.0 });

        cx.emit(SearchSelectorEvent::OpenChanged { open: self.behavior.state.open });
        if dismissed && !self.behavior.state.open {
            cx.emit(SearchSelectorEvent::Dismiss);
        }
        true
    }

    fn emit_focus_changed_if_needed(&mut self, was_focused: bool, cx: &mut Context<Self>) -> bool {
        if was_focused == self.behavior.state.focused {
            return false;
        }

        cx.emit(SearchSelectorEvent::FocusChanged { focused: self.behavior.state.focused });
        true
    }

    fn restore_trigger_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.popup_search_textfield.read(cx).focus_handle(cx).is_focused(window) {
            self.interaction.focus_handle().focus(window, cx);
        }
    }

    fn focus_popup_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        let focus_handle = self.popup_search_textfield.read(cx).focus_handle(cx);
        focus_handle.focus(window, cx);
    }
}

impl Focusable for SearchSelectorControl {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for SearchSelectorControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.disclosure_transition.sync();
        self.disclosure_transition.schedule_frame(window, cx);
        self.presence.sync();
        self.presence.schedule_frame(window, cx);
        self.sync_disabled_state(cx);
        if !self.model.enabled && self.interaction.focus_handle().contains_focused(window, cx) {
            window.blur(cx);
        } else if !self.behavior.state.open {
            self.restore_trigger_focus(window, cx);
        }

        let trigger_focused = self.interaction.focus_handle().is_focused(window);
        let autocomplete_look = self.model.autocomplete_theme.resolve(self.model.size);
        let geometry = self.model.stylesheet.resolve_geometry(self.model.size.stylesheet_key(), Default::default());
        let look = (self.model.popup_look_provider)(self.model.size);
        let selected_label = self
            .committed_selection
            .and_then(|index| self.model.items.get(index))
            .map(|item| item.label.to_string());

        let minimum_trigger_width = {
            let textfield_look =
                self.model.textfield_template.resolve_look(TextFieldState::default(), true, self.model.size);
            let mut text_font = font(".SystemUIFont");
            text_font.weight = textfield_look.typography.weight;
            let placeholder_run = TextRun {
                len: self.model.placeholder.len(),
                font: text_font,
                color: autocomplete_look.muted_text_color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let placeholder_line = window.text_system().shape_line(
                self.model.placeholder.clone(),
                px(textfield_look.typography.size),
                &[placeholder_run],
                None,
            );
            let placeholder_width = placeholder_line.width().as_f32();

            let textfield_side_padding = (textfield_look.padding_x * 2.0) + textfield_look.border_width;
            let search_icon_width = geometry.search_icon_width + geometry.trigger_icon_extra_width;
            let spacing = geometry.trigger_extra_gap;

            px(textfield_side_padding + placeholder_width + search_icon_width + spacing)
        };

        let visible_indices = self.behavior.state.filtered.clone();

        let item_hovers = (0..visible_indices.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as SelectorPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..visible_indices.len())
            .map(|index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(index, event, window, cx);
                })) as SelectorPanelClickHandler
            })
            .collect::<Vec<_>>();

        let status_label = SharedString::from(format!("status: {}", self.behavior.state.status.label()));
        let status_detail = SharedString::from(format!(
            "selected: {} | matches: {} | last: {} | keyboard: {}",
            selected_label.clone().unwrap_or_else(|| "none".to_string()),
            self.behavior.state.filtered.len(),
            self.last_event,
            self.last_keyboard_event
        ));

        let popup_content = if self.model.enabled && self.presence.should_paint() {
            let list_id = SharedString::from("search-selector-menu");
            let row_height = px(look.item_height);
            let content_top_padding = px(look.padding);
            let item_count = visible_indices.len();
            let min_visible_rows = self.model.min_visible_rows.max(1);
            let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
            let effective_max_visible_rows = if self.model.fill_popup_viewport {
                max_visible_rows_for_viewport(
                    self.trigger_bounds,
                    window.viewport_size().height,
                    &look,
                    min_visible_rows,
                    geometry.search_chrome_height,
                )
            } else {
                max_visible_rows
            };
            let visible_rows = item_count.max(1).clamp(min_visible_rows, effective_max_visible_rows) as f32;
            let viewport_height = px((look.padding * 2.0) + (look.item_height * visible_rows));
            let allow_scrolling = self.model.scrolling && item_count > effective_max_visible_rows;

            self.popup_surface.set_scrolling_enabled(allow_scrolling);
            self.popup_surface.configure(item_count.max(1), row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let list_content = if visible_indices.is_empty() {
                div()
                    .flex()
                    .items_center()
                    .min_h(px(look.item_height))
                    .px(px(look.item_padding_x))
                    .text_size(px(look.item_typography.size))
                    .line_height(px(look.item_typography.line_height))
                    .font_weight(look.item_typography.weight)
                    .text_color(autocomplete_look.muted_text_color)
                    .child("No matches")
                    .into_any_element()
            } else {
                self.model
                    .items_template
                    .render(
                        &SearchSelectorItemsRenderModel {
                            trailing_icon_size: geometry.trailing_icon_size,
                            menu_id: &list_id,
                            search_selector_id: &self.model.id,
                            items: &self.model.items,
                            visible_indices: &visible_indices,
                            selected_source_index: self.behavior.state.selected_item,
                            active_visible_index: self.behavior.state.highlighted_filtered,
                            open: self.presence.should_paint(),
                            enabled: self.model.enabled,
                            item_template: self.model.item_template.as_ref(),
                            look: look.clone(),
                        },
                        SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks },
                        cx,
                    )
                    .into_any_element()
            };

            let panel_content = self.model.panel_template.render(
                SearchSelectorPanelRenderModel {
                    geometry,
                    id: &self.model.id,
                    items: &self.model.items,
                    visible_indices: &visible_indices,
                    selected_source_index: self.behavior.state.selected_item,
                    active_visible_index: self.behavior.state.highlighted_filtered,
                    open: self.presence.should_paint(),
                    enabled: self.model.enabled,
                    item_template: self.model.item_template.as_ref(),
                    popup_bounds: self.trigger_bounds,
                    popup_look: look.clone(),
                    search_content: Some(self.popup_search_textfield.clone().into_any_element()),
                    list_content: self
                        .popup_surface
                        .render_with_scroll_wheel(list_content, cx.listener(Self::handle_popup_scroll_wheel)),
                },
                cx,
            );

            Some(panel_content)
        } else {
            None
        };

        let handlers = SearchSelectorTemplateHandlers {
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            // The popup surface owns wheel dispatch, even outside the trigger bounds.
            scroll_wheel: Box::new(|_, _, _| {}),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_hover: Box::new(cx.listener(Self::handle_trigger_hover)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_trigger_mouse_down)),
            trigger_mouse_up: Box::new(cx.listener(Self::handle_trigger_mouse_up)),
            trigger_mouse_up_out: Box::new(cx.listener(Self::handle_trigger_mouse_up_out)),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
        };

        let interaction_state = self.interaction.render_state(self.model.enabled, window);
        let trigger_look = self.model.selector_theme.resolve_visual_look(
            self.model.trigger_style,
            SelectorVisualState {
                interaction: interaction_state,
                open: self.behavior.state.open,
                selected: self.committed_selection.is_some(),
                invalid: self.model.invalid,
            },
            self.model.size,
            &crate::theme::StandardBoxScale::compute(
                self.model.size,
                &self.model.selector_theme.metrics(),
                window.scale_factor(),
            ),
            self.model.without_elevation,
        );
        let render_model = SearchSelectorRenderModel {
            icons: self.model.icons.clone(),
            geometry,
            id: self.model.id.clone(),
            trigger_label: selected_label.map(SharedString::from).unwrap_or_else(|| self.model.placeholder.clone()),
            trigger_label_is_placeholder: self.committed_selection.is_none(),
            trigger_state: TextFieldState {
                hovered: interaction_state.hovered,
                focused: self.behavior.state.open || trigger_focused,
                focus_visible: self.behavior.state.open || trigger_focused,
                invalid: self.model.invalid,
                ..TextFieldState::default()
            },
            trigger_look,
            trigger_typography_override: None,
            enabled: self.model.enabled,
            size: self.model.size,
            full_width: self.model.full_width,
            minimum_trigger_width,
            status_label,
            status_detail,
            status_color: autocomplete_look.status_color,
            muted_text_color: autocomplete_look.muted_text_color,
            popup_content,
            presence: self.presence.clone(),
            disclosure_progress: self.disclosure_transition.progress(),
        };

        let mut tip_state = interaction_state;
        tip_state.disabled |= self.behavior.state.open;
        let root = self.model.template.render(render_model, handlers, window, cx);
        let root = self.attachments.render(root, tip_state, cx);
        div()
            .when(self.model.full_width, |root| root.w_full().h_full())
            .child(root.track_focus(self.interaction.focus_handle()).capture_key_down(cx.listener(
                |this, event: &KeyDownEvent, window, cx| {
                    if this.behavior.state.open && event.keystroke.key == "escape" {
                        this.handle_key_down(event, window, cx);
                    }
                },
            )))
            .into_any_element()
    }
}

fn max_visible_rows_for_viewport(
    trigger_bounds: Option<Bounds<Pixels>>,
    viewport_height: Pixels,
    look: &SelectorItemsPanelLook,
    min_visible_rows: usize,
    search_chrome_height: f32,
) -> usize {
    if look.item_height <= 0.0 {
        return min_visible_rows.max(1);
    }
    let viewport_bottom = viewport_height - px(look.window_margin);
    let available_below = trigger_bounds
        .map(|bounds| (viewport_bottom - (bounds.bottom() + px(look.popup_offset_y))).max(px(0.0)))
        .unwrap_or(viewport_bottom)
        .as_f32();
    let list_area = (available_below - search_chrome_height - (look.padding * 2.0)).max(look.item_height);
    ((list_area / look.item_height).floor() as usize).max(min_visible_rows).max(1)
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn escape_from_popup_search_restores_trigger_focus() {
    use gpui::{TestAppContext, point};
    for key in ["escape", "enter"] {
        let mut app = TestAppContext::single();
        let (selector, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            SearchSelectorControl::from_builder(
                SearchSelectorBuilder::new("restore-search", [super::behavior::SelectionItem::new("one", "One")]),
                cx,
            )
        });
        cx.run_until_parked();
        cx.simulate_click(point(px(25.0), px(15.0)), Default::default());
        cx.run_until_parked();
        cx.update(|window, app| {
            let selector = selector.read(app);
            assert!(selector.behavior.state.open);
            assert!(selector.popup_search_textfield.read(app).focus_handle(app).is_focused(window));
        });
        cx.simulate_keystrokes(key);
        cx.run_until_parked();
        cx.update(|window, app| {
            let selector = selector.read(app);
            assert!(!selector.behavior.state.open);
            assert!(selector.interaction.focus_handle().is_focused(window));
        });
    }
}

impl AttachmentTarget for SearchSelectorControl {
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
mod geometry_tests {
    use super::*;

    #[test]
    fn viewport_row_budget_uses_resolved_placement_and_handles_zero_row_height() {
        let mut look = crate::controls::selector_list::default_selector_items_panel_look(
            &crate::theme::ThemeTokens::default(),
            crate::theme::ControlSize::Md,
        );
        look.item_height = 10.0;
        look.padding = 2.0;
        let trigger = Some(Bounds::new(gpui::point(px(0.0), px(10.0)), gpui::size(px(80.0), px(20.0))));
        assert_eq!(max_visible_rows_for_viewport(trigger, px(100.0), &look, 1, 57.0), 1);
        look.window_margin = 0.0;
        look.popup_offset_y = 0.0;
        assert_eq!(max_visible_rows_for_viewport(trigger, px(100.0), &look, 1, 10.0), 5);
        look.item_height = 0.0;
        assert_eq!(max_visible_rows_for_viewport(trigger, px(100.0), &look, 3, 0.0), 3);
    }
}
