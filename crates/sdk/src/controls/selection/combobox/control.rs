#[cfg(all(test, feature = "test-support"))]
#[path = "click_tests.rs"]
mod click_tests;

use gpui::App;
use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, IntoElement, KeyDownEvent, MouseDownEvent, Pixels, Render,
    ScrollEvent, SharedString, Subscription, TextRun, Window, font, px,
};

use crate::controls::selector_list::{SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::textfield::TextFieldState;
use crate::theme::observe_theme_revision;
use crate::motion::overlay_presence::OverlayPresence;

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::item_template::ComboBoxItemTemplate;
use super::model::ComboBoxBuilder;
use super::panel_template::{ComboBoxPanelRenderModel, ComboBoxPanelTemplate};
use super::template::{ComboBoxItemsTemplate, ComboBoxTemplate};
use crate::controls::popup_scroll_surface::PopupScrollSurface;
use super::template::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplateHandlers, ComboBoxRenderModel, ComboBoxTemplateHandlers,
};
use super::text_selection::{self, TextSelectionEvent};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ComboBoxEvent {
    Change { query: SharedString },
    Select { item_id: SharedString, label: SharedString },
    Complete { item_id: SharedString, label: SharedString },
    Clear,
    FocusChanged { focused: bool },
    OpenChanged { open: bool },
    Dismiss,
}

pub type ComboBox = Entity<ComboBoxControl>;

