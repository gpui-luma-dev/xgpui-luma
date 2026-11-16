//! Basic SDK navigation controls with their native templates and themes.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{sidebar, tabs};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum NavigationKind {
    Sidebar,
    Tabs,
}
impl NavigationKind {
    fn title(self) -> &'static str {
        match self {
            Self::Sidebar => "Sidebar",
            Self::Tabs => "Tabs Navigation",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Sidebar => {
                "SDK Sidebar. Select a destination or expand Projects. Arrow keys move focus; Enter activates the focused item."
            }
            Self::Tabs => "SDK Tabs. Select a tab or use arrow keys to navigate. Disabled tabs are skipped.",
        }
    }
}
enum Control {
    Sidebar(Entity<sidebar::SidebarControl>),
    Tabs(Entity<tabs::Tabs>),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Sidebar(v) => v.clone().into(),
            Self::Tabs(v) => v.clone().into(),
        }
    }
}
pub(super) struct NavigationSample {
    kind: NavigationKind,
    look: Arc<Look>,
    control: Control,
    selected: String,
    events: Entity<ControlEventStream>,
}
impl NavigationSample {
    pub(super) fn new(kind: NavigationKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            NavigationKind::Sidebar => Control::Sidebar(
                sidebar::SidebarControlBuilder::new("workshop-sdk-sidebar")
                    .selected_id("overview")
                    .sidebar(
                        sidebar::sidebar("workshop-sidebar-panel").content(
                            sidebar::sidebar_content().group(
                                sidebar::sidebar_group().menu(
                                    sidebar::sidebar_menu("destinations")
                                        .item(sidebar::sidebar_menu_item("overview", "Overview"))
                                        .item(
                                            sidebar::sidebar_menu_item("projects", "Projects").sub(
                                                sidebar::sidebar_menu_sub()
                                                    .item(sidebar::sidebar_menu_item("drafts", "Drafts"))
                                                    .item(sidebar::sidebar_menu_item("archive", "Archive")),
                                            ),
                                        )
                                        .item(sidebar::sidebar_menu_item("unavailable", "Unavailable").disabled(true))
                                        .item(sidebar::sidebar_menu_item("settings", "Settings")),
                                ),
                            ),
                        ),
                    )
                    .spawn(cx),
            ),
            NavigationKind::Tabs => Control::Tabs(
                tabs::TabsBuilder::new("workshop-sdk-tabs")
                    .items([
                        tabs::TabsItem::new("overview").label("Overview"),
                        tabs::TabsItem::new("unavailable").label("Unavailable").enabled(false),
                        tabs::TabsItem::new("activity").label("Activity"),
                        tabs::TabsItem::new("settings").label("Settings"),
                    ])
                    .active("overview")
                    .spawn(cx),
            ),
        };
        match &control {
            Control::Sidebar(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &sidebar::SidebarEvent, cx| {
                    match event {
                        sidebar::SidebarEvent::Select { id } => this.selected = id.to_string(),
                        sidebar::SidebarEvent::HoverChanged { .. } | sidebar::SidebarEvent::ItemFocused { .. } => {
                            return;
                        }
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::Tabs(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &tabs::TabsEvent, cx| {
                    match event {
                        tabs::TabsEvent::Change { tab_id, .. } => this.selected = tab_id.to_string(),
                        tabs::TabsEvent::ItemBoundsChanged { .. }
                        | tabs::TabsEvent::ItemFocused { .. }
                        | tabs::TabsEvent::FocusChanged { .. } => return,
                        _ => {}
                    }
                    this.record(event, cx);
                })
                .detach();
            }
        }
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-navigation-events",
                "Select a native SDK navigation item to see its events.",
            )
        });
        Self { kind, look, control, selected: "overview".into(), events }
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(&format!("{event:?}"), cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for NavigationSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let preview = match self.control {
            Control::Sidebar(_) => div()
                .debug_selector(|| "workshop-sidebar-viewport".into())
                .w(px(260.))
                .h(px(280.))
                .child(self.control.view())
                .into_any_element(),
            Control::Tabs(_) => self.control.view().into_any_element(),
        };
        page(
            "workshop-navigation-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-navigation-example",
                    self.kind.title(),
                    self.kind.description(),
                    700.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(preview).child(
                        div()
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(format!("Selected: {}", self.selected)),
                    ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext, VisualTestContext, point};
    use crate::workshop::{Sample, Workshop};
    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<NavigationSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let Sample::Navigation(v) = &workshop.read(cx).samples[id] else {
                panic!("missing SDK navigation")
            };
            v.clone()
        })
    }
    fn assert_retained(
        workshop: &Entity<Workshop>,
        sample: &Entity<NavigationSample>,
        id: &'static str,
        expected: &str,
        cx: &mut VisualTestContext,
    ) {
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
        });
        assert_eq!(select(workshop, id, cx), *sample);
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected, expected));
    }
    #[test]
    fn native_tabs_keyboard_skips_disabled_items_wraps_and_retains_selection() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "tabs-navigation", cx);
        let tabs = cx.update(|window, cx| {
            let Control::Tabs(v) = &sample.read(cx).control else {
                unreachable!()
            };
            let v = v.clone();
            v.read(cx).focus_handle(cx).focus(window, cx);
            v
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(tabs.read(cx).active_id().map(|id| id.as_ref()), Some("activity"));
            assert_eq!(sample.read(cx).selected, "activity");
        });
        cx.simulate_keystrokes("right right");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected, "overview"));
        cx.simulate_keystrokes("left");
        cx.run_until_parked();
        assert_retained(&workshop, &sample, "tabs-navigation", "settings", cx);
        cx.update(|_, cx| assert_eq!(tabs.read(cx).active_id().map(|id| id.as_ref()), Some("settings")));
    }
    #[test]
    fn native_sidebar_expands_submenu_skips_disabled_destination_and_retains_state() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "sidebar", cx);
        let viewport = cx.debug_bounds("workshop-sidebar-viewport").unwrap();
        cx.simulate_click(point(viewport.left() + px(40.), viewport.top() + px(15.)), Default::default());
        cx.simulate_keystrokes("home down right down enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected, "drafts"));
        assert_retained(&workshop, &sample, "sidebar", "drafts", cx);
        let viewport = cx.debug_bounds("workshop-sidebar-viewport").unwrap();
        cx.simulate_click(point(viewport.left() + px(40.), viewport.top() + px(15.)), Default::default());
        // The Projects branch remains expanded, so Archive is still reachable.
        cx.simulate_keystrokes("home down down down enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected, "archive"));
        cx.simulate_keystrokes("down enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sample.read(cx).selected, "settings"));
    }
}
