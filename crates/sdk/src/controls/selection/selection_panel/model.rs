use crate::interaction::{ScrollInteraction, WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled};

use super::control::SelectionPanelControl;
use super::item_template::{
    SelectionPanelItemRenderModel, SelectionPanelItemTemplate, item_template_with_modifier,
    make_selection_panel_item_template,
};
use super::template::{SelectionPanelRenderModel, template_with_modifier};
use crate::infra::icon::IconSource;
use crate::infra::icon::SelectionStatusIcons;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarVisibility};

use super::template::{SelectionPanelTemplate, default_selection_panel_template};

pub trait SelectionPanelItemLike {
    fn id(&self) -> &SharedString;

    fn label(&self) -> &SharedString {
        self.id()
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn icon(&self) -> Option<&IconSource> {
        None
    }

    fn is_selected(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct SelectionPanelItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<IconSource>,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
}

impl SelectionPanelItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, icon: None, enabled: true, selected: false }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }
}

impl SelectionPanelItemLike for SelectionPanelItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn icon(&self) -> Option<&IconSource> {
        self.icon.as_ref()
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelPath {
    Item(usize),
}

impl SelectionPanelPath {
    pub fn is_item(self, index: usize) -> bool {
        matches!(self, Self::Item(active) if active == index)
    }
}

/// Role selects the default hover behavior, not wheel eligibility or focus ownership.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectionPanelRole {
    #[default]
    Embedded,
    Popup,
}
/// Explicit hover override. Hover never acquires focus or commits selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoverActivationPolicy {
    PreserveActive,
    FollowPointer,
}

#[derive(Clone)]
pub struct SelectionPanelModel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) panel_id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) visible_indices: Vec<usize>,
    pub(crate) selected_source_index: Option<usize>,
    pub(crate) open: bool,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: ScrollInteraction,
    pub(crate) pointer_focus: crate::interaction::PointerFocusPolicy,
    pub(crate) role: SelectionPanelRole,
    pub(crate) hover_activation: Option<HoverActivationPolicy>,
    pub(crate) show_selection_marker: bool,
    pub(crate) icons_explicit: bool,
    pub(crate) icons: SelectionStatusIcons,
    pub(crate) scrolling: bool,
    pub(crate) scrollbar_visibility: ScrollbarVisibility,
    pub(crate) scrollbar_auto_hide_activate: ScrollbarAutoHideActivate,
    pub(crate) min_visible_rows_explicit: bool,
    pub(crate) max_visible_rows_explicit: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) item_template: Option<SelectionPanelItemTemplate<T>>,
    pub(crate) template: Arc<dyn SelectionPanelTemplate<T>>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) menu: crate::controls::popup_menu::MenuSource,
}

pub struct SelectionPanelBuilder<T = SelectionPanelItem>
where
    T: SelectionPanelItemLike + 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::selection_panel::SelectionPanelEvent>,

    pub(crate) model: SelectionPanelModel<T>,
    pub(crate) initial_active_visible_index: Option<usize>,
}

