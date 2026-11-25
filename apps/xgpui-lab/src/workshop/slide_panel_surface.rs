//! The SDK slide panel on the design surface page (issue #393). `slide_panel.lss` holds the drawer's size (a
//! side drawer's width, a top or bottom drawer's height and its cap); its resize edge is drawn with the
//! resizable panels look. The page draws the drawer open inside a frame standing for the window, with the
//! SDK's own `render_slide_panel_surface`, and offers Right · Bottom to show both sizes and Replay to slide it
//! closed and open with the SDK's motion. Resizing is off: the drawer's handles are inert. The `.lss` has no states: the matrix is one cell.

use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, Context, Entity, FocusHandle, SharedString, Subscription, Window, div, prelude::*, px,
    size,
};
use xgpui_luma::controls::button::HasPresenter;
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::resizable_panels::{ResizablePanelsTheme, StyledResizablePanelsTheme};
use xgpui_luma::controls::slide_panel::{
    SlidePanelEdge, SlidePanelGeometry, SlidePanelOverlayHandlers, SlidePanelResizeHandlers, SlidePanelSizeConfig,
    SlidePanelState, SlidePanelTopAnchor, render_slide_panel_surface,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, ThemeTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The frame standing for the window the drawer slides over.
const FRAME: (f32, f32) = (560., 320.);
const CELL_FRAME: (f32, f32) = (280., 160.);

/// The edge option's items.
const RIGHT: &str = "right";
const BOTTOM: &str = "bottom";

fn binding(control: &'static str, theme: &Theme) -> StyleBinding {
    StyleBinding::new(control, ThemeSource::Fixed(theme.clone()))
}

/// A drawer state open at `edge`, sized from `theme`'s `slide_panel.lss`. It opens at once; `motion` then
/// decides whether later opening and closing slide.
fn open_state(theme: &Theme, edge: SlidePanelEdge, focus: FocusHandle, motion: bool) -> SlidePanelState {
    let mut state = SlidePanelState::new(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0., 0., 10_000.))
        .motion_enabled(false)
        .with_geometry(SlidePanelGeometry::from_binding(&binding("slide_panel", theme)));
    state.open(edge, focus);
    state.set_motion_enabled(motion);
    state
}

/// The drawer in `state` inside a `frame`, drawn from `theme`, with inert handlers.
fn drawer(theme: &Theme, state: &SlidePanelState, frame: (f32, f32)) -> AnyElement {
    let panels = StyledResizablePanelsTheme::new(binding("resizable_panels", theme));
    let look = panels.resolve(InteractionState::default());
    let palette = ThemeTokens::default().palette.surface;
    let content = div()
        .size_full()
        .p(px(16.))
        .text_color(palette.panel.foreground)
        .child("Filters and display options")
        .into_any_element();
    let handlers =
        SlidePanelOverlayHandlers { key_down: Box::new(|_, _, _| {}), backdrop_mouse_down: Box::new(|_, _, _| {}) };
    let resize = SlidePanelResizeHandlers {
        mouse_down: Box::new(|_, _, _| {}),
        drag_move: Box::new(|_, _, _| {}),
        mouse_up: Box::new(|_, _, _| {}),
        mouse_up_out: Box::new(|_, _, _| {}),
        hover: Box::new(|_, _, _| {}),
    };
    div()
        .w(px(frame.0))
        .h(px(frame.1))
        .bg(palette.subtle.background)
        .child(render_slide_panel_surface(
            palette.panel.background,
            &look,
            state,
            size(px(frame.0), px(frame.1)),
            content,
            handlers,
            resize,
        ))
        .into_any_element()
}

/// The live drawer, in the theme last bound and the picked edge. It keeps its state with the SDK's motion on,
/// so Replay slides it closed and open again.
struct SlidePanelHost {
    theme: Option<Theme>,
    edge: SlidePanelEdge,
    focus: FocusHandle,
    state: Option<SlidePanelState>,
    /// Reopen once the replayed close has finished.
    reopen: bool,
}

