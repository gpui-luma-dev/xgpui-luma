use crate::infra::locale::{LayoutDirection, LocaleContext, LogicalStyled};
crate::motion_profile!(TableMotion => table {
    drop_layout: crate::motion::Transition,
});

use crate::interaction::{ScrollInteraction, WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, ListAlignment, SharedString, Stateful, Window, div, prelude::*, px};

use super::control::TableControl;
use super::template::{TableTemplate, default_table_template, modified_table_template};
use super::theme::{TableLook, TableRowLook, TableTheme, default_table_theme};
use crate::infra::state::ControlFocusState;
use crate::theme::ControlSize;

pub type TableLabelFn<T> = Arc<dyn Fn(&T) -> SharedString + Send + Sync + 'static>;
pub type TableEnabledFn<T> = Arc<dyn Fn(&T) -> bool + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct TableLabel {
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl TableLabel {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TableSelectionMode {
    None,
    #[default]
    Single,
    /// Each click or Space independently toggles a row.
    Multiple,
    /// Plain selection replaces; Ctrl/Cmd toggles; Shift extends a range.
    Extended,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TableScrollMode {
    /// Smooth pixel-by-pixel scrolling (default).
    #[default]
    ScrollSmooth,
    /// Smooth scrolling with snap-to-row boundaries on release.
    ScrollSnap,
    /// Paginated layout displaying N items at a time with page selectors.
    Paged { page_size: usize },
}

impl TableScrollMode {
    pub fn is_paged(self) -> bool {
        matches!(self, Self::Paged { .. })
    }
}

pub struct TableRenderModel<'a> {
    pub id: &'a SharedString,
    pub look: TableLook,
    pub row_look: TableRowLook,
    pub row_count: usize,
    pub selection_mode: TableSelectionMode,
    pub scroll_mode: TableScrollMode,
    pub visible_rows: Option<usize>,
    pub body_rows_height: Option<f32>,
    pub shell_height: Option<f32>,
    pub has_header: bool,
    pub current_page: usize,
    pub page_size: Option<usize>,
    pub page_count: usize,
    pub selected_count: usize,
    pub enabled: bool,
    pub size: ControlSize,
    pub focus: ControlFocusState,
}

pub type TableLookOverride = Arc<dyn Fn(TableLook) -> TableLook + Send + Sync + 'static>;

pub type TableHeaderTemplate =
    Arc<dyn for<'a> Fn(&TableRenderModel<'a>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_table_header_template<F, E>(template: F) -> TableHeaderTemplate
where
    F: for<'a> Fn(&TableRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, window, cx| template(model, window, cx).into_any_element())
}

#[derive(Clone, Debug)]
pub struct TableRowRenderModel<'a, T>
where
    T: 'static,
{
    pub table_id: &'a SharedString,
    pub row: &'a T,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub look: TableRowLook,
}

pub type TableRowTemplate<T> = Arc<
    dyn for<'a> Fn(&TableRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

pub fn make_table_row_template<T, F, E>(template: F) -> TableRowTemplate<T>
where
    T: 'static,
    F: for<'a> Fn(&TableRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, cells, window, cx| template(model, cells, window, cx).into_any_element())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TableColumnWidth {
    Fixed(f32),
    #[default]
    Fill,
}

/// Horizontal padding and overflow behavior for a grid column cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableColumnCellLayout {
    /// Text columns: horizontal inset and ellipsis overflow.
    #[default]
    Text,
    /// Embedded controls (checkbox, icon button): centered, no truncate.
    Control,
}

pub type TableColumnCellTemplate<T> =
    Arc<dyn for<'a> Fn(&TableRowRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub trait IntoTableColumnCellTemplate<T> {
    fn into_cell_template(self) -> TableColumnCellTemplate<T>;
}

impl<T> IntoTableColumnCellTemplate<T> for TableColumnCellTemplate<T>
where
    T: 'static,
{
    fn into_cell_template(self) -> TableColumnCellTemplate<T> {
        self
    }
}

impl<T, F, E> IntoTableColumnCellTemplate<T> for F
where
    T: 'static,
    F: Fn(&T) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    fn into_cell_template(self) -> TableColumnCellTemplate<T> {
        Arc::new(move |model, _window, _cx| (self)(model.row).into_any_element())
    }
}