impl<T> SelectionPanelBuilder<T>
where
    T: SelectionPanelItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::selection_panel::SelectionPanelEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SelectionPanelEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::HoverChanged { visible_index } = event {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActivateRow` through the existing event stream.
    pub fn on_activate_row<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, usize, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::ActivateRow {
                source_index,
                visible_index,
                item_id,
            } = event
            {
                callback(owner, *source_index, *visible_index, item_id, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActiveIndexChanged` through the existing event stream.
    pub fn on_active_index_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::ActiveIndexChanged { visible_index } = event {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),
            model: default_selection_panel_model(id),
            initial_active_visible_index: None,
        }
    }

    /// Select role defaults; an explicit hover policy takes precedence in either order.
    pub fn role(mut self, role: SelectionPanelRole) -> Self {
        self.model.role = role;
        self
    }
    /// Override hover-to-active independently of role and scrolling.
    pub fn hover_activation_policy(mut self, policy: HoverActivationPolicy) -> Self {
        self.model.hover_activation = Some(policy);
        self
    }

    pub fn panel_id(mut self, panel_id: impl Into<SharedString>) -> Self {
        self.model.panel_id = panel_id.into();
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self.model.visible_indices = (0..self.model.items.len()).collect();
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self.model.visible_indices = (0..self.model.items.len()).collect();
        self
    }

    pub fn visible_indices(mut self, visible_indices: impl IntoIterator<Item = usize>) -> Self {
        self.model.visible_indices = visible_indices.into_iter().collect();
        self
    }

    pub fn selected_source_index(mut self, selected_source_index: Option<usize>) -> Self {
        self.model.selected_source_index = selected_source_index;
        self
    }

    pub fn active_visible_index(mut self, active_visible_index: Option<usize>) -> Self {
        self.initial_active_visible_index = active_visible_index;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    /// Configure pointer_focus independently of wheel and keyboard ownership.
    pub fn pointer_focus_policy(mut self, policy: crate::interaction::PointerFocusPolicy) -> Self {
        self.model.pointer_focus = policy;
        self
    }

    /// Override wheel without changing other interaction settings.
    pub fn wheel_scroll_policy(mut self, policy: WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Override boundary without changing other interaction settings.
    pub fn scroll_boundary_policy(mut self, policy: ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Override focus_scope without changing other interaction settings.
    pub fn wheel_focus_scope(mut self, policy: WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn show_selection_marker(mut self, show_selection_marker: bool) -> Self {
        self.model.show_selection_marker = show_selection_marker;
        self
    }

    /// Capture default selection glyphs; explicit icons retain precedence.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let (minimum, maximum) = stylesheet.row_limits.resolve("selection_panel");
        if !self.model.min_visible_rows_explicit {
            self.model.min_visible_rows = minimum;
        }
        if !self.model.max_visible_rows_explicit {
            self.model.max_visible_rows = maximum;
        }
        if !self.model.icons_explicit {
            self.model.icons = SelectionStatusIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.model.icons_explicit = true;
        self.model.icons = icons;
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.model.scrolling = scrolling;
        self
    }

    /// Always show, hide, or auto-hide the scrollbar. Defaults to AlwaysVisible.
    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    /// Choose what reveals auto-hiding chrome. Move hides after the existing idle timeout.
    pub fn scrollbar_auto_hide_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.model.scrollbar_auto_hide_activate = activate;
        self
    }

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.model.min_visible_rows_explicit = true;
        self.model.min_visible_rows = min_visible_rows.max(1);
        if self.model.max_visible_rows < self.model.min_visible_rows {
            self.model.max_visible_rows = self.model.min_visible_rows;
        }
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.model.max_visible_rows_explicit = true;
        self.model.max_visible_rows = max_visible_rows.max(1);
        if self.model.min_visible_rows > self.model.max_visible_rows {
            self.model.min_visible_rows = self.model.max_visible_rows;
        }
        self
    }

    pub fn visible_row_limits(mut self, min_visible_rows: usize, max_visible_rows: usize) -> Self {
        self.model.min_visible_rows_explicit = true;
        self.model.max_visible_rows_explicit = true;
        self = self.min_visible_rows(min_visible_rows);
        self.max_visible_rows(max_visible_rows)
    }

    pub fn item_template(mut self, item_template: SelectionPanelItemTemplate<T>) -> Self {
        self.model.item_template = Some(item_template);
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_selection_panel_item_template(template));
        self
    }

    pub fn with_item_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::AnyElement, &SelectionPanelItemRenderModel<'a, T>, &mut App) -> gpui::AnyElement
            + Send
            + Sync
            + 'static,
    {
        let base = self.model.item_template.take().unwrap_or_else(|| {
            make_selection_panel_item_template(|item: &SelectionPanelItemRenderModel<'_, T>, _cx| {
                gpui::div().min_w(gpui::px(0.0)).truncate().child(item.item.label().clone())
            })
        });
        self.model.item_template = Some(item_template_with_modifier(base, modifier));
        self
    }

    pub fn template(mut self, template: Arc<dyn SelectionPanelTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::Stateful<gpui::Div>, &SelectionPanelRenderModel<'a, T>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    /// Where the panel's style comes from: a `selection_panel` binding, read each time it is drawn (#433).
    pub fn menu(mut self, menu: crate::controls::popup_menu::MenuSource) -> Self {
        self.model.menu = menu;
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SelectionPanelControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| SelectionPanelControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new(id: impl Into<SharedString>) -> SelectionPanelBuilder<SelectionPanelItem> {
    SelectionPanelBuilder::new(id)
}

pub(crate) fn default_selection_panel_model<T>(id: impl Into<SharedString>) -> SelectionPanelModel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    let id = id.into();

    SelectionPanelModel {
        panel_id: format!("{}-panel", id).into(),
        id,
        items: Vec::new(),
        visible_indices: Vec::new(),
        selected_source_index: None,
        open: true,
        enabled: true,
        scroll_interaction: ScrollInteraction::VIEWPORT,
        pointer_focus: Default::default(),
        role: SelectionPanelRole::Embedded,
        hover_activation: None,
        show_selection_marker: true,
        icons_explicit: false,
        icons: SelectionStatusIcons::default(),
        scrolling: true,
        scrollbar_visibility: ScrollbarVisibility::default(),
        scrollbar_auto_hide_activate: ScrollbarAutoHideActivate::default(),
        min_visible_rows_explicit: false,
        max_visible_rows_explicit: false,
        min_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .row_limits
            .resolve("selection_panel")
            .0,
        max_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .row_limits
            .resolve("selection_panel")
            .1,
        item_template: None,
        template: default_selection_panel_template(),
        scrollbar_template: default_scrollbar_template(),
        menu: Arc::new(|| crate::style::StyleBinding::sdk("selection_panel")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_selection_panel_template::<SelectionPanelItem>();
        let builder = SelectionPanelBuilder::new("selection-panel-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_item_template_modifier_creates_template_from_default_content() {
        let builder = SelectionPanelBuilder::<SelectionPanelItem>::new("selection-panel-test")
            .with_item_template_modifier(|content, _, _| content);

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn with_item_template_modifier_wraps_existing_item_template() {
        let item_template =
            make_selection_panel_item_template(|_item: &SelectionPanelItemRenderModel<'_, SelectionPanelItem>, _cx| {
                gpui::div()
            });
        let builder = SelectionPanelBuilder::new("selection-panel-test")
            .item_template(item_template.clone())
            .with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn template_and_item_modifiers_compose_on_builder() {
        let template = default_selection_panel_template::<SelectionPanelItem>();
        let item_template =
            make_selection_panel_item_template(|_item: &SelectionPanelItemRenderModel<'_, SelectionPanelItem>, _cx| {
                gpui::div()
            });
        let builder = SelectionPanelBuilder::new("selection-panel-test")
            .template(template.clone())
            .item_template(item_template.clone())
            .with_template_modifier(|element, _| element)
            .with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }
}
