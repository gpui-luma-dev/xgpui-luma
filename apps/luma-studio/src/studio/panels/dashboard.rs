use xgpui_luma::controls::table::TableEventExt;
use std::sync::Arc;
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::motion::VisualTransition;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonContentContext, ControlIcon};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::table::{TableSelectionMode, TableEvent, PagingTable};
use xgpui_luma::controls::pager::PagerStyle;
use xgpui_luma::infra::presenter::ControlPresenter;
use xgpui_luma::controls::sidebar::{SidebarPresentation, SidebarControl};
use xgpui_luma::{column, column_emphasis, paging_table};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize, with_look};
use lucide_svg_static::Icon as LucideIcon;

use super::sidebar::{INITIAL_PROPERTY_SELECTION_ID, property_sidebar};
use super::task_list::{Task, build_task_rows, email_column, status_cell, tag_pill};

const DEFAULT_PAGE_SIZE: usize = 25;
const LIST_HEADER_TITLE: &str = "Documents";
const TABLE_OUTER_PADDING_PX: f32 = 16.0;
const CONTENT_INSET_PX: f32 = 10.0;

pub struct DashboardPanel {
    look: Arc<ShadcnLook>,
    sidebar: Entity<SidebarControl>,
    sidebar_toggle: IconButton,
    table: PagingTable<Task>,
    sidebar_open: bool,
    sidebar_transition: VisualTransition,
    _subscriptions: Vec<Subscription>,
}

impl DashboardPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let sidebar = shadcn::Sidebar::new("studio-dashboard-nav")
            .look(look.as_ref())
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .drag_drop(xgpui_luma::controls::sidebar::SidebarDragDrop::new().allowed_groups(["properties"]))
            .sidebar(property_sidebar(&look, "studio-dashboard-nav-panel", "Properties", "Task workspace"))
            .spawn(cx);

        let sidebar_toggle = shadcn::Button::icon_button("studio-dashboard-sidebar-toggle", LucideIcon::PanelLeft)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);
        sidebar_toggle.update(cx, |button, cx| {
            button.set_presenter(
                sidebar_toggle_presenter(ControlIcon::Lucide(LucideIcon::PanelLeft), look.chrome().title_text),
                cx,
            );
        });

        let tasks = build_task_rows();
        let dashboard_muted_text = look.chrome().muted_text;
        let table = paging_table! {
            table_theme = look.table_theme();
            id = "studio-dashboard-tasks";
            items = tasks;
            page_size = DEFAULT_PAGE_SIZE;
            pager = shadcn::Pager::new("studio-dashboard-tasks-pager").look(look.as_ref())
                .style(PagerStyle::MinimalEdge)
                .page_size(DEFAULT_PAGE_SIZE)
                .into_sdk_builder(cx);
            selection = TableSelectionMode::Single;
            selected_index = 1;
            active_index = 1;
            row_label = |row| row.title.clone();
            row_enabled = |row| row.enabled;
            grid_view = {
                column_emphasis!("Task", width = 108 => |row: &Task| row.id.clone()),
                column!("Title" => |row: &Task| {
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tag_pill(row.tag))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .truncate()
                                .child(row.title.clone()),
                        )
                }),
                column!("Status", width = 132 => |row: &Task| status_cell(row.status)),
                email_column(),
                column!("", width = 44 => move |_row: &Task| {
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(xgpui_luma::infra::icon::lucide_icon(
                            LucideIcon::EllipsisVertical,
                            dashboard_muted_text,
                            16.0,
                        ))
                }).resizable(false),
            };
            row_template = |model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.look.min_height))
                    .py(px(model.look.padding_y))
                    .bg(model.look.background)
                    .text_color(model.look.label_color)
                    .typography_style(model.look.label_typography)
                    .child(cells)
            };
        }
        .fill_height()
        .column_resizing(true)
        .spawn(cx);

        let list = table.read(cx).list().clone();
        list.update(cx, |table, cx| {
            table.set_row_key(|row| row.id.clone(), cx).expect("unique task IDs");
            table.set_row_reordering(true, cx).expect("keyed table");
        });

        let subscriptions = vec![cx.observe(&sidebar, |_, _, cx| cx.notify())];
        sidebar_toggle.clone().on_click(cx, |panel, cx| {
            panel.sidebar_open = !panel.sidebar_open;
            let presentation = if panel.sidebar_open {
                SidebarPresentation::Expanded
            } else {
                SidebarPresentation::Icons
            };
            panel.sidebar.update(cx, |sidebar, cx| sidebar.set_presentation(presentation, cx));
            cx.notify();
        });
        table.clone().on_event(cx, |_, _: &TableEvent, cx| {
            cx.notify();
        });

        Self {
            look,
            sidebar,
            sidebar_toggle,
            table,
            sidebar_open: true,
            sidebar_transition: VisualTransition::default(),
            _subscriptions: subscriptions,
        }
    }

    fn sync_sidebar_toggle_icon(&self, cx: &mut Context<Self>) {
        let icon = if self.sidebar_open {
            ControlIcon::Lucide(LucideIcon::PanelLeft)
        } else {
            ControlIcon::Lucide(LucideIcon::PanelLeftOpen)
        };
        self.sidebar_toggle.update(cx, |button, cx| {
            button.set_presenter(sidebar_toggle_presenter(icon, self.look.chrome().title_text), cx);
        });
    }
}