pub struct TableColumn<T>
where
    T: 'static,
{
    header: SharedString,
    pub(super) width: TableColumnWidth,
    pub(super) fill_weight: f32,
    pub(super) resizable: bool,
    cell_layout: TableColumnCellLayout,
    cell_template: TableColumnCellTemplate<T>,
}

impl<T> Clone for TableColumn<T>
where
    T: 'static,
{
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            width: self.width,
            fill_weight: self.fill_weight,
            resizable: self.resizable,
            cell_layout: self.cell_layout,
            cell_template: self.cell_template.clone(),
        }
    }
}

impl<T> TableColumn<T>
where
    T: 'static,
{
    pub fn new(
        header: impl Into<SharedString>,
        width: TableColumnWidth,
        cell_template: TableColumnCellTemplate<T>,
    ) -> Self {
        Self {
            header: header.into(),
            width,
            fill_weight: 1.0,
            resizable: true,
            cell_layout: TableColumnCellLayout::Text,
            cell_template,
        }
    }

    pub fn fixed(
        header: impl Into<SharedString>,
        width: f32,
        cell_template: impl IntoTableColumnCellTemplate<T>,
    ) -> Self {
        Self::fixed_with_layout(header, width, TableColumnCellLayout::Text, cell_template)
    }

    /// Fixed-width column for embedded controls (checkboxes, icon buttons).
    pub fn fixed_control(
        header: impl Into<SharedString>,
        width: f32,
        cell_template: impl IntoTableColumnCellTemplate<T>,
    ) -> Self {
        Self::fixed_with_layout(header, width, TableColumnCellLayout::Control, cell_template)
    }

    pub fn fixed_with_layout(
        header: impl Into<SharedString>,
        width: f32,
        cell_layout: TableColumnCellLayout,
        cell_template: impl IntoTableColumnCellTemplate<T>,
    ) -> Self {
        Self {
            header: header.into(),
            width: TableColumnWidth::Fixed(width),
            fill_weight: 1.0,
            resizable: cell_layout != TableColumnCellLayout::Control,
            cell_layout,
            cell_template: cell_template.into_cell_template(),
        }
    }

    pub fn fill(header: impl Into<SharedString>, cell_template: impl IntoTableColumnCellTemplate<T>) -> Self {
        Self {
            header: header.into(),
            width: TableColumnWidth::Fill,
            fill_weight: 1.0,
            resizable: true,
            cell_layout: TableColumnCellLayout::Text,
            cell_template: cell_template.into_cell_template(),
        }
    }

    /// Allow this column to participate in adjacent-column resizing.
    /// Text columns default to true; fixed control columns default to false.
    /// The table must also opt in with `column_resizing(true)`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn cell_layout(&self) -> TableColumnCellLayout {
        self.cell_layout
    }

    pub fn header(&self) -> &SharedString {
        &self.header
    }

    pub fn width(&self) -> TableColumnWidth {
        self.width
    }

    pub fn render_cell(&self, model: &TableRowRenderModel<'_, T>, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.cell_template)(model, window, cx)
    }
}

#[derive(Clone)]
pub struct TableModel<T>
where
    T: 'static,
{
    pub(crate) motion: crate::motion::MotionConfig<TableMotion>,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) selected_indices: Vec<usize>,
    pub(crate) active_index: Option<usize>,
    pub(crate) selection_mode: TableSelectionMode,
    /// When `false`, row clicks update the active row only; keyboard and owner selection remain available.
    pub(crate) select_on_row_click: bool,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: ScrollInteraction,
    pub(crate) pointer_focus: crate::interaction::PointerFocusPolicy,
    pub(crate) size: ControlSize,
    pub(crate) alignment: ListAlignment,
    pub(crate) overdraw: f32,
    pub(crate) row_label: TableLabelFn<T>,
    pub(crate) row_enabled: TableEnabledFn<T>,
    pub(crate) template: Arc<dyn TableTemplate>,
    pub(crate) header_template: Option<TableHeaderTemplate>,
    pub(crate) grid_header: bool,
    pub(crate) column_resizing: bool,
    pub(crate) row_template: Option<TableRowTemplate<T>>,
    pub(crate) theme: Arc<dyn TableTheme>,
    pub(crate) look_override: Option<TableLookOverride>,
    pub(crate) columns: Vec<TableColumn<T>>,
    pub(crate) scroll_mode: TableScrollMode,
    pub(crate) visible_rows: Option<usize>,
    pub(crate) visible_row_height: Option<f32>,
    /// When true, the shell fills parent height (`h_full`) instead of sizing from
    /// `visible_rows` / paged `page_size`. Explicit `visible_rows` still wins when set.
    pub(crate) fill_height: bool,
}

