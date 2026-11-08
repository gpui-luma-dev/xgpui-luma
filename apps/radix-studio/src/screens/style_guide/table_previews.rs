//! Static SDK table templates with shared column layout and native row sizing.
use std::sync::Arc;
use gpui::{AnyElement, App, Entity, SharedString, Window, div, prelude::*, px};
use xgpui_luma::{GridLayout, GridTrack};
use xgpui_luma::controls::{table::*, tabs::Tabs};
use xgpui_luma::infra::state::ControlFocusState;
use xgpui_luma::theme::{ControlSize, InteractionState, ListRowScale};
use xgpui_luma_look_radix::{self as radix, Accent, Gray, Look, SemanticRole};
use super::{
    matrix_grid::{centered, column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label},
    states,
};

pub(super) fn tabbed(look: &Arc<Look>, tabs: Entity<Tabs>, window: &mut Window, cx: &mut App) -> AnyElement {
    let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();
    let states = states::samples();
    let columns = match active.as_ref() {
        "colors" => 1,
        "all-sizes" => 3,
        _ => states.len(),
    };
    let mut grid = fixed_grid(equal_data_columns(112.0, 160.0, columns), 12.0, 24.0).child(empty_corner(), 0, 0);
    match active.as_ref() {
        "colors" => {
            grid = grid.child(column_header("Selected row", muted), 0, 1);
            for (row, accent) in Accent::ALL.iter().enumerate() {
                let fork = look.fork();
                fork.set_palettes(*accent, Gray::Auto);
                grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), fg), row + 1, 0).child(
                    sample(&fork, format!("color-{row}"), ControlSize::Md, Default::default(), true, window, cx),
                    row + 1,
                    1,
                );
            }
        }
        "all-sizes" => {
            grid = grid.child(row_label("Table", fg), 1, 0);
            for (col, (size, label)) in
                [(ControlSize::Sm, "Size 1"), (ControlSize::Md, "Size 2"), (ControlSize::Lg, "Size 3")]
                    .into_iter()
                    .enumerate()
            {
                grid = grid.child(column_header(label, muted), 0, col + 1).child(
                    sample(look, format!("size-{col}"), size, Default::default(), true, window, cx),
                    1,
                    col + 1,
                );
            }
        }
        _ => {
            for (col, state) in states.iter().enumerate() {
                grid = grid.child(states::header_cell(state, muted), 0, col + 1);
            }
            for (row, (label, selected)) in [("Default row", false), ("Selected row", true)].into_iter().enumerate() {
                grid = grid.child(row_label(label, fg), row + 1, 0);
                for (col, state) in states.iter().enumerate() {
                    grid = grid.child(
                        sample(look, format!("state-{row}-{col}"), ControlSize::Md, state.state, selected, window, cx),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
    }
    preview_tabbed(tabs, border, grid.into_any_element())
}
struct Record {
    name: &'static str,
}
fn columns() -> Vec<TableColumn<Record>> {
    vec![TableColumn::fill("Name", default_emphasis_column_template(|row: &Record| row.name))]
}
fn column_grid(cells: impl IntoIterator<Item = AnyElement>, padding: f32) -> AnyElement {
    let mut grid = GridLayout::new().columns([GridTrack::Star(1.0)]);
    for (index, cell) in cells.into_iter().enumerate() {
        grid = grid.child(div().w_full().min_w_0().px(px(padding)).child(cell), 0, index);
    }
    grid.into_any_element()
}
fn sample(
    look: &Look,
    name: String,
    size: ControlSize,
    state: InteractionState,
    selected: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("guide-table-{name}"));
    let theme = radix::table_theme(look);
    let shell = theme.resolve_look(!state.disabled, state.focused, size);
    let scale = ListRowScale::compute(size, &theme.metrics(), window.scale_factor());
    let row_look = theme.resolve_row_look(false, Default::default(), size, &scale);
    let row_height = default_row_height(&row_look);
    let body_height = body_rows_height(3, row_height, shell.row_divider_width);
    let shell_height = compute_shell_height(3, row_height, &shell, true);
    let columns = columns();
    let header = column_grid(
        columns.iter().map(|column| div().truncate().child(column.header().clone()).into_any_element()),
        shell.column_padding_x,
    );
    let records = [Record { name: "Olivia" }, Record { name: "Noah" }, Record { name: "Emma" }];
    let mut body = div().w_full().flex().flex_col();
    for (index, record) in records.iter().enumerate() {
        let enabled = !state.disabled && index != 2;
        let interaction = if index == 0 {
            state
        } else {
            InteractionState { disabled: !enabled, ..Default::default() }
        };
        let mut resolved = theme.resolve_row_look(selected && index == 0, interaction, size, &scale);
        // As in the SDK grid view, columns own horizontal insets and alignment.
        resolved.padding_x = 0.0;
        let model = TableRowRenderModel {
            table_id: &id,
            row: record,
            index,
            sibling_count: records.len(),
            selected: selected && index == 0,
            active: index == 0 && state.focused,
            hovered: index == 0 && state.hovered,
            pressed: index == 0 && state.pressed,
            focused: state.focused,
            focus_visible: state.focused,
            enabled,
            look: resolved,
        };
        let content =
            column_grid(columns.iter().map(|column| column.render_cell(&model, window, cx)), resolved.column_padding_x);
        let row_id = format!("{id}-row-{index}");
        body = body.child(
            render_table_row(
                (id.clone(), index),
                content,
                resolved,
                enabled,
                index > 0,
                false,
                Some(row_height + if index > 0 { shell.row_divider_width } else { 0.0 }),
            )
            .debug_selector(move || row_id.clone()),
        );
    }
    let model = TableRenderModel {
        id: &id,
        look: shell,
        row_look,
        row_count: 3,
        selection_mode: TableSelectionMode::Extended,
        scroll_mode: TableScrollMode::ScrollSmooth,
        visible_rows: Some(3),
        body_rows_height: Some(body_height),
        shell_height: Some(shell_height),
        has_header: true,
        current_page: 0,
        page_size: None,
        page_count: 1,
        selected_count: usize::from(selected),
        enabled: !state.disabled,
        size,
        focus: ControlFocusState { focused: state.focused, focus_visible: state.focused },
    };
    let selector = id.to_string();
    centered(
        div().w(px(160.0)).flex_none().child(
            radix::table_template(look)
                .render(&model, Some(header), body.into_any_element(), window, cx)
                .debug_selector(move || selector.clone()),
        ),
    )
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext};
    use xgpui_luma::theme::ThemeMode;
    struct Harness {
        look: Arc<Look>,
        tabs: Entity<Tabs>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            tabbed(&self.look, self.tabs.clone(), window, cx)
        }
    }
    #[test]
    fn table_previews_render_all_tabs_and_keep_rows_inside_native_shells() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            let tabs = super::super::state::preview_navigation("table", &look, cx);
            Harness { look, tabs }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(1400.0)));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for tab in ["template-preview", "colors", "all-sizes"] {
                cx.update(|_, app| {
                    view.update(app, |view, cx| {
                        view.look.set_mode(mode);
                        view.tabs.update(cx, |tabs, cx| tabs.set_active(tab, cx));
                        cx.notify();
                    })
                });
                cx.run_until_parked();
                if tab == "template-preview" {
                    let first = cx.debug_bounds("guide-table-state-0-0").unwrap();
                    let selected = cx.debug_bounds("guide-table-state-1-0").unwrap();
                    let disabled = cx.debug_bounds("guide-table-state-0-4").unwrap();
                    assert!(first.left() >= px(0.0));
                    assert!(disabled.right() <= px(1040.0));
                    assert_eq!(disabled.top(), first.top());
                    assert!(disabled.left() > first.right());
                    assert_eq!(first.size.width, px(160.0));
                    assert!(selected.top() > first.bottom());
                }
            }
            let mut previous_height = px(0.0);
            for (shell_id, row_id) in [
                ("guide-table-size-0", "guide-table-size-0-row-2"),
                ("guide-table-size-1", "guide-table-size-1-row-2"),
                ("guide-table-size-2", "guide-table-size-2-row-2"),
            ] {
                let shell = cx.debug_bounds(shell_id).unwrap();
                let last = cx.debug_bounds(row_id).unwrap();
                assert!(shell.size.height > previous_height);
                assert!(last.bottom() <= shell.bottom());
                assert!(last.size.width <= shell.size.width);
                assert_eq!(shell.size.width, px(160.0));
                previous_height = shell.size.height;
            }
        }
    }
}
