//! Table selection and row identity.
//!
//! `Single` (the default) selects one row. `Multiple` independently toggles rows
//! on click or Space/Enter. `Extended` replaces selection on plain click or
//! Space/Enter, toggles with Ctrl/Cmd, and extends from a fixed anchor with Shift.
//! Ctrl/Cmd+Shift adds a range. `None` retains navigation without selection.
//! Arrows and Home/End move the active row without changing selection;
//! Shift+arrows/Home/End selects ranges in Extended mode. Ctrl/Cmd+A selects all
//! enabled rows across pages; adding Shift clears selection in both multi modes.
//! Ranges follow current row order, skip disabled rows, and can cross pages.
//!
//! Controls own their selection by default. Owners can replace it with
//! [`TableControl::set_selected_indices`] or [`TableControl::set_selected_keys`]
//! without moving focus, the active row, or scroll position. These setters reset
//! the range anchor only for different membership and emit changes once. This permits
//! subscribing to events and feeding owner state back without an event loop.
//!
//! After spawning, configure [`TableControl::set_row_key`] before replacing or
//! sorting rows. Keys must be unique and stable for a record. [`TableControl::set_items`]
//! returns an error for duplicate keys; successful replacements retain selected,
//! active and anchor identities, dropping removed/disabled selections. Unkeyed
//! tables retain positional selection and cannot preserve identity across sorting.
//! Selection indices are returned in row order. The legacy index event can fire
//! on a reorder; [`TableEvent::SelectedKeysChanged`] fires only for different key
//! membership. Mode changes and row replacement also emit reconciliation events.
//!
//! [`TableSelectionMode::Extended`] and the fallible `set_items` return value are
//! additions to the original API; exhaustive mode matches and row replacement
//! callers should handle them.

//! # Row reordering
//!
//! After `set_row_key`, opt in with `set_row_reordering(true, cx)`. Dragging a
//! selected row moves all selected rows in dataset order; dragging an unselected
//! row moves only that row and preserves the selection. Disabled rows cannot be
//! dragged. Before/after gaps are resolved against current keys. Unchanged gaps
//! show invalid feedback and complete without a mutation. Escape or release
//! outside a gap cancels. Replacing items/keys or disabling reordering invalidates
//! the captured session. Keys must remain stable for the lifetime of each record.
//!
//! The default drop commits synchronously. `set_row_drop_handler` lets an owner
//! call `commit_row_drop` or `reject_row_drop` synchronously; validation runs again
//! at commit. `TableEvent::RowDrag` exposes the shared start, reorder, drop/reject
//! and end lifecycle. Read `row_keys()` / `items()` after `ItemsReordered` to persist
//! the final order. Row data does not need to implement `Clone`.
//!
//! Scrolling and virtualized tables support edge auto-scroll. Paged tables use
//! visible-page gaps in the full dataset (including selected rows on other pages)
//! without switching pages during the drag. Table has no native sort/filter/group
//! projection: owners must disable reordering while an external transformation
//! makes displayed order non-authoritative. Cross-table and hierarchical moves
//! are unsupported. Wrap nested interactive cells with
//! [`crate::infra::drag_drop::DragDropElementExt::drag_boundary`] so their clicks
//! and drags do not arm the row. Themes own preview and valid/invalid marker colors.

//! # Manual column resizing
//!
//! Opt in with [`TableBuilder::column_resizing`] (also available on the paging
//! and Shadcn builders). Drag a grid header's right divider to resize that column
//! and its neighbor, preserving their combined width. Both must be resizable;
//! [`TableColumn::resizable`] can lock action columns. Fixed control columns are
//! locked by default. Dragging cannot shrink a column below 48px, or below its
//! starting width if already narrower. Fill columns continue sharing available
//! space when the viewport changes. Widths live on the table for its lifetime.
//!
//! Escape, focus loss, disabling the table, or a viewport change during a drag
//! restores the original widths. Mouse release commits, including outside the
//! table. Custom header templates own their interactions and get no handles.

mod styled;
pub use styled::{StyledTable, StyledTableTheme, table_drag_look};
mod control;
mod column_template;
mod layout;
mod macros;
mod model;
mod paging;
mod row;
mod template;
pub(crate) mod theme;

pub use control::{TableControl, TableEvent, TableSelectionError, TableReorderError, TableRowDrop, TableRowDragEvent};
pub use paging::{PagingTable, PagingTableControl, PagingTableBuilder};
pub use column_template::{
    TableColumnRenderModel, column_template_with_modifier, default_emphasis_column_template,
    default_muted_column_template, default_numeric_column_template, default_text_column_template,
};
pub use layout::{
    body_rows_height, compute_shell_height, default_row_height, distributed_row_height, effective_visible_rows,
    page_after_size_change, page_count, rows_for_viewport_height, rows_for_viewport_height_border_box,
    visible_row_height,
};
pub use model::{
    IntoTableColumnCellTemplate, TableScrollMode, TableSelectionMode, TableLookOverride, TableBuilder, TableColumn,
    TableColumnCellLayout, TableColumnCellTemplate, TableColumnWidth, TableEnabledFn, TableHeaderTemplate,
    TableGridColumnsBuilder, TableLabel, TableLabelFn, TableRenderModel, TableRowRenderModel, TableRowTemplate,
    make_table_header_template, make_table_row_template,
};
pub use row::render_table_row;

pub use template::{
    DefaultTableShellTemplate, TableTemplate, TableTemplateModifier, default_table_template, table_template_with_theme,
};
pub use theme::{
    DefaultTableTheme, TableDragLook, TableLook, TableRowLook, TableRowPalette, TableTheme, default_table_theme,
};

use gpui::{Entity, SharedString};

pub type Table<T> = Entity<TableControl<T>>;

pub fn new(id: impl Into<SharedString>) -> TableBuilder<TableLabel> {
    TableBuilder::new(id)
}

pub fn new_typed<T>(id: impl Into<SharedString>) -> TableBuilder<T>
where
    T: 'static,
{
    TableBuilder::new_typed(id)
}

mod callback;
pub use callback::TableEventExt;

pub use model::TableMotion;