impl<T> TableModel<T>
where
    T: 'static,
{
    pub(crate) fn label_for_row(&self, row: &T) -> SharedString {
        (self.row_label)(row)
    }

    pub(crate) fn row_is_enabled(&self, row: &T) -> bool {
        (self.row_enabled)(row)
    }
}

pub struct TableBuilder<T>
where
    T: 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::table::TableEvent>,

    pub(crate) model: TableModel<T>,
}

pub struct TableGridColumnsBuilder<T>
where
    T: 'static,
{
    builder: TableBuilder<T>,
    columns: Vec<TableColumn<T>>,
}

impl TableBuilder<TableLabel> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id).row_label(|row| row.label.clone()).row_enabled(|row| row.enabled)
    }
}

impl<T> TableBuilder<T>
where
    T: 'static,
{
    crate::motion_methods!(TableMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe every drag/drop lifecycle phase. Alias of `on_row_drag`.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_row_drag(cx, callback)
    }

    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } = event
            {
                callback(owner, *source, *target, keys, before, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            crate::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `TableEvent::RowDrag` through the existing event stream.
    pub fn on_row_drag<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::RowDrag(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectionChanged` through the existing event stream.
    pub fn on_selection_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::SelectionChanged { selected_indices } = event {
                callback(owner, selected_indices, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectedKeysChanged` through the existing event stream.
    pub fn on_selected_keys_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::SelectedKeysChanged { selected_keys } = event {
                callback(owner, selected_keys, cx);
            }
        })
    }

    /// Handle `TableEvent::ActiveIndexChanged` through the existing event stream.
    pub fn on_active_index_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::ActiveIndexChanged { active_index } = event {
                callback(owner, active_index, cx);
            }
        })
    }

    /// Handle `TableEvent::PageChanged` through the existing event stream.
    pub fn on_page_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::PageChanged { page } = event {
                callback(owner, *page, cx);
            }
        })
    }

    /// Handle `TableEvent::PageSizeChanged` through the existing event stream.
    pub fn on_page_size_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::PageSizeChanged { page_size } = event {
                callback(owner, *page_size, cx);
            }
        })
    }

    /// Handle `TableEvent::ScrollChanged` through the existing event stream.
    pub fn on_scroll_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TableEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TableEvent::RowHoverChanged` through the existing event stream.
    pub fn on_row_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::RowHoverChanged { index, hovered } = event {
                callback(owner, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TableEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: TableModel {
                motion: Default::default(),
                id: id.into(),
                items: Vec::new(),
                selected_indices: Vec::new(),
                active_index: None,
                selection_mode: TableSelectionMode::Single,
                select_on_row_click: true,
                enabled: true,
                scroll_interaction: ScrollInteraction::VIEWPORT,
                pointer_focus: Default::default(),
                size: ControlSize::Md,
                alignment: ListAlignment::Top,
                overdraw: 480.0,
                row_label: Arc::new(|_| SharedString::default()),
                row_enabled: Arc::new(|_| true),
                template: default_table_template(),
                header_template: None,
                grid_header: false,
                column_resizing: false,
                row_template: None,
                theme: default_table_theme(),
                look_override: None,
                columns: Vec::new(),
                scroll_mode: TableScrollMode::ScrollSmooth,
                visible_rows: None,
                visible_row_height: None,
                fill_height: false,
            },
        }
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, selection_mode: TableSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn single(mut self) -> Self {
        self.model.selection_mode = TableSelectionMode::Single;
        self
    }

    pub fn multiple(mut self) -> Self {
        self.model.selection_mode = TableSelectionMode::Multiple;
        self
    }

    pub fn extended(mut self) -> Self {
        self.model.selection_mode = TableSelectionMode::Extended;
        self
    }

    pub fn no_selection(mut self) -> Self {
        self.model.selection_mode = TableSelectionMode::None;
        self
    }

    /// Data-table style: row clicks only move the active row. Selection remains
    /// available through the keyboard and embedded controls (e.g. row checkboxes).
    pub fn select_on_row_click(mut self, select_on_row_click: bool) -> Self {
        self.model.select_on_row_click = select_on_row_click;
        self
    }

    pub fn selected_index(mut self, selected_index: usize) -> Self {
        self.model.selected_indices = vec![selected_index];
        self
    }

    pub fn selected_indices(mut self, selected_indices: impl IntoIterator<Item = usize>) -> Self {
        self.model.selected_indices = selected_indices.into_iter().collect();
        self
    }

    pub fn active_index(mut self, active_index: usize) -> Self {
        self.model.active_index = Some(active_index);
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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn alignment(mut self, alignment: ListAlignment) -> Self {
        self.model.alignment = alignment;
        self
    }

    pub fn overdraw(mut self, overdraw: impl Into<f64>) -> Self {
        self.model.overdraw = overdraw.into().max(0.0) as f32;
        self
    }

    pub fn scroll_mode(mut self, mode: TableScrollMode) -> Self {
        self.model.scroll_mode = mode;
        self
    }

    pub fn paged(self, page_size: usize) -> Self {
        self.scroll_mode(TableScrollMode::Paged { page_size: page_size.max(1) })
    }

    pub fn scroll_snap(self, enabled: bool) -> Self {
        if enabled {
            self.scroll_mode(TableScrollMode::ScrollSnap)
        } else {
            self.scroll_mode(TableScrollMode::ScrollSmooth)
        }
    }

    pub fn visible_rows(mut self, count: usize) -> Self {
        self.model.visible_rows = Some(count.max(1));
        self
    }

    /// Fill the parent height instead of using a content-sized shell.
    ///
    /// Intended for pane/split layouts. The shell uses `h_full` unless [`Self::visible_rows`]
    /// is also set. For paged lists, `page_size` is recalculated from the tableport on
    /// resize and row heights are stretched so the page fills the viewport flush (pager updates
    /// with the new page size).
    pub fn fill_height(mut self) -> Self {
        self.model.fill_height = true;
        self
    }

    /// Fix row border-box height, clipping overflow, and size the visible-row shell.
    ///
    /// Fixed rows seed offscreen height metadata instead of rendering every row
    /// for measurement. Use for uniform data-table rows, including custom content.
    /// Fill-height paged tables still stretch rows to the available page height.
    pub fn visible_row_height(mut self, height: impl Into<f32>) -> Self {
        self.model.visible_row_height = Some(height.into().max(1.0));
        self
    }

    pub fn row_label<F, S>(mut self, label: F) -> Self
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        self.model.row_label = Arc::new(move |row| label(row).into());
        self
    }

    pub fn row_enabled<F>(mut self, enabled: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.model.row_enabled = Arc::new(enabled);
        self
    }

    pub fn template(mut self, template: Arc<dyn TableTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn theme(mut self, theme: Arc<dyn TableTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn look_override<F>(mut self, override_fn: F) -> Self
    where
        F: Fn(TableLook) -> TableLook + Send + Sync + 'static,
    {
        self.model.look_override = Some(Arc::new(override_fn));
        self
    }

    pub fn square_corners(self) -> Self {
        self.look_override(|mut look| {
            look.radius = 0.0;
            look
        })
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TableRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = modified_table_template(std::sync::Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn header_template(mut self, header_template: TableHeaderTemplate) -> Self {
        self.model.grid_header = false;
        self.model.header_template = Some(header_template);
        self
    }

    pub fn with_header_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&TableRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.grid_header = false;
        self.model.header_template = Some(make_table_header_template(template));
        self
    }

    pub fn row_template(mut self, row_template: TableRowTemplate<T>) -> Self {
        self.model.row_template = Some(row_template);
        self
    }

    pub fn with_row_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&TableRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.row_template = Some(make_table_row_template(template));
        self
    }

    pub fn grid_view(mut self, columns: impl IntoIterator<Item = TableColumn<T>>) -> Self {
        self.model.columns = columns.into_iter().collect();
        self.model.grid_header = true;
        self.model.header_template = None;
        self
    }

    /// Enable manual resizing between adjacent resizable grid columns.
    /// Dragging preserves their combined width and keeps fill columns flexible.
    /// Custom header templates do not receive resize handles. Defaults to false.
    pub fn column_resizing(mut self, enabled: bool) -> Self {
        self.model.column_resizing = enabled;
        self
    }

    pub fn grid_columns(self) -> TableGridColumnsBuilder<T> {
        TableGridColumnsBuilder { builder: self, columns: Vec::new() }
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<TableControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| TableControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

impl<T> TableGridColumnsBuilder<T>
where
    T: 'static,
{
    pub fn column_fixed<F, E>(mut self, header: impl Into<SharedString>, width: f32, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.columns.push(TableColumn::fixed(header, width, cell_template));
        self
    }

    pub fn column_fill<F, E>(mut self, header: impl Into<SharedString>, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.columns.push(TableColumn::fill(header, cell_template));
        self
    }

    pub fn finish(self) -> TableBuilder<T> {
        self.builder.grid_view(self.columns)
    }
}

pub(crate) fn render_grid_view_cells<T>(
    model: &TableRowRenderModel<'_, T>,
    columns: &[TableColumn<T>],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement
where
    T: 'static,
{
    let direction = LocaleContext::current(cx).direction;
    let mut cells = div()
        .w_full()
        .flex()
        .flex_row_logical(direction)
        .items_center()
        .when(direction == LayoutDirection::Rtl, |cells| cells.text_right());

    for column in columns {
        cells = cells.child(render_grid_view_column_slot(
            column.width(),
            column.fill_weight,
            column.cell_layout(),
            model.look.column_padding_x,
            column.render_cell(model, window, cx),
        ));
    }

    cells.into_any_element()
}

fn render_grid_view_column_slot(
    width: TableColumnWidth,
    fill_weight: f32,
    cell_layout: TableColumnCellLayout,
    padding_x: f32,
    content: AnyElement,
) -> AnyElement {
    match (width, cell_layout) {
        (TableColumnWidth::Fixed(width), TableColumnCellLayout::Control) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (TableColumnWidth::Fixed(width), TableColumnCellLayout::Text) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .child(div().w_full().min_w(px(0.0)).px(px(padding_x)).truncate().child(content))
            .into_any_element(),
        (TableColumnWidth::Fill, TableColumnCellLayout::Control) => div()
            .flex_1()
            .flex_grow(fill_weight)
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (TableColumnWidth::Fill, TableColumnCellLayout::Text) => div()
            .flex_1()
            .flex_grow(fill_weight)
            .min_w(px(0.0))
            .child(div().w_full().min_w(px(0.0)).px(px(padding_x)).truncate().child(content))
            .into_any_element(),
    }
}

pub(super) fn render_grid_view_header_column_slot(
    width: TableColumnWidth,
    fill_weight: f32,
    cell_layout: TableColumnCellLayout,
    padding_x: f32,
    content: AnyElement,
) -> Div {
    match (width, cell_layout) {
        (TableColumnWidth::Fixed(width), TableColumnCellLayout::Control) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(content),
        (TableColumnWidth::Fixed(width), TableColumnCellLayout::Text) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(grid_view_header_label_slot(content, padding_x)),
        (TableColumnWidth::Fill, TableColumnCellLayout::Control) => div()
            .flex_1()
            .flex_grow(fill_weight)
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(content),
        (TableColumnWidth::Fill, TableColumnCellLayout::Text) => div()
            .flex_1()
            .flex_grow(fill_weight)
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(grid_view_header_label_slot(content, padding_x)),
    }
}

fn grid_view_header_label_slot(content: AnyElement, padding_x: f32) -> impl IntoElement {
    div()
        .w_full()
        .min_w(px(0.0))
        .px(px(padding_x))
        .flex()
        .items_center()
        .justify_center()
        .truncate()
        .child(content)
}