pub struct ComboBoxControl {
    textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    model: super::model::ComboBoxModel,
    behavior: SelectionBehavior,
    trigger_bounds: Option<Bounds<Pixels>>,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    committed_selection: Option<usize>,
    open_popup_on_next_focus: bool,
    presence: OverlayPresence,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ComboBoxEvent> for ComboBoxControl {}

impl ComboBoxControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
    ) -> ComboBoxBuilder {
        ComboBoxBuilder::new(id, items)
    }

    crate::presence_policy!(presence);

    crate::motion_snapshot!(super::ComboBoxMotion, model.motion);

    pub(crate) fn from_builder(builder: ComboBoxBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion.enabled();
        let model = builder.model;

        let textfield = text_selection::new(format!("{}-textfield", model.id))
            .placeholder(model.placeholder.clone())
            .enabled(model.enabled)
            .size(model.size)
            .full_width(model.full_width)
            .clean_on_escape(model.clean_on_escape)
            .propagate_home_end_to_parent(true)
            .template(model.textfield_template.clone())
            .spawn(cx);
        textfield.update(cx, |textfield, cx| textfield.set_invalid(model.invalid, cx));

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-popup-surface", model.id), model.scrollbar_template.clone(), cx);
        popup_surface.set_scrolling_enabled(model.scrolling);

        let subscriptions = vec![
            cx.subscribe(&textfield, |this, _, event: &crate::controls::textfield::TextFieldEvent, cx| {
                if let Some(event) = text_selection::map_event(event) {
                    this.handle_text_selection_event(event, cx);
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

        Self {
            textfield,
            popup_surface,
            model,
            behavior: SelectionBehavior::new(),
            trigger_bounds: None,
            last_event: SharedString::from("none"),
            last_keyboard_event: SharedString::from("none"),
            committed_selection: None,
            open_popup_on_next_focus: false,
            presence: OverlayPresence::with_motion(false, motion_enabled, motion.presence.clone()),
            _subscriptions: subscriptions,
        }
    }

    fn handle_text_selection_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        match event {
            TextSelectionEvent::Change { value } => {
                let was_open = self.behavior.state.open;
                self.behavior.set_query(value.clone(), &self.model.items);
                self.last_event = SharedString::from("change");
                self.emit_open_changed_if_needed(was_open, false, cx);
                cx.emit(ComboBoxEvent::Change { query: value.into() });
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                let was_focused = self.behavior.state.focused;
                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Focus, &self.model.items);
                self.emit_focus_changed_if_needed(was_focused, cx);
                self.emit_open_changed_if_needed(was_open, false, cx);
                if self.open_popup_on_next_focus {
                    self.open_popup_on_next_focus = false;
                    self.open_popup_with_all_items(cx);
                }
                cx.notify();
            }
            TextSelectionEvent::FocusLeave => {
                let was_focused = self.behavior.state.focused;
                let was_open = self.behavior.state.open;
                self.open_popup_on_next_focus = false;
                self.apply_blur_selection_policy(cx);
                self.behavior.apply(SelectionEvent::Blur, &self.model.items);
                self.emit_focus_changed_if_needed(was_focused, cx);
                self.emit_open_changed_if_needed(was_open, true, cx);
                cx.notify();
            }
        }
    }

    fn handle_submit_result(&mut self, result: SubmitResult, cx: &mut Context<Self>) {
        match result {
            SubmitResult::None => {
                self.last_event = SharedString::from("submit:no-match");
                cx.notify();
            }
            SubmitResult::Select { index, exact_complete } => {
                self.select_item(index, exact_complete, cx);
            }
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        let was_open = self.behavior.state.open;
        self.committed_selection = None;
        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.last_event = SharedString::from("clear");

        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));

        self.emit_open_changed_if_needed(was_open, false, cx);
        cx.emit(ComboBoxEvent::Clear);
        cx.notify();
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.model.items[index];
        let item_id = item.id.clone();
        let label = item.label.clone();
        let was_open = self.behavior.state.open;

        self.behavior.set_query(label.clone(), &self.model.items);
        self.behavior.select_index(index);
        self.committed_selection = Some(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));

        let label = self.model.items[index].label.clone();
        self.emit_open_changed_if_needed(was_open, false, cx);
        if exact_complete {
            cx.emit(ComboBoxEvent::Complete { item_id, label });
        } else {
            cx.emit(ComboBoxEvent::Select { item_id, label });
        }
        cx.notify();
    }

    fn apply_blur_selection_policy(&mut self, cx: &mut Context<Self>) {
        let query = self.behavior.state.query.to_string();
        let trimmed_query = query.trim();

        if trimmed_query.is_empty() {
            self.committed_selection = None;
            self.behavior.apply(SelectionEvent::Clear, &self.model.items);
            self.last_event = SharedString::from("blur:clear-empty");
            self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
            return;
        }

        if self.behavior.state.status == SelectionStatus::Complete {
            if let Some(index) = self.behavior.state.selected_item {
                self.committed_selection = Some(index);
            }
            self.last_event = SharedString::from("blur:commit");
            return;
        }

        if let Some(index) = self.committed_selection.filter(|index| *index < self.model.items.len()) {
            let item = &self.model.items[index];
            let label = item.label.clone();
            self.behavior.set_query(label.clone(), &self.model.items);
            self.behavior.select_index(index);
            self.last_event = SharedString::from(format!("blur:revert:{}", item.label));
            self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));
            return;
        }

        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.last_event = SharedString::from("blur:clear-invalid");
        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
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

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() || !self.behavior.state.open {
            return;
        }

        if let Some(item_index) = self.behavior.state.filtered.get(index).copied()
            && self.model.items.get(item_index).is_some_and(|item| item.enabled)
        {
            self.select_item(item_index, false, cx);
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
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "down" | "arrowdown" => {
                if !self.behavior.state.open {
                    self.open_popup_with_all_items(cx);
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
            _ => {}
        }
    }

    fn handle_clear_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() {
            return;
        }

        self.clear(cx);
        cx.stop_propagation();
    }

    fn open_popup_with_all_items(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let was_open = self.behavior.state.open;
        if self.model.items.is_empty() {
            self.behavior.state.filtered.clear();
            self.behavior.state.highlighted_filtered = None;
            self.behavior.state.open = false;
            self.emit_open_changed_if_needed(was_open, false, cx);
            return;
        }

        self.behavior.state.filtered = (0..self.model.items.len()).collect();
        self.behavior.state.highlighted_filtered =
            self.model.items.iter().enumerate().find_map(|(index, item)| item.enabled.then_some(index));
        self.behavior.state.open = true;
        self.behavior.state.status = SelectionStatus::Searching;
        self.sync_popup_highlight_visibility(cx);
        self.emit_open_changed_if_needed(was_open, false, cx);
        cx.notify();
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() {
            return;
        }

        if self.behavior.state.open {
            let was_open = self.behavior.state.open;
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.open_popup_on_next_focus = false;
            self.emit_open_changed_if_needed(was_open, true, cx);
            cx.stop_propagation();
            cx.notify();
            return;
        }

        if self.is_textfield_focused(cx) {
            self.open_popup_with_all_items(cx);
        } else {
            self.open_popup_on_next_focus = true;
        }

        cx.stop_propagation();
    }

    fn handle_trigger_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if event.click_count >= 2 {
            if self.is_textfield_focused(cx) {
                self.open_popup_with_all_items(cx);
            } else {
                self.open_popup_on_next_focus = true;
            }
        }
    }

    fn is_textfield_focused(&self, cx: &mut Context<Self>) -> bool {
        let mut focused = false;
        self.textfield.update(cx, |textfield, _| {
            focused = textfield.is_focused();
        });
        focused
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
        let owner = self.textfield.read(cx).focus_handle(cx);
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

    pub fn set_size(&mut self, size: crate::theme::ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        self.textfield.update(cx, |textfield, cx| textfield.set_size(size, cx));
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.textfield.update(cx, |textfield, cx| textfield.set_enabled(enabled, cx));
        if !enabled {
            let was_open = self.behavior.state.open;
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.open_popup_on_next_focus = false;
            self.emit_open_changed_if_needed(was_open, false, cx);
        }
        cx.notify();
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.model.invalid != invalid {
            self.model.invalid = invalid;
            self.textfield.update(cx, |textfield, cx| textfield.set_invalid(invalid, cx));
            cx.notify();
        }
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn ComboBoxTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_items_template(&mut self, template: std::sync::Arc<dyn ComboBoxItemsTemplate>, cx: &mut Context<Self>) {
        self.model.items_template = template;
        cx.notify();
    }

    pub fn set_panel_template(&mut self, template: std::sync::Arc<dyn ComboBoxPanelTemplate>, cx: &mut Context<Self>) {
        self.model.panel_template = template;
        cx.notify();
    }

    pub fn set_item_template(
        &mut self,
        template: Option<ComboBoxItemTemplate<super::behavior::SelectionItem>>,
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

        use crate::infra::attachments::AttachmentTarget;
        self.textfield.update(cx, |field, cx| {
            field.attachments_mut().set_blocked(self.behavior.state.open);
            if self.behavior.state.open {
                field.attachments().dismiss(cx);
            }
            cx.notify();
        });
        cx.emit(ComboBoxEvent::OpenChanged { open: self.behavior.state.open });
        if dismissed && !self.behavior.state.open {
            cx.emit(ComboBoxEvent::Dismiss);
        }
        true
    }

    fn emit_focus_changed_if_needed(&mut self, was_focused: bool, cx: &mut Context<Self>) -> bool {
        if was_focused == self.behavior.state.focused {
            return false;
        }

        cx.emit(ComboBoxEvent::FocusChanged { focused: self.behavior.state.focused });
        true
    }
}

impl Render for ComboBoxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.presence.sync();
        self.presence.schedule_frame(window, cx);
        let autocomplete_look = self.model.autocomplete_theme.resolve(self.model.size);
        let geometry = self.model.stylesheet.resolve_geometry(self.model.size.stylesheet_key(), Default::default());
        let look = (self.model.popup_look_provider)(self.model.size);
        let selected_label = self
            .behavior
            .state
            .selected_item
            .map(|index| self.model.items[index].label.to_string())
            .unwrap_or_else(|| "none".to_string());

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
            let affordance_width =
                reserved_affordance_width(&geometry, self.model.show_down_arrow, self.model.show_clear_button);
            px(textfield_side_padding + placeholder_width + affordance_width)
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
            "selected: {selected_label} | matches: {} | last: {} | keyboard: {}",
            self.behavior.state.filtered.len(),
            self.last_event,
            self.last_keyboard_event
        ));

        let popup_content = if self.model.enabled && self.presence.should_paint() && !visible_indices.is_empty() {
            let menu_id = SharedString::from("combobox-menu");
            let row_height = px(look.item_height);
            let content_top_padding = px(look.padding);
            let item_count = visible_indices.len();
            let min_visible_rows = self.model.min_visible_rows.max(1);
            let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
            let visible_rows = item_count.clamp(min_visible_rows, max_visible_rows) as f32;
            let viewport_height = px((look.padding * 2.0) + (look.item_height * visible_rows));
            let allow_scrolling = self.model.scrolling && item_count > max_visible_rows;

            self.popup_surface.set_scrolling_enabled(allow_scrolling);
            self.popup_surface.configure(item_count, row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let list_content = self.model.items_template.render(
                &ComboBoxItemsRenderModel {
                    menu_id: &menu_id,
                    combobox_id: &self.model.id,
                    items: &self.model.items,
                    visible_indices: &visible_indices,
                    selected_source_index: self.behavior.state.selected_item,
                    active_visible_index: self.behavior.state.highlighted_filtered,
                    open: self.presence.should_paint(),
                    enabled: self.model.enabled,
                    item_template: self.model.item_template.as_ref(),
                    look: look.clone(),
                },
                ComboBoxItemsTemplateHandlers { item_hovers, item_clicks },
                cx,
            );

            let panel_content = self.model.panel_template.render(
                ComboBoxPanelRenderModel {
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
                    list_content: self.popup_surface.render_with_scroll_wheel(
                        list_content.into_any_element(),
                        cx.listener(Self::handle_popup_scroll_wheel),
                    ),
                },
                cx,
            );

            Some(panel_content)
        } else {
            None
        };

        let handlers = ComboBoxTemplateHandlers {
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            // The popup surface owns wheel dispatch, even outside the trigger bounds.
            scroll_wheel: Box::new(|_, _, _| {}),
            clear_click: Box::new(cx.listener(Self::handle_clear_click)),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_trigger_mouse_down)),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
        };

        let render_model = ComboBoxRenderModel {
            icons: self.model.icons.clone(),
            geometry,
            textfield: self.textfield.clone().into_any_element(),
            query_is_empty: self.behavior.state.query.is_empty(),
            show_down_arrow: self.model.show_down_arrow,
            show_clear_button: self.model.show_clear_button,
            full_width: self.model.full_width,
            minimum_trigger_width,
            status_label,
            status_detail,
            status_color: autocomplete_look.status_color,
            muted_text_color: autocomplete_look.muted_text_color,
            popup_bounds: self.trigger_bounds,
            popup_look: look,
            popup_content,
            presence: self.presence.clone(),
        };

        self.model.template.render(render_model, handlers, window, cx)
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn popup_wheel_applies_once_and_pass_through_reaches_page() {
    use gpui::{Focusable, ScrollDelta, ScrollHandle, TestAppContext, point};
    use crate::interaction::{ScrollBoundaryPolicy, WheelScrollPolicy};
    use crate::interaction_tests::Page;
    let mut app = TestAppContext::single();
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        Page {
            child: ComboBoxBuilder::new(
                "popup-wheel",
                (0..40).map(|i| super::behavior::SelectionItem::new(i.to_string(), format!("Item {i}"))),
            )
            .spawn(cx),
            scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
        }
    });
    cx.run_until_parked();
    let combo = cx.update(|window, app| {
        let combo = page.read(app).child.clone();
        combo.read(app).textfield.read(app).focus_handle(app).focus(window, app);
        combo
    });
    combo.update(cx, |combo, cx| combo.open_popup_with_all_items(cx));
    cx.run_until_parked();
    let position = cx.update(|_, app| {
        let bounds = combo.read(app).trigger_bounds.unwrap();
        point(bounds.left() + px(30.0), bounds.bottom() + px(40.0))
    });
    let event = ScrollEvent { position, delta: ScrollDelta::Pixels(point(px(0.0), px(-25.0))), ..Default::default() };
    cx.simulate_event(event.clone());
    cx.run_until_parked();
    cx.update(|window, app| {
        assert_eq!(combo.read(app).popup_surface.vertical_offset(), px(25.0));
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
        assert!(combo.read(app).textfield.read(app).focus_handle(app).is_focused(window));
    });
    combo.update(cx, |combo, cx| {
        combo.set_wheel_scroll_policy(WheelScrollPolicy::PassThrough, cx);
        combo.set_scroll_boundary_policy(ScrollBoundaryPolicy::Contain, cx);
    });
    cx.run_until_parked();
    cx.simulate_event(event);
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(combo.read(app).popup_surface.vertical_offset(), px(25.0));
        assert!(page.read(app).scroll.offset().y < px(0.0));
    });
}