impl Render for DashboardPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sidebar_transition.set_target(if self.sidebar_open { 1.0 } else { 0.0 });
        self.sidebar_transition.sync();
        self.sidebar_transition.schedule_frame(window, cx);
        self.sync_sidebar_toggle_icon(cx);

        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let metrics = look.sidebar_metric_scale();
            let shell_radius = look.radius(ShadcnRadius::Lg);
            let sidebar_width =
                self.sidebar_transition.interpolate_pixels(metrics.width_icon_rail, metrics.width_expanded);
            let sidebar_bg = look.token_color("sidebar").unwrap_or(chrome.panel_background);
            let title_style = look.typography_scale(ShadcnTextSize::Lg);

            div().size_full().min_h_0().p(px(24.0)).child(
                div()
                    .id("studio-dashboard-shell")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .relative()
                    .overflow_hidden()
                    .rounded(px(shell_radius))
                    .bg(sidebar_bg)
                    .child(
                        div()
                            .id("studio-dashboard-rail")
                            .flex_none()
                            .w(sidebar_width)
                            .h_full()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div().flex_1().min_h(px(0.0)).w_full().overflow_hidden().child(
                                    shadcn::Frame::sidebar("dashboard-nav-frame")
                                        .look(look)
                                        .child(self.sidebar.clone())
                                        .render(cx),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .id("studio-dashboard-content")
                            .flex_1()
                            .min_w(px(0.0))
                            .h_full()
                            .p(px(CONTENT_INSET_PX))
                            .child(
                                shadcn::Frame::new("studio-dashboard-content-card")
                                    .look(look)
                                    .bg(chrome.content_background)
                                    .border_1()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .render(cx)
                                    .child(
                                        div()
                                            .id("studio-dashboard-list-header")
                                            .w_full()
                                            .flex_shrink_0()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .h(px(44.0))
                                            .px(px(12.0))
                                            .border_b_1()
                                            .border_color(chrome.border)
                                            .child(self.sidebar_toggle.clone())
                                            .child(div().h(px(16.0)).w(px(1.0)).bg(chrome.border))
                                            .child(
                                                div()
                                                    .typography_style(title_style)
                                                    .text_color(chrome.title_text)
                                                    .child(LIST_HEADER_TITLE),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h(px(0.0))
                                            .w_full()
                                            .flex()
                                            .flex_col()
                                            .p(px(TABLE_OUTER_PADDING_PX))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w(px(0.0))
                                                    .min_h(px(0.0))
                                                    .w_full()
                                                    .child(self.table.clone()),
                                            ),
                                    ),
                            ),
                    ),
            )
        })
    }
}

