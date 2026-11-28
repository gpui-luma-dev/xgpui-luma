use super::*;
use super::drag::{location, item, move_item};
use gpui::{
    Context, Entity, Render, Window, TestAppContext, VisualTestContext, IntoElement, MouseButton, div, point, px,
    prelude::*,
};
use crate::infra::drag_drop::DragEndReason;
fn fixture() -> SidebarBuilder {
    sidebar("panel")
        .content(
            sidebar_content()
                .group(
                    sidebar_group().id("one").label("One").menu(
                        sidebar_menu("menu-one")
                            .item(sidebar_menu_item("branch", "Branch").expanded(true).sub(
                                sidebar_menu_sub().item(sidebar_menu_item("a", "A")).item(sidebar_menu_item("b", "B")),
                            ))
                            .item(sidebar_menu_item("fixed", "Fixed").draggable(false)),
                    ),
                )
                .group(
                    sidebar_group().id("two").label("Two").menu(
                        sidebar_menu("menu-two")
                            .item(sidebar_menu_item("empty", "Empty branch").expanded(false).sub(sidebar_menu_sub()))
                            .item(sidebar_menu_item("c", "C"))
                            .item(sidebar_menu_item("disabled", "Disabled").disabled(true)),
                    ),
                ),
        )
        .footer(sidebar_footer().child(sidebar_menu_item("footer", "Footer")))
}
struct Page {
    sidebar: Entity<SidebarControl>,
    events: Vec<SidebarEvent>,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.)).h(px(500.)).child(self.sidebar.clone())
    }
}
fn setup(app: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
    setup_animated(app, false)
}
fn setup_animated(app: &mut TestAppContext, motion_enabled: bool) -> (Entity<Page>, &mut VisualTestContext) {
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let sidebar = SidebarControl::new("nav")
            .motion_enabled(motion_enabled)
            .motion(|m| {
                m.drop_layout = crate::motion::Transition::timed(
                    std::time::Duration::from_millis(1000),
                    crate::motion::Easing::OutCubic,
                )
            })
            .selected_id("a")
            .drag_drop(SidebarDragDrop::new())
            .sidebar(fixture())
            .on_event(cx, |owner: &mut Page, event, _| owner.events.push(event.clone()))
            .spawn(cx);
        Page { sidebar, events: vec![] }
    });
    cx.run_until_parked();
    (page, cx)
}
fn drag(cx: &mut VisualTestContext, from: &str, to: &str, fraction: f32) {
    let start = cx.debug_bounds(Box::leak(format!("sidebar-drag-{from}").into_boxed_str())).unwrap().center();
    let bounds = cx.debug_bounds(Box::leak(format!("sidebar-drag-{to}").into_boxed_str())).unwrap();
    let end = point(bounds.center().x, bounds.top() + bounds.size.height * fraction);
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(start.x + px(12.), start.y), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
}
#[test]
fn native_group_move_and_reparent_preserve_selection_and_branch() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app);
    drag(cx, "a", "c", 0.2);
    cx.update(|_,cx| { let page=page.read(cx);let sidebar=page.sidebar.read(cx);let panel=sidebar.model.sidebar.as_ref().unwrap();
        assert_eq!(location(panel,&"a".into()),Some(SidebarDropLocation{group_id:"two".into(),parent_id:None,before:Some("c".into())}));
        assert_eq!(sidebar.model.selected_id.as_deref(),Some("a")); assert!(item(panel,&"branch".into()).unwrap().expanded.unwrap());
        assert!(!page.events.iter().any(|e| matches!(e,SidebarEvent::Select{..}|SidebarEvent::SubMenuToggle{..})));
        let events=page.events.iter().filter_map(|e| if let SidebarEvent::DragDrop(e)=e{Some(e)}else{None}).collect::<Vec<_>>();
        assert_eq!(events.len(),4);assert!(matches!(events[1],SidebarDragEvent::ItemsReordered{location,..} if location.group_id.as_ref()=="two" && location.before.as_deref()==Some("c")));
        assert!(matches!(events[3],SidebarDragEvent::DragEnded{reason:DragEndReason::Reordered,..}));
    });
    drag(cx, "branch", "empty", 0.5);
    cx.update(|_, cx| {
        let page = page.read(cx);
        let panel = page.sidebar.read(cx).model.sidebar.as_ref().unwrap();
        assert_eq!(location(panel, &"branch".into()).unwrap().parent_id.as_deref(), Some("empty"));
        assert_eq!(item(panel, &"branch".into()).unwrap().children[0].id.as_ref(), "b");
    });
}
#[test]
fn transaction_rejects_cycles_collisions_fixed_missing_and_wrong_group_without_mutation() {
    let config = SidebarDragDrop::new();
    for (key, group, parent, before) in [
        ("branch", "one", Some("a"), None),
        ("fixed", "two", None, None),
        ("a", "missing", None, None),
        ("a", "two", None, Some("b")),
        ("a", "two", Some("disabled"), None),
        ("footer", "two", None, None),
    ] {
        let mut panel = fixture().build();
        let original = location(&panel, &"a".into());
        let target = SidebarDropLocation {
            group_id: group.into(),
            parent_id: parent.map(Into::into),
            before: before.map(Into::into),
        };
        assert!(move_item(&mut panel, &key.into(), &target, &config).is_err());
        assert_eq!(location(&panel, &"a".into()), original);
    }
    let mut panel = fixture().build();
    let dest =
        SidebarDropLocation { group_id: "one".into(), parent_id: Some("branch".into()), before: Some("a".into()) };
    assert!(!move_item(&mut panel, &"a".into(), &dest, &config).unwrap());
    assert!(move_item(&mut panel, &"a".into(), &dest, &SidebarDragDrop::new().allowed_groups(["two"])).is_err());
    panel.content.as_mut().unwrap().groups[1]
        .menu
        .as_mut()
        .unwrap()
        .items
        .push(SidebarMenuItemBuilder::new("a", "Duplicate").build());
    assert!(move_item(&mut panel, &"a".into(), &dest, &config).is_err());
}