// Editable selectors attach help to their existing focused TextField.
impl crate::infra::attachments::TooltipEntityExt for gpui::Entity<ComboBoxControl> {
    fn with_tooltip_theme(
        self,
        theme: std::sync::Arc<dyn crate::controls::tooltip::TooltipTheme>,
        cx: &mut App,
    ) -> Self {
        self.read(cx).textfield.clone().with_tooltip_theme(theme, cx);
        self
    }
    fn help(self, text: impl Into<SharedString>, cx: &mut App) -> Self {
        self.read(cx).textfield.clone().help(text, cx);
        self
    }
    fn tooltip(self, config: crate::controls::tooltip::Tooltip, cx: &mut App) -> Self {
        self.read(cx).textfield.clone().tooltip(config, cx);
        self
    }
    fn clear_tooltip(self, cx: &mut App) -> Self {
        self.read(cx).textfield.clone().clear_tooltip(cx);
        self
    }
}

// Preserve the stock inset reservation while allowing a wider configured gap
// between both affordances to increase the minimum trigger width.
fn reserved_affordance_width(geometry: &crate::theme::stylesheet::ComboBoxGeometry, arrow: bool, clear: bool) -> f32 {
    let inset = geometry.affordance_inset;
    let width = geometry.affordance_width;
    let arrow_width = if arrow { width + inset } else { 0.0 };
    let clear_inset = if arrow {
        inset.max(geometry.affordance_gap)
    } else {
        inset
    };
    let clear_width = if clear { width + clear_inset } else { 0.0 };
    arrow_width + clear_width + geometry.trigger_extra_gap
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    use crate::theme::stylesheet::CommonStylesheet;

    #[test]
    fn affordance_reservation_retains_defaults_and_grows_with_configured_spacing() {
        let baseline = CommonStylesheet::sdk_default().combobox.resolve_geometry("md", Default::default());
        assert_eq!(reserved_affordance_width(&baseline, false, false), 8.0);
        assert_eq!(reserved_affordance_width(&baseline, true, false), 36.0);
        assert_eq!(reserved_affordance_width(&baseline, false, true), 36.0);
        assert_eq!(reserved_affordance_width(&baseline, true, true), 64.0);
        let sheet = CommonStylesheet::resolve_layers(&["[common.combobox.geometry]\naffordance_gap=20"]).unwrap();
        let geometry = sheet.combobox.resolve_geometry("md", Default::default());
        assert_eq!(reserved_affordance_width(&geometry, true, true), 74.0);
        let zero = CommonStylesheet::resolve_layers(&[
            "[common.combobox.geometry]\naffordance_inset=0\naffordance_width=0\naffordance_gap=0\ntrigger_extra_gap=0",
        ])
        .unwrap();
        assert_eq!(
            reserved_affordance_width(&zero.combobox.resolve_geometry("md", Default::default()), true, true),
            0.0
        );
    }
}