fn sidebar_toggle_presenter(icon: ControlIcon, color: gpui::Hsla) -> ControlPresenter<ButtonContentContext<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(lucide) => {
            div().child(xgpui_luma::infra::icon::lucide_icon(*lucide, color, 16.0)).into_any_element()
        }
        ControlIcon::SvgPath(path) => {
            gpui::svg().size(px(16.0)).text_color(color).path(path.clone()).into_any_element()
        }
    })
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{MouseButton, TestAppContext, point, size};

    #[test]
    fn dashboard_properties_move_without_navigation_and_pinned_items_stay_fixed() {
        use xgpui_luma::controls::sidebar::SidebarEvent;
        let mut app = TestAppContext::single();
        let (panel, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            DashboardPanel::new(cx, Arc::new(ShadcnLook::built_in()))
        });
        cx.simulate_resize(size(px(1400.0), px(900.0)));
        cx.run_until_parked();
        let sidebar = cx.update(|_, cx| panel.read(cx).sidebar.clone());
        let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let log = events.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&sidebar, move |_, event: &SidebarEvent, _| log.borrow_mut().push(event.clone()))
        });
        let start = cx.debug_bounds("sidebar-drag-dimensions").unwrap().center();
        let target = cx.debug_bounds("sidebar-drag-fill").unwrap();
        let end = point(target.center().x, target.top() + px(2.0));
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(start + point(px(12.0), px(0.0)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| {
            let sidebar = sidebar.read(cx);
            let location = sidebar.item_location("dimensions").unwrap();
            assert_eq!(location.group_id.as_ref(), "properties");
            assert_eq!(location.parent_id.as_deref(), Some("look"));
            assert_eq!(sidebar.selected_id().map(|id| id.as_ref()), Some(INITIAL_PROPERTY_SELECTION_ID));
        });
        assert!(
            !events
                .borrow()
                .iter()
                .any(|event| matches!(event, SidebarEvent::Select { .. } | SidebarEvent::SubMenuToggle { .. }))
        );
        events.borrow_mut().clear();
        let start = cx.debug_bounds("sidebar-drag-summary").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(start + point(px(12.0), px(0.0)), MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        assert!(!events.borrow().iter().any(|event| matches!(event, SidebarEvent::DragDrop(_))));
    }

    #[test]
    fn dashboard_column_handles_resize_on_first_drag() {
        // Exercise the visible divider and both sides, not just the invisible
        // center of a handle. Start unfocused, as the real dashboard does.
        for selector in ["table-column-0", "table-column-1", "table-column-2"] {
            for hit_offset in [-3.0, 0.0, 3.0] {
                let mut app = TestAppContext::single();
                let (panel, cx) =
                    app.add_window_view(|_, cx| DashboardPanel::new(cx, Arc::new(ShadcnLook::built_in())));
                cx.update(|window, _| window.activate_window());
                cx.simulate_resize(size(px(1400.0), px(900.0)));
                cx.run_until_parked();
                let before = cx.debug_bounds(selector).unwrap();
                let start = point(before.right() + px(hit_offset), before.center().y);
                cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
                for offset in 1..=40 {
                    cx.simulate_mouse_move(
                        start + point(px(offset as f32), px(0.0)),
                        MouseButton::Left,
                        Default::default(),
                    );
                    cx.run_until_parked();
                }
                let after = cx.debug_bounds(selector).unwrap();
                assert!(
                    (after.size.width - before.size.width - px(40.0)).abs() < px(1.0),
                    "{selector}, hit offset {hit_offset}: before {before:?}, after {after:?}"
                );
                cx.simulate_mouse_up(start + point(px(40.0), px(0.0)), MouseButton::Left, Default::default());
                cx.update(|_, app| {
                    let table = panel.read(app).table.read(app).list().read(app);
                    assert_eq!(table.selected_keys(), [gpui::SharedString::from("T-0001")]);
                });
            }
        }
    }
    #[test]
    fn dashboard_resizes_when_sidebars_leave_title_collapsed() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| DashboardPanel::new(cx, Arc::new(ShadcnLook::built_in())));
        cx.update(|window, _| window.activate_window());
        // Dashboard width remaining in the default 1200px Studio window after
        // the 360px theme sidebar; Dashboard includes its own property sidebar.
        cx.simulate_resize(size(px(840.0), px(700.0)));
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("table-column-1").unwrap().size.width, px(0.0));
        for selector in ["table-column-2", "table-column-1"] {
            let before = cx.debug_bounds(selector).unwrap();
            let start = point(before.right(), before.center().y);
            cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
            for offset in 1..=40 {
                cx.simulate_mouse_move(
                    start + point(px(offset as f32), px(0.0)),
                    MouseButton::Left,
                    Default::default(),
                );
            }
            cx.simulate_mouse_up(start + point(px(40.0), px(0.0)), MouseButton::Left, Default::default());
            let after = cx.debug_bounds(selector).unwrap();
            assert!(after.size.width > before.size.width + px(30.0), "{selector}: before {before:?}, after {after:?}");
        }
    }
}