#[test]
fn native_noop_reject_unanswered_and_cycle_are_atomic() {
    for mode in 0..4 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        let sidebar = cx.update(|_, cx| page.read(cx).sidebar.clone());
        if mode == 1 || mode == 2 {
            sidebar.update(cx, |s, _| {
                s.set_drop_handler(move |s, p, _, cx| {
                    if mode == 1 {
                        s.reject_drop(p, "policy", cx);
                    }
                })
            });
        }
        if mode == 3 {
            drag(cx, "branch", "b", 0.2);
        } else {
            drag(cx, "a", "a", 0.2);
        }
        cx.update(|_, cx| {
            let page = page.read(cx);
            let panel = page.sidebar.read(cx).model.sidebar.as_ref().unwrap();
            assert_eq!(location(panel, &"a".into()).unwrap().parent_id.as_deref(), Some("branch"));
            let events = page
                .events
                .iter()
                .filter_map(|e| {
                    if let SidebarEvent::DragDrop(e) = e {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 3, "mode {mode}: {events:?}");
            if mode == 0 {
                assert!(matches!(events[1], SidebarDragEvent::Dropped { changed: false, .. }));
            } else {
                assert!(matches!(events[1], SidebarDragEvent::DropRejected { .. }));
            }
        });
    }
}
#[test]
fn escape_and_presentation_change_cancel_and_fixed_rows_never_start() {
    for mode in 0..2 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        let start = cx.debug_bounds("sidebar-drag-a").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(point(start.x + px(12.), start.y), MouseButton::Left, Default::default());
        cx.run_until_parked();
        if mode == 0 {
            cx.simulate_keystrokes("escape");
        } else {
            let sidebar = cx.update(|_, cx| page.read(cx).sidebar.clone());
            sidebar.update(cx, |s, cx| s.set_presentation(SidebarPresentation::Icons, cx));
        }
        cx.simulate_mouse_up(start, MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(
                page.read(cx)
                    .events
                    .iter()
                    .filter(|e| matches!(
                        e,
                        SidebarEvent::DragDrop(SidebarDragEvent::DragEnded { reason: DragEndReason::Cancelled, .. })
                    ))
                    .count(),
                1
            )
        });
    }
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app);
    drag(cx, "fixed", "c", 0.8);
    drag(cx, "disabled", "c", 0.8);
    cx.update(|_, cx| assert!(!page.read(cx).events.iter().any(|e| matches!(e, SidebarEvent::DragDrop(_)))));
}

#[test]
fn reparenting_animates_row_positions_without_changing_selection() {
    use std::time::Duration;
    let mut app = TestAppContext::single();
    let (page, cx) = setup_animated(&mut app, true);
    let sidebar = cx.update(|_, cx| page.read(cx).sidebar.clone());
    let old = cx.debug_bounds("sidebar-drag-a").unwrap();
    // Freeze the first frame of the drop so machine speed cannot advance the animation before the assertion.
    sidebar.update(cx, |s, cx| s.drop_layout_motion(cx).elapse_for_test(Duration::ZERO));
    drag(cx, "a", "c", 0.2);
    assert!((cx.debug_bounds("sidebar-drag-a").unwrap().top() - old.top()).abs() < px(1.));
    cx.update(|_, cx| {
        let s = sidebar.read(cx);
        assert_eq!(s.item_location("a").unwrap().group_id.as_ref(), "two");
        assert_eq!(s.selected_id().map(|s| s.as_ref()), Some("a"));
        assert!(s.drop_layout_motion(cx).is_animating());
    });
    sidebar.update(cx, |s, cx| s.drop_layout_motion(cx).elapse_for_test(Duration::from_millis(500)));
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    let mid = cx.debug_bounds("sidebar-drag-a").unwrap().top();
    sidebar.update(cx, |s, cx| s.drop_layout_motion(cx).elapse_for_test(Duration::from_millis(1100)));
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    let end = cx.debug_bounds("sidebar-drag-a").unwrap().top();
    assert!(mid > old.top() && mid < end);
}
