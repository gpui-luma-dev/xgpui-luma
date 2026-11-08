//! Ensure benchmark gestures change real production controls and remain virtualized.
#[allow(dead_code)]
#[path = "../benches/support/collections.rs"]
mod collections;
use collections::*;
use gpui::{TestAppContext, SharedString};
use std::sync::atomic::Ordering;

#[test]
fn tree_fixture_has_exact_unique_node_counts_and_nested_paths() {
    fn visit(
        items: &[xgpui_luma::controls::tree_view::TreeNode<Row>],
        ids: &mut std::collections::HashSet<SharedString>,
        depth: usize,
        max: &mut usize,
    ) {
        for item in items {
            assert!(ids.insert(item.id.clone()));
            *max = (*max).max(depth);
            visit(&item.children, ids, depth + 1, max);
        }
    }
    for count in [100, 101, 102, 10_000] {
        let mut ids = std::collections::HashSet::new();
        let mut depth = 0;
        visit(&nodes(count, true), &mut ids, 0, &mut depth);
        assert_eq!(ids.len(), count);
        assert_eq!(depth, 2);
    }
}
#[test]
fn scrolling_keyboard_and_presenters_are_real_and_bounded() {
    for case in Case::ALL {
        let mut app = TestAppContext::single();
        let (host, cx, paints) = setup(&mut app, case, 10_000);
        let initial_calls = paints.calls();
        operate(&host, cx, "warm_redraw", 10_000, 1);
        let warm_calls = paints.calls() - initial_calls;
        assert!(
            warm_calls > 0 && warm_calls < 1000,
            "steady rendering should be viewport bounded: {case:?}, {warm_calls}"
        );
        let before = paints.last.load(Ordering::Relaxed);
        operate(&host, cx, "wheel", 10_000, 1);
        assert_ne!(paints.last.load(Ordering::Relaxed), before, "wheel must move viewport: {case:?}");
        operate(&host, cx, "keyboard", 10_000, 1);
        cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
            Control::Table(t) => assert_eq!(t.read(app).active_index(), Some(1)),
            Control::Tree(t) => assert_eq!(
                t.read(app).active_node_id().unwrap().as_ref(),
                if case == Case::TreeClosed { "row-101" } else { "row-1" }
            ),
        });
        assert!(released(&host, cx));
        app.quit();
    }
}
#[test]
fn data_projection_and_keyed_replacement_preserve_selected_identity() {
    for case in [Case::Grid, Case::TreeOpen, Case::TreeClosed] {
        let mut app = TestAppContext::single();
        let (host, cx, _) = setup(&mut app, case, 10_000);
        operate(&host, cx, "selection", 10_000, 1);
        operate(&host, cx, "sort", 10_000, 1);
        operate(&host, cx, "single_value_replace", 10_000, 1);
        cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
            Control::Table(t) => assert_eq!(t.read(app).selected_keys(), [SharedString::from("row-1")]),
            Control::Tree(t) => assert!(t.read(app).selected_ids().contains(&SharedString::from("row-101"))),
        });
        operate(&host, cx, "filter", 10_000, 1);
        cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
            Control::Table(t) => assert_eq!(t.read(app).items().len(), 5000),
            Control::Tree(t) => assert!(t.read(app).visible_ids().len() < 10_000),
        });
        assert!(released(&host, cx));
        app.quit();
    }
}
#[test]
fn expansion_and_column_drag_change_displayed_geometry() {
    let mut app = TestAppContext::single();
    let (host, cx, _) = setup(&mut app, Case::TreeOpen, 10_000);
    let visible = cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
        Control::Tree(t) => t.read(app).visible_ids().len(),
        _ => unreachable!(),
    });
    operate(&host, cx, "expand_collapse", 10_000, 1);
    cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
        Control::Tree(t) => assert_eq!(t.read(app).visible_ids().len(), visible - 100),
        _ => unreachable!(),
    });
    assert!(released(&host, cx));
    app.quit();
    let mut app = TestAppContext::single();
    let (host, cx, _) = setup(&mut app, Case::Grid, 100);
    let width = cx.debug_bounds("table-column-0").unwrap().size.width;
    for step in 1..=3 {
        operate(&host, cx, "column_resize", 100, step);
        let delta = if step % 2 == 0 { 0.0 } else { 24.0 };
        assert!(
            (cx.debug_bounds("table-column-0").unwrap().size.width - width - gpui::px(delta)).abs() < gpui::px(1.0)
        );
    }
    assert!(released(&host, cx));
    app.quit();
}

#[test]
fn explicit_shutdown_releases_the_window_host() {
    let mut app = TestAppContext::single();
    let (host, _, _) = setup(&mut app, Case::Grid, 100);
    let weak = host.downgrade();
    drop(host);
    app.quit();
    assert!(weak.upgrade().is_none());
}

#[test]
fn explicit_shutdown_releases_the_app_context() {
    let mut app = TestAppContext::single();
    let weak_app = std::rc::Rc::downgrade(&app.app);
    let (host, cx, _) = setup(&mut app, Case::Grid, 100);
    assert!(released(&host, cx));
    drop(host);
    app.quit();
    app.run_until_parked();
    drop(app);
    assert!(weak_app.upgrade().is_none());
}

#[test]
fn jump_reveals_the_last_loaded_row_inside_the_viewport() {
    for case in Case::ALL {
        let mut app = TestAppContext::single();
        let (host, cx, _) = setup(&mut app, case, 10_000);
        operate(&host, cx, "jump", 10_000, 1);
        let selector = if case.is_tree() {
            "collection-tree/node/row-9999"
        } else {
            "table-motion-row-9999"
        };
        let bounds = cx.debug_bounds(selector).unwrap_or_else(|| panic!("jump target must be prepainted: {case:?}"));
        assert!(
            bounds.top() >= gpui::px(0.0) && bounds.bottom() <= gpui::px(701.0),
            "jump target outside viewport: {case:?}: {bounds:?}"
        );
        assert!(released(&host, cx));
        app.quit();
        app.run_until_parked();
    }
}