impl SlidePanelHost {
    /// Open afresh at the picked edge in the theme last bound.
    fn reset(&mut self) {
        self.state = self.theme.as_ref().map(|theme| open_state(theme, self.edge, self.focus.clone(), true));
        self.reopen = false;
    }

    /// Slide closed, then open again.
    fn replay(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = self.state.as_mut()
            && state.active_edge().is_some()
        {
            state.request_close();
            self.reopen = true;
            cx.notify();
        }
    }
}

impl Render for SlidePanelHost {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut animate = false;
        if let Some(state) = self.state.as_mut() {
            animate = state.sync_animation();
            if self.reopen && state.active_edge().is_none() {
                self.reopen = false;
                state.open(self.edge, self.focus.clone());
                animate = true;
            }
        }
        if animate || self.reopen {
            window.request_animation_frame();
        }
        let drawn = self.theme.as_ref().zip(self.state.as_ref()).map(|(theme, state)| drawer(theme, state, FRAME));
        div()
            .debug_selector(|| "workshop-sdk-slide-panel".into())
            .w(px(FRAME.0))
            .h(px(FRAME.1))
            .children(drawn)
    }
}

pub(super) struct SlidePanelSurface {
    host: Entity<SlidePanelHost>,
    /// Radix toggle group: Right · Bottom.
    edge_picker: ControlGroup<ControlGroupItem>,
    /// Slides the drawer closed and open again, with the SDK's motion.
    replay: Entity<xgpui_luma::controls::button::Button>,
}

impl SlidePanelSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let host = cx.new(|cx| SlidePanelHost {
            theme: None,
            edge: SlidePanelEdge::Right,
            focus: cx.focus_handle(),
            state: None,
            reopen: false,
        });
        let edge_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-surface-slide-panel-edge",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items([ControlGroupItem::new(RIGHT).label("Right"), ControlGroupItem::new(BOTTOM).label("Bottom")])
        .selected_ids([RIGHT])
        .spawn(cx);
        let target = host.clone();
        cx.subscribe(&edge_picker, move |_: &mut ControlSide, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some(id) = selected_ids.first()
            {
                let edge = if id.as_ref() == BOTTOM {
                    SlidePanelEdge::Bottom
                } else {
                    SlidePanelEdge::Right
                };
                target.update(cx, |host, cx| {
                    host.edge = edge;
                    host.reset();
                    cx.notify();
                });
                cx.notify();
            }
        })
        .detach();
        let target = host.clone();
        let replay = xgpui_luma_look_radix::Button::new("workshop-surface-slide-panel-replay")
            .look(look.as_ref())
            .soft()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .content(|_, _| div().child("Replay"))
            .on_click(cx, move |_: &mut ControlSide, cx| target.update(cx, |host, cx| host.replay(cx)))
            .spawn(cx);
        Box::new(Self { host, edge_picker, replay })
    }
}

impl SurfaceControl for SlidePanelSurface {
    fn title(&self) -> &'static str {
        "Slide Panel"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.theme = Some(theme);
            host.reset();
            cx.notify();
        });
    }

    /// The drawn drawer is inert.
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A drawn drawer emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.host, |_, _, cx| cx.notify())
    }

    fn cell(&self, theme: &Theme, _id: SharedString, _states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let host = self.host.read(cx);
        drawer(theme, &open_state(theme, host.edge, host.focus.clone(), false), CELL_FRAME)
    }

    fn options(&self) -> Option<AnyElement> {
        Some(
            div()
                .flex()
                .items_center()
                .gap(px(12.))
                .child(self.edge_picker.clone())
                .child(self.replay.clone())
                .into_any_element(),
        )
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_slide_panel_page_draws_the_open_drawer() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "slide-panel");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<SlidePanelSurface>().is_some()));
        let drawn = cx.debug_bounds("workshop-sdk-slide-panel").expect("the drawer is drawn");
        assert_eq!(drawn.size.width, px(FRAME.0), "in its frame");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws the drawer");
    }
}
