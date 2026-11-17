//! SDK layout primitives and controls, without app-authored control templates.
use std::sync::Arc;
use gpui::{AnyElement, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::{
    DockPanel,
    controls::{
        resizable_panels::{self, ResizablePanelSpec},
        split_view,
    },
    theme::ThemeTokens,
};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum LayoutKind {
    Dock,
    Resizable,
    Split,
}
impl LayoutKind {
    fn title(self) -> &'static str {
        match self {
            Self::Dock => "DockPanel",
            Self::Resizable => "Resizable Panels",
            Self::Split => "Split View",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Dock => {
                "SDK DockPanel docks children in declaration order; the last child fills the remaining space."
            }
            Self::Resizable => {
                "SDK ResizablePanels. Drag the divider, or click it and use arrow keys to resize the panes."
            }
            Self::Split => {
                "SDK SplitView with its default template. Drag the separator to resize; click it to collapse or expand the sidebar. The SDK has no separate Unified style."
            }
        }
    }
}
// These are passive layout contents, using SDK default colors, not interactive chrome.
fn pane(label: &'static str) -> gpui::Div {
    let palette = ThemeTokens::default().palette.surface.panel;
    div()
        .size_full()
        .min_w_0()
        .min_h_0()
        .p(px(12.))
        .bg(palette.background)
        .text_color(palette.foreground)
        .child(crate::l10n::ui(label))
}
fn dock() -> DockPanel {
    DockPanel::new()
        .top(div().debug_selector(|| "workshop-dock-top".into()).h(px(40.)).flex_none().child(pane("Top")))
        .bottom(div().debug_selector(|| "workshop-dock-bottom".into()).h(px(40.)).flex_none().child(pane("Bottom")))
        .left(div().debug_selector(|| "workshop-dock-left".into()).w(px(110.)).flex_none().child(pane("Left")))
        .right(div().debug_selector(|| "workshop-dock-right".into()).w(px(110.)).flex_none().child(pane("Right")))
        .fill(div().debug_selector(|| "workshop-dock-fill".into()).size_full().child(pane("Fill")))
}
enum Control {
    Dock,
    Resizable(Entity<resizable_panels::ResizablePanels>),
    Split(Entity<split_view::SplitView>),
}
impl Control {
    fn view(&self) -> AnyElement {
        match self {
            Self::Dock => dock().into_any_element(),
            Self::Resizable(v) => v.clone().into_any_element(),
            Self::Split(v) => v.clone().into_any_element(),
        }
    }
    fn state(&self, cx: &gpui::App) -> String {
        match self {
            Self::Dock => "Top, Bottom, Left, Right, then Fill".into(),
            Self::Resizable(v) => format!(
                "Pane widths: {}",
                v.read(cx).panel_sizes_px().iter().map(|v| format!("{v:.0} px")).collect::<Vec<_>>().join(", ")
            ),
            Self::Split(v) => format!(
                "Sidebar: {:.0} px · Collapsed: {}",
                f32::from(v.read(cx).sidebar_width()),
                v.read(cx).collapsed()
            ),
        }
    }
}
pub(super) struct LayoutSample {
    kind: LayoutKind,
    look: Arc<Look>,
    control: Control,
    events: Entity<ControlEventStream>,
}
impl LayoutSample {
    pub(super) fn new(kind: LayoutKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            LayoutKind::Dock => Control::Dock,
            LayoutKind::Resizable => Control::Resizable(
                resizable_panels::ResizablePanelsBuilder::new("workshop-sdk-resizable-panels")
                    .panel(
                        ResizablePanelSpec::new_render(|| {
                            pane("First pane").debug_selector(|| "workshop-layout-first".into())
                        })
                        .min(px(80.)),
                    )
                    .panel(
                        ResizablePanelSpec::new_render(|| {
                            pane("Second pane").debug_selector(|| "workshop-layout-second".into())
                        })
                        .min(px(80.)),
                    )
                    .spawn(cx),
            ),
            LayoutKind::Split => Control::Split(
                split_view::SplitViewBuilder::new("workshop-sdk-split-view")
                    .sidebar(|| pane("Sidebar").debug_selector(|| "workshop-layout-sidebar".into()))
                    .content(|| pane("Content").debug_selector(|| "workshop-layout-content".into()))
                    .spawn(cx),
            ),
        };
        match &control {
            Control::Dock => {}
            Control::Resizable(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &resizable_panels::ResizablePanelsEvent, cx| {
                    if !matches!(
                        event,
                        resizable_panels::ResizablePanelsEvent::HandleHoverChanged { .. }
                            | resizable_panels::ResizablePanelsEvent::HandleFocusChanged { .. }
                    ) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
            Control::Split(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &split_view::SplitViewEvent, cx| {
                    if !matches!(event, split_view::SplitViewEvent::SeparatorHoverChanged { .. }) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
        }
        let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "workshop-layout-events", "Native resize and collapse events appear here. DockPanel is a passive layout primitive and emits no control events."));
        Self { kind, look, control, events }
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
impl Render for LayoutSample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        page(
            "workshop-layout-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-layout-example",
                    self.kind.title(),
                    self.kind.description(),
                    900.,
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(16.))
                        .child(
                            div()
                                .debug_selector(|| "workshop-layout-viewport".into())
                                .w_full()
                                .h(px(280.))
                                .child(self.control.view()),
                        )
                        .child(
                            div()
                                .typography_style(self.look.typography_scale(TextSize::Xs))
                                .text_color(self.look.chrome().muted_text)
                                .child(self.control.state(cx)),
                        ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{MouseButton, TestAppContext, VisualTestContext, point};
    use crate::workshop::{Sample, Workshop};
    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<LayoutSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let Sample::Layout(v) = &workshop.read(cx).samples[id] else {
                panic!("missing native layout")
            };
            v.clone()
        })
    }
    #[test]
    fn sdk_dock_panel_allocates_edges_and_fills_remainder() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        select(&workshop, "dock-panel", cx);
        let top = cx.debug_bounds("workshop-dock-top").unwrap();
        let bottom = cx.debug_bounds("workshop-dock-bottom").unwrap();
        let left = cx.debug_bounds("workshop-dock-left").unwrap();
        let right = cx.debug_bounds("workshop-dock-right").unwrap();
        let fill = cx.debug_bounds("workshop-dock-fill").unwrap();
        assert_eq!(top.size.height, px(40.));
        assert_eq!(bottom.size.height, px(40.));
        assert_eq!(left.size.width, px(110.));
        assert_eq!(right.size.width, px(110.));
        assert_eq!(fill.top(), top.bottom());
        assert_eq!(fill.bottom(), bottom.top());
        assert_eq!(fill.left(), left.right());
        assert_eq!(fill.right(), right.left());
    }
    #[test]
    fn sdk_layout_dividers_resize_and_retain_state_across_navigation_and_theme() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        for (id, first, second) in [
            ("resizable-panels", "workshop-layout-first", "workshop-layout-second"),
            ("split-view-unified", "workshop-layout-sidebar", "workshop-layout-content"),
        ] {
            let sample = select(&workshop, id, cx);
            let before = cx.debug_bounds(first).unwrap();
            let next = cx.debug_bounds(second).unwrap();
            let start = point((before.right() + next.left()) / 2., before.center().y);
            let end = start + point(px(32.), px(0.));
            cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
            cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
            cx.run_until_parked();
            let resized = cx.debug_bounds(first).unwrap().size.width;
            assert!(resized > before.size.width, "{id} divider must resize its native pane");
            let state = cx.update(|_, cx| sample.read(cx).control.state(cx));
            workshop.update(cx, |v, cx| {
                v.select("button", cx);
                v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            });
            let retained = select(&workshop, id, cx);
            assert_eq!(sample, retained);
            cx.update(|_, cx| assert_eq!(sample.read(cx).control.state(cx), state));
            assert_eq!(cx.debug_bounds(first).unwrap().size.width, resized);
            if id == "resizable-panels" {
                let first_bounds = cx.debug_bounds(first).unwrap();
                let next = cx.debug_bounds(second).unwrap();
                cx.simulate_click(
                    point((first_bounds.right() + next.left()) / 2., first_bounds.center().y),
                    Default::default(),
                );
                cx.simulate_keystrokes("right");
                cx.run_until_parked();
                assert!(cx.debug_bounds(first).unwrap().size.width > resized);
            } else {
                let first_bounds = cx.debug_bounds(first).unwrap();
                let next = cx.debug_bounds(second).unwrap();
                cx.simulate_click(
                    point((first_bounds.right() + next.left()) / 2., first_bounds.center().y),
                    Default::default(),
                );
                cx.run_until_parked();
                cx.update(|_, cx| {
                    let Control::Split(v) = &sample.read(cx).control else {
                        unreachable!()
                    };
                    assert!(v.read(cx).collapsed());
                });
            }
        }
    }
    #[test]
    fn unsupported_layout_entries_explain_sdk_limitations() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        for id in ["slide-panel", "split-view-inset"] {
            workshop.update(cx, |v, cx| v.select(id, cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-sdk-unavailable").is_some());
            cx.update(|_, cx| assert!(!workshop.read(cx).samples.contains_key(id)));
        }
    }
}
