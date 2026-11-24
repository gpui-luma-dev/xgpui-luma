use gpui::{
    AnyElement, App, Anchor, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Size, Window, anchored, div, point,
    prelude::*, px, transparent_black,
};

use crate::controls::resizable_panels::ResizablePanelsLook;

mod handle;
mod resize;

pub use handle::{
    SlidePanelResizeHandlers, SlidePanelResizeHitHandlers, render_slide_panel_inner_resize_handle,
    render_slide_panel_resize_handle,
};
pub use resize::SlidePanelResizeDrag;

use super::model::{SUPPORTED_TOP_ANCHORS, SlidePanelEdge, SlidePanelState};

pub type SlidePanelKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type SlidePanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

pub struct SlidePanelOverlayHandlers {
    pub key_down: SlidePanelKeyDownHandler,
    pub backdrop_mouse_down: SlidePanelMouseDownHandler,
}

pub fn render_slide_panel_overlay(
    panel_background: Hsla,
    panels_look: &ResizablePanelsLook,
    state: &SlidePanelState,
    viewport: Size<gpui::Pixels>,
    panel_content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize_handlers: SlidePanelResizeHandlers,
) -> AnyElement {
    if state.active_edge().is_none() {
        return div().into_any_element();
    }
    anchored()
        .snap_to_window_with_margin(px(0.0))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), state.top_anchor().inset()))
        .child(render_slide_panel_surface(
            panel_background,
            panels_look,
            state,
            viewport,
            panel_content,
            handlers,
            resize_handlers,
        ))
        .into_any_element()
}

/// Render the SDK overlay inside a bounded local viewport, without window anchoring.
/// Uses the same drawer/resize chrome as the window overlay; handlers remain owner-supplied.
pub fn render_slide_panel_surface(
    panel_background: Hsla,
    panels_look: &ResizablePanelsLook,
    state: &SlidePanelState,
    viewport: Size<gpui::Pixels>,
    panel_content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize_handlers: SlidePanelResizeHandlers,
) -> AnyElement {
    debug_assert_eq!(SUPPORTED_TOP_ANCHORS.len(), 2);

    let Some(edge) = state.active_edge() else {
        return div().into_any_element();
    };

    let overlay_top_inset = state.top_anchor().inset();
    let available_height = (viewport.height - overlay_top_inset).max(px(1.0));
    let viewport_width = viewport.width.as_f32();
    let viewport_height = available_height.as_f32();

    let geometry = state.geometry();
    let capped_edge_panel_height = geometry.edge_height.min(viewport_height * geometry.edge_max_viewport_ratio);
    let side_panel_width = state.main_axis_size(geometry.side_width);
    let edge_panel_height = state.main_axis_size(capped_edge_panel_height);
    let (panel_width, panel_height) = match edge {
        SlidePanelEdge::Left | SlidePanelEdge::Right => (px(side_panel_width), available_height),
        SlidePanelEdge::Top | SlidePanelEdge::Bottom => (viewport.width, px(edge_panel_height)),
    };

    let horizontal_offset = px(-((1.0 - state.open_progress()) * side_panel_width));
    let vertical_offset = px(-((1.0 - state.open_progress()) * edge_panel_height));
    let split_px = resize_split_px(
        edge,
        viewport_width,
        viewport_height,
        side_panel_width,
        edge_panel_height,
        horizontal_offset,
        vertical_offset,
    );

    let panel_shell = div()
        .id("slide-panel-shell")
        .absolute()
        .occlude()
        .when(matches!(edge, SlidePanelEdge::Left), |panel| {
            panel.left(horizontal_offset).top_0().bottom_0().w(panel_width)
        })
        .when(matches!(edge, SlidePanelEdge::Right), |panel| {
            panel.right(horizontal_offset).top_0().bottom_0().w(panel_width)
        })
        .when(matches!(edge, SlidePanelEdge::Top), |panel| {
            panel.top(vertical_offset).left_0().right_0().h(panel_height)
        })
        .when(matches!(edge, SlidePanelEdge::Bottom), |panel| {
            panel.bottom(vertical_offset).left_0().right_0().h(panel_height)
        })
        .bg(panel_background)
        .on_key_down(handlers.key_down)
        .child(panel_content);

    let handle_metrics = state.resize_handle_metrics();
    let handle_active = state.resize_handle_hovered() || state.is_resizing();
    let SlidePanelResizeHandlers { mouse_down, drag_move, mouse_up, mouse_up_out, hover } = resize_handlers;
    let resize_handle = render_slide_panel_resize_handle(
        edge,
        panels_look,
        &handle_metrics,
        split_px,
        handle_active,
        SlidePanelResizeHitHandlers { mouse_down, hover },
    );

    let mut overlay_root = div()
        .id("slide-panel-window-overlay")
        .relative()
        .w(viewport.width)
        .h(available_height)
        .overflow_hidden()
        .on_drag_move(drag_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out);

    if state.backdrop_click_closes() {
        overlay_root = overlay_root.child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .bg(transparent_black())
                .on_mouse_down(MouseButton::Left, handlers.backdrop_mouse_down),
        );
    }

    overlay_root.child(panel_shell).child(resize_handle).into_any_element()
}

