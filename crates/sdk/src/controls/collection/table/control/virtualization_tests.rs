//! Fixed-height measurement, positioning and variable-height fallback regressions.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use gpui::{prelude::*, Context, Entity, Render, TestAppContext, VisualTestContext, Window, div, point, px};
use super::*;
use crate::controls::table::{TableColumn, DefaultTableTheme};

struct Page {
    table: Entity<TableControl<usize>>,
    width: f32,
    height: f32,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(self.width)).h(px(self.height)).child(self.table.clone())
    }
}
fn setup(
    app: &mut TestAppContext,
    count: usize,
    fixed: bool,
    fill: bool,
    paged: bool,
) -> (Entity<Page>, &mut VisualTestContext, Entity<TableControl<usize>>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let count_calls = calls.clone();
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let mut builder = TableBuilder::new_typed("height-table")
            .items(0..count)
            .row_label(|id| id.to_string())
            .column_resizing(true)
            .grid_view([
                TableColumn::fixed("ID", 200.0, move |id: &usize| {
                    count_calls.fetch_add(1, Ordering::Relaxed);
                    div().child(id.to_string())
                }),
                TableColumn::fill("Value", |id: &usize| div().child(format!("Value {id}"))),
            ]);
        if fill {
            builder = builder.fill_height();
        }
        if fixed {
            builder = builder.visible_row_height(36.0_f32);
        } else {
            builder = builder.with_row_template(|row, cells, _, _| {
                div().h(px(if row.index.is_multiple_of(2) { 20.0 } else { 60.0 })).child(cells)
            });
        }
        if paged {
            builder = builder.paged(5);
        }
        let table = builder.spawn(cx);
        table.update(cx, |table, cx| {
            table.set_row_key(|id| id.to_string(), cx).unwrap();
            table.set_selected_keys(["0"], cx).unwrap();
        });
        Page { table, width: 600.0, height: 320.0 }
    });
    settle(cx);
    let table = cx.update(|_, app| page.read(app).table.clone());
    (page, cx, table, calls)
}
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, app| window.simulate_next_frame(app));
    }
    cx.run_until_parked();
}
fn fixed_extents(table: &Entity<TableControl<usize>>, cx: &mut VisualTestContext, count: usize) {
    cx.update(|_, app| {
        let table = table.read(app);
        let viewport = table.list_state.viewport_bounds().size.height.as_f32();
        let expected = (count as f32 * 36.0 - viewport).max(0.0);
        assert!(
            (table.list_state.max_offset_for_scrollbar().y.as_f32() - expected).abs() < 1.0,
            "fixed-height scrollbar extent"
        );
    });
}
fn visible_last(cx: &mut VisualTestContext, count: usize, height: f32) {
    let selector = match count {
        1000 => "table-motion-999",
        10_000 => "table-motion-9999",
        10_001 => "table-motion-10000",
        _ => unreachable!("fixture size"),
    };
    let bounds = cx.debug_bounds(selector).expect("distant target must be prepainted");
    assert!(bounds.top() >= px(0.0) && bounds.bottom() <= px(height + 1.0));
}

#[test]
fn fixed_height_startup_replacement_and_viewport_resize_stay_virtualized() {
    let mut app = TestAppContext::single();
    let (page, cx, table, calls) = setup(&mut app, 10_000, true, true, false);
    assert!(calls.load(Ordering::Relaxed) < 200, "startup must not build every row");
    fixed_extents(&table, cx, 10_000);
    table.update(cx, |table, cx| table.scroll_to_row(9999, cx));
    settle(cx);
    visible_last(cx, 10_000, 320.0);
    let top = cx.update(|_, app| table.read(app).list_state.logical_scroll_top().item_ix);
    calls.store(0, Ordering::Relaxed);
    table.update(cx, |table, cx| table.set_items(0..10_000, cx).unwrap());
    settle(cx);
    assert!(calls.load(Ordering::Relaxed) < 200);
    cx.update(|_, app| {
        let table = table.read(app);
        assert_eq!(table.selected_keys(), [SharedString::from("0")]);
        assert_eq!(table.list_state.logical_scroll_top().item_ix, top);
    });
    calls.store(0, Ordering::Relaxed);
    page.update(cx, |page, cx| {
        page.width = 480.0;
        page.height = 400.0;
        cx.notify();
    });
    settle(cx);
    assert!(calls.load(Ordering::Relaxed) < 200);
    fixed_extents(&table, cx, 10_000);
    table.update(cx, |table, cx| table.scroll_to_row(9999, cx));
    settle(cx);
    visible_last(cx, 10_000, 400.0);
    table.update(cx, |table, cx| table.set_items(0..10_001, cx).unwrap());
    settle(cx);
    fixed_extents(&table, cx, 10_001);
    table.update(cx, |table, cx| table.scroll_to_row(10_000, cx));
    settle(cx);
    visible_last(cx, 10_001, 400.0);
    app.quit();
    app.run_until_parked();
}

