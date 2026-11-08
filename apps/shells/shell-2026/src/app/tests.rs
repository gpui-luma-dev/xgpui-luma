use super::*;
use super::titlebar::{CLOSED_TITLEBAR_WIDTH, titlebar_compartment_width};
use xgpui_luma::shell::TITLE_BAR_LEFT_PADDING;
use gpui::{MouseButton, MouseDownEvent, TestAppContext, VisualTestContext, point};

fn assert_aligned(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    let separator = cx.debug_bounds("shell-2026-separator").unwrap();
    let workspace = cx.debug_bounds("shell-2026-workspace").unwrap();
    assert!(
        (separator.origin.x - workspace.origin.x).abs() <= px(1.0),
        "separator {separator:?}, workspace {workspace:?}"
    );
    assert_eq!(cx.debug_bounds("shell-2026-sidebar-fill").unwrap().right(), workspace.left());
    assert!(separator.bottom() < workspace.top(), "the two separators must remain disconnected");
    assert_eq!(cx.debug_bounds("shell-2026-rail-bounds").unwrap().size.width, RAIL_WIDTH);
    assert_eq!(cx.debug_bounds("shell-2026-rail-bounds").unwrap().top(), SHELL_TITLEBAR_HEIGHT);
}

#[test]
fn rendered_separator_tracks_drag_and_restores_from_both_collapse_paths() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|window, cx| {
        xgpui_luma::init(cx).unwrap();
        Shell2026App::new(window, cx, ShellThemeChoice::Default)
    });
    assert_aligned(cx);
    let workspace = cx.debug_bounds("shell-2026-workspace").unwrap();
    let start = point(workspace.left(), workspace.top() + px(100.0));
    let end = start + point(px(60.0), px(0.0));
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
    assert_aligned(cx);
    let resized_width = cx.update(|_, cx| view.read(cx).sidebar_width);
    assert!(resized_width > INITIAL_SIDEBAR_WIDTH, "drag must resize the sidebar");
    let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let _subscription = cx.update(|_, cx| {
        let panels = view.read(cx).panels.clone();
        let samples = samples.clone();
        cx.subscribe(&panels, move |_, event: &ResizablePanelsEvent, _| {
            if let ResizablePanelsEvent::SizesChanged { sizes_px } = event {
                samples.borrow_mut().push(sizes_px[0]);
            }
        })
    });
    let toggle = cx.debug_bounds("shell-2026-toggle-sidebar").unwrap().center();
    cx.simulate_click(toggle, Default::default());
    cx.run_until_parked();
    assert!(
        samples.borrow().iter().any(|&width| width > 0.0 && width < resized_width.as_f32()),
        "collapse must publish intermediate animated widths"
    );
    let closed_separator = cx.debug_bounds("shell-2026-separator").unwrap();
    assert_eq!(closed_separator.left(), TITLE_BAR_LEFT_PADDING + CLOSED_TITLEBAR_WIDTH - px(1.0));
    assert_eq!(cx.debug_bounds("shell-2026-workspace").unwrap().left(), RAIL_WIDTH + CANVAS_BORDER);
    assert_eq!(cx.debug_bounds("shell-2026-rail-bounds").unwrap().size.width, RAIL_WIDTH);
    cx.simulate_click(toggle, Default::default());
    cx.run_until_parked();
    assert_aligned(cx);
    assert_eq!(cx.update(|_, cx| view.read(cx).sidebar_width), resized_width);

    let workspace = cx.debug_bounds("shell-2026-workspace").unwrap();
    let seam = point(workspace.left(), workspace.top() + px(100.0));
    samples.borrow_mut().clear();
    cx.simulate_event(MouseDownEvent {
        position: seam,
        button: MouseButton::Left,
        click_count: 2,
        ..Default::default()
    });
    cx.simulate_mouse_up(seam, MouseButton::Left, Default::default());
    assert!(
        samples.borrow().iter().any(|&width| width > 0.0 && width < resized_width.as_f32()),
        "double-click collapse must also animate"
    );
    cx.run_until_parked();
    assert_eq!(cx.debug_bounds("shell-2026-separator").unwrap(), closed_separator);
    cx.simulate_click(toggle, Default::default());
    cx.run_until_parked();
    assert_aligned(cx);
    assert_eq!(cx.update(|_, cx| view.read(cx).sidebar_width), resized_width);
}

#[test]
fn separator_tracks_the_body_seam_including_native_and_fullscreen_insets() {
    for origin in [px(12.0), px(80.0), px(92.0)] {
        for width in [px(200.0), px(260.0), px(379.5), px(480.0)] {
            let separator_x = origin + titlebar_compartment_width(width, origin) - px(1.0);
            assert_eq!(separator_x, RAIL_WIDTH + CANVAS_BORDER + width);
        }
    }
}

#[test]
fn closed_separator_has_a_fixed_position_after_the_navigation_controls() {
    assert_eq!(titlebar_compartment_width(px(0.0), px(80.0)), CLOSED_TITLEBAR_WIDTH);
    assert_eq!(titlebar_compartment_width(px(0.0), px(92.0)), CLOSED_TITLEBAR_WIDTH);
}