pub fn render_slide_panel_inset(
    panels_look: &ResizablePanelsLook,
    state: &SlidePanelState,
    panel_content: AnyElement,
    panel_background: Hsla,
    handlers: SlidePanelOverlayHandlers,
    resize_handlers: SlidePanelResizeHandlers,
) -> AnyElement {
    let Some(edge) = state.active_edge() else {
        return div().into_any_element();
    };

    debug_assert_eq!(edge, SlidePanelEdge::Right, "inset panel supports right edge only");

    let side_panel_width = state.main_axis_size(state.geometry().side_width);
    let panel_width = px(side_panel_width);
    let horizontal_offset = px(-((1.0 - state.open_progress()) * side_panel_width));

    let handle_metrics = state.resize_handle_metrics();
    let handle_active = state.resize_handle_hovered() || state.is_resizing();
    let SlidePanelResizeHandlers { mouse_down, drag_move, mouse_up, mouse_up_out, hover } = resize_handlers;
    let resize_handle = render_slide_panel_inner_resize_handle(
        edge,
        panels_look,
        &handle_metrics,
        handle_active,
        SlidePanelResizeHitHandlers { mouse_down, hover },
    );

    let panel_shell = div()
        .id("slide-panel-shell")
        .absolute()
        .occlude()
        .right(horizontal_offset)
        .top_0()
        .bottom_0()
        .w(panel_width)
        .border_l(px(panels_look.divider_width))
        .border_color(panels_look.divider)
        .bg(panel_background)
        .on_key_down(handlers.key_down)
        .child(panel_content)
        .child(resize_handle);

    let mut overlay_root = div()
        .id("slide-panel-inset-overlay")
        .absolute()
        .inset_0()
        .overflow_hidden()
        .on_drag_move(drag_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out);

    if state.backdrop_click_closes() {
        overlay_root = overlay_root.child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .bg(transparent_black())
                .on_mouse_down(MouseButton::Left, handlers.backdrop_mouse_down),
        );
    }

    overlay_root.child(panel_shell).into_any_element()
}

fn resize_split_px(
    edge: SlidePanelEdge,
    viewport_width: f32,
    viewport_height: f32,
    side_panel_width: f32,
    edge_panel_height: f32,
    horizontal_offset: gpui::Pixels,
    vertical_offset: gpui::Pixels,
) -> f32 {
    match edge {
        SlidePanelEdge::Right => viewport_width - side_panel_width - horizontal_offset.as_f32(),
        SlidePanelEdge::Left => side_panel_width + horizontal_offset.as_f32(),
        SlidePanelEdge::Bottom => viewport_height - edge_panel_height - vertical_offset.as_f32(),
        SlidePanelEdge::Top => edge_panel_height + vertical_offset.as_f32(),
    }
}
