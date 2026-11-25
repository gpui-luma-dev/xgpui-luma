//! The SDK table on the design surface page (issue #383): the Paging Table and Scrolling Table pages. Its
//! `.lss` (`table`) styles the frame, the header and body rows: hover, pressed, focused, disabled, and
//! `selected` for selected rows. Design mode pins the picked states on every row through the table's theme;
//! `:selected` keeps the selection, and without it no row is selected. A table lays out a virtualized list,
//! so each State matrix cell is a small table of its own, drawn in that cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::pager;
use xgpui_luma::controls::table::{
    self, StyledTable, TableControl, TableEvent, TableLook, TableRowLook, TableRowPalette, TableTheme,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, ListRowScale, MetricTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The `.lss` states a row reads, and `focused` for the frame.
const STATES: [&str; 5] = ["hover", "pressed", "focused", "selected", "disabled"];
/// Row names, with ascenders and descenders.
const NAMES: [&str; 6] = [
    "Quarterly budget",
    "Typography study",
    "Shipping log",
    "Playground sketch",
    "Legacy backup",
    "Holiday photo",
];
/// Rows in the live table, and in a matrix cell's.
const ROWS: usize = 48;
const CELL_ROWS: usize = 3;

pub(super) struct Row {
    id: usize,
    name: String,
    category: &'static str,
}

fn rows(count: usize) -> Vec<Row> {
    (1..=count)
        .map(|id| Row {
            id,
            name: format!("{} {id:02}", NAMES[(id - 1) % NAMES.len()]),
            category: if id % 2 == 0 { "Spreadsheet" } else { "Image" },
        })
        .collect()
}

fn table_builder(id: impl Into<SharedString>, count: usize) -> table::TableBuilder<Row> {
    table::TableBuilder::new_typed(id)
        .items(rows(count))
        .row_label(|row: &Row| row.name.clone())
        .grid_view([
            table::TableColumn::fixed(
                "ID",
                64.,
                table::default_numeric_column_template(|row: &Row| row.id.to_string()),
            ),
            table::TableColumn::fill("Name", table::default_text_column_template(|row: &Row| row.name.clone())),
            table::TableColumn::fixed("Category", 120., table::default_text_column_template(|row: &Row| row.category)),
        ])
}

/// The table from a theme's `table` control, every row in the picked states when some are pinned.
struct PinnedTableTheme {
    base: StyledTable,
    pinned: Option<StateSet>,
}

impl PinnedTableTheme {
    fn new(theme: Theme, pinned: Option<StateSet>) -> Self {
        Self { base: StyledTable::new(StyleBinding::new("table", ThemeSource::Fixed(theme))), pinned }
    }

    /// A row's selection and state: the picked ones (`:selected` keeps the row's own), or the row's.
    fn row_states(&self, selected: bool, state: InteractionState) -> (bool, InteractionState) {
        match &self.pinned {
            Some(states) => (selected && states.has("selected"), states.interaction()),
            None => (selected, state),
        }
    }
}

impl TableTheme for PinnedTableTheme {
    fn resolve_look(&self, enabled: bool, focused: bool) -> TableLook {
        match &self.pinned {
            Some(states) => self.base.look(!states.has("disabled"), states.has("focused")),
            None => self.base.look(enabled, focused),
        }
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        let (selected, state) = self.row_states(selected, state);
        self.base.row(selected, state)
    }

    fn resolve_row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        let (selected, state) = self.row_states(selected, state);
        self.base.row_look(selected, state, scale)
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TableKind {
    Paging,
    Scrolling,
}

pub(super) struct TableSurface {
    kind: TableKind,
    /// The page's control: the paging table, or the table itself.
    view: AnyView,
    table: Entity<TableControl<Row>>,
    /// Each matrix cell's table, made with the theme last bound.
    cells: RefCell<HashMap<SharedString, Entity<TableControl<Row>>>>,
}

impl TableSurface {
    pub(super) fn make_paging(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let paging = table::PagingTableBuilder::new(
            table_builder("workshop-sdk-paging-table", ROWS).paged(8).visible_rows(8),
            pager::PagerBuilder::new("workshop-sdk-table-pager").page_size_options([8, 16, 24]),
        )
        .spawn(cx);
        let table = paging.read(cx).list().clone();
        Box::new(Self { kind: TableKind::Paging, view: paging.into(), table, cells: RefCell::default() })
    }

    pub(super) fn make_scrolling(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let table = table_builder("workshop-sdk-scrolling-table", ROWS)
            .scroll_mode(table::TableScrollMode::ScrollSmooth)
            .visible_rows(8)
            .spawn(cx);
        Box::new(Self { kind: TableKind::Scrolling, view: table.clone().into(), table, cells: RefCell::default() })
    }

    /// The SDK table, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn table(&self) -> Entity<TableControl<Row>> {
        self.table.clone()
    }
}

impl SurfaceControl for TableSurface {
    fn title(&self) -> &'static str {
        match self.kind {
            TableKind::Paging => "Paging Table",
            TableKind::Scrolling => "Scrolling Table",
        }
    }

    fn view(&self) -> AnyView {
        self.view.clone()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let styled = Arc::new(PinnedTableTheme::new(theme, pinned.cloned()));
        self.table.update(cx, |table, cx| table.set_theme(styled, cx));
        // Cells are remade in the new theme.
        self.cells.borrow_mut().clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.table.update(cx, |table, cx| table.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    /// Whether a row is selected.
    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", !self.table.read(cx).selected_indices().is_empty())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.table, move |side: &mut ControlSide, _, event: &TableEvent, cx| {
            // Enabling is the page's own doing (Design disables input); scrolling reports are noise.
            if matches!(event, TableEvent::EnabledChanged { .. } | TableEvent::ScrollChanged { .. }) {
                return;
            }
            let state = match event {
                TableEvent::RowHoverChanged { hovered, .. } => Some(("hover", *hovered)),
                TableEvent::FocusChanged { focused } => Some(("focused", *focused)),
                TableEvent::SelectionChanged { selected_indices } => Some(("selected", !selected_indices.is_empty())),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// A three-row table in `states`, its second row selected when `:selected` is.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let table = self
            .cells
            .borrow_mut()
            .entry(id.clone())
            .or_insert_with(|| {
                table_builder(id, CELL_ROWS)
                    .selected_index(1)
                    .visible_rows(CELL_ROWS)
                    .enabled(false)
                    .theme(Arc::new(PinnedTableTheme::new(theme.clone(), Some(states.clone()))))
                    .spawn(cx)
            })
            .clone();
        div().w(px(360.)).child(table).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_table_pages_select_rows_and_pin_their_states() {
        for (id, last) in [("paging-table", 47), ("scrolling-table", 47)] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            let table = cx.update(|_, cx| side.read(cx).control_as::<TableSurface>().expect("a table").table());

            cx.update(|window, cx| table.read(cx).focus_handle(cx).focus(window, cx));
            cx.run_until_parked();
            cx.simulate_keystrokes("end space");
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(table.read(cx).selected_indices(), [last], "{id}"));
            assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "{id} follows the selection");

            side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
            side.update(cx, |side, cx| side.choose_interaction("hover", cx));
            cx.run_until_parked();
            let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
            assert!(pinned.has("hover") && pinned.has("selected"), "{id}: {pinned:?}");

            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "{id}'s matrix draws tables");
        }
    }
}