#[test]
fn fixed_height_column_drag_size_theme_and_custom_content_keep_extents() {
    let mut app = TestAppContext::single();
    let (_, cx, table, calls) = setup(&mut app, 10_000, true, true, false);
    let old = cx.debug_bounds("table-column-0").unwrap().size.width;
    let start = cx.debug_bounds("table-column-resize-0").unwrap().center();
    let end = start + point(px(24.0), px(0.0));
    calls.store(0, Ordering::Relaxed);
    cx.simulate_mouse_down(start, gpui::MouseButton::Left, Default::default());
    cx.simulate_mouse_move(start + point(px(10.0), px(0.0)), gpui::MouseButton::Left, Default::default());
    cx.simulate_mouse_move(end, gpui::MouseButton::Left, Default::default());
    cx.simulate_mouse_up(end, gpui::MouseButton::Left, Default::default());
    settle(cx);
    assert!((cx.debug_bounds("table-column-0").unwrap().size.width - old - px(24.0)).abs() < px(1.0));
    assert!(calls.load(Ordering::Relaxed) < 200);
    fixed_extents(&table, cx, 10_000);
    calls.store(0, Ordering::Relaxed);
    table.update(cx, |table, cx| {
        table.set_theme(Arc::new(DefaultTableTheme::default()), cx);
        table.set_row_template_fn(|_, cells, _, _| div().h(px(80.0)).child(cells), cx);
    });
    settle(cx);
    assert!(calls.load(Ordering::Relaxed) < 200);
    fixed_extents(&table, cx, 10_000);
    assert_eq!(cx.debug_bounds("table-motion-0").unwrap().size.height, px(36.0));
    app.quit();
    app.run_until_parked();
}

#[test]
fn variable_height_rows_keep_exact_measured_fallback() {
    let mut app = TestAppContext::single();
    let (_, cx, table, calls) = setup(&mut app, 1000, false, true, false);
    assert!(calls.load(Ordering::Relaxed) >= 1000);
    cx.update(|_, app| {
        let table = table.read(app);
        let a = table.list_state.bounds_for_item(0).unwrap();
        let b = table.list_state.bounds_for_item(1).unwrap();
        assert!(b.size.height > a.size.height + px(30.0));
    });
    table.update(cx, |table, cx| table.scroll_to_row(999, cx));
    settle(cx);
    visible_last(cx, 1000, 320.0);
    app.quit();
    app.run_until_parked();
}

#[test]
fn auto_height_paging_and_switching_scroll_mode_refresh_policy() {
    let mut app = TestAppContext::single();
    let (_, cx, table, _) = setup(&mut app, 1000, true, true, true);
    cx.update(|_, app| {
        let table = table.read(app);
        assert!(table.uniform_row_height.is_none());
        assert!(table.fill_row_height.is_some());
        assert_eq!(table.list_state.item_count(), table.page_size().unwrap());
    });
    table.update(cx, |table, cx| table.set_scroll_mode(TableScrollMode::ScrollSmooth, cx));
    settle(cx);
    fixed_extents(&table, cx, 1000);
    cx.update(|_, app| assert!(table.read(app).fill_row_height.is_none()));
    table.update(cx, |table, cx| table.scroll_to_row(999, cx));
    settle(cx);
    visible_last(cx, 1000, 320.0);
    table.update(cx, |table, cx| table.set_scroll_mode(TableScrollMode::Paged { page_size: 5 }, cx));
    settle(cx);
    cx.update(|_, app| {
        let table = table.read(app);
        assert!(table.uniform_row_height.is_none());
        assert!(table.fill_row_height.is_some());
    });
    app.quit();
    app.run_until_parked();
}

#[test]
fn unmeasured_default_rows_support_distant_reveal_and_keyboard_navigation() {
    let mut app = TestAppContext::single();
    let (_, cx, table, _) = setup(&mut app, 10_000, false, false, false);
    table.update(cx, |table, cx| table.scroll_to_row(9999, cx));
    settle(cx);
    visible_last(cx, 10_000, 320.0);
    table.update(cx, |table, cx| table.set_active_index(Some(0), cx));
    settle(cx);
    cx.update(|window, app| table.read(app).focus_handle.clone().focus(window, app));
    cx.simulate_keystrokes("end");
    settle(cx);
    visible_last(cx, 10_000, 320.0);
    cx.update(|_, app| assert_eq!(table.read(app).active_index(), Some(9999)));
    app.quit();
    app.run_until_parked();
}

#[test]
fn fixed_height_keyboard_reveal_scrolls_minimally_at_the_viewport_edge() {
    let mut app = TestAppContext::single();
    let (_, cx, table, _) = setup(&mut app, 1000, true, true, false);
    let last = cx.update(|_, app| table.read(app).visible_row_capacity() - 1);
    table.update(cx, |table, cx| table.set_active_index(Some(last), cx));
    settle(cx);
    cx.update(|window, app| table.read(app).focus_handle.clone().focus(window, app));
    cx.simulate_keystrokes("down");
    settle(cx);
    cx.update(|_, app| {
        let table = table.read(app);
        assert_eq!(table.active_index(), Some(last + 1));
        assert!(
            table.list_state.logical_scroll_top().item_ix <= 1,
            "one-row navigation must not jump a full viewport"
        );
        let row = table.list_state.bounds_for_item(last + 1).unwrap();
        assert!(row.bottom() <= table.list_state.viewport_bounds().bottom() + px(1.0));
    });
    app.quit();
    app.run_until_parked();
}
