use crate::style::PartRead;
use gpui::{
    AnyElement, App, Div, DragMoveEvent, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Window, div, prelude::*, px,
};

use crate::controls::resizable_panels::{ResizablePanelsOrientation, ResizeHandleMetrics, handle_hit_target_main_axis_px};

use super::resize::SlidePanelResizeDrag;
use super::super::model::SlidePanelEdge;

pub type SlidePanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SlidePanelDragMoveHandler = Box<dyn Fn(&DragMoveEvent<SlidePanelResizeDrag>, &mut Window, &mut App) + 'static>;
pub type SlidePanelMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SlidePanelResizeHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub struct SlidePanelResizeHandlers {
    pub mouse_down: SlidePanelMouseDownHandler,
    pub drag_move: SlidePanelDragMoveHandler,
    pub mouse_up: SlidePanelMouseUpHandler,
    pub mouse_up_out: SlidePanelMouseUpHandler,
    pub hover: SlidePanelResizeHoverHandler,
}

pub struct SlidePanelResizeHitHandlers {
    pub mouse_down: SlidePanelMouseDownHandler,
    pub hover: SlidePanelResizeHoverHandler,
}

pub fn render_slide_panel_resize_handle(
    edge: SlidePanelEdge,
    look: &xgpui_luma_style::Resolved,
    handle_metrics: &ResizeHandleMetrics,
    split_px: f32,
    handle_active: bool,
    handlers: SlidePanelResizeHitHandlers,
) -> AnyElement {
    let orientation = if edge.is_horizontal_main_axis() {
        ResizablePanelsOrientation::Horizontal
    } else {
        ResizablePanelsOrientation::Vertical
    };

    let lane_px = if handle_active {
        handle_metrics.lane_px.max(1.0)
    } else {
        1.0
    };
    let (origin_px, lane_px, divider_local_px) = handle_overlay_geometry(split_px, lane_px);

    let mut handle = div()
        .id("slide-panel-resize-handle")
        .absolute()
        .when(orientation == ResizablePanelsOrientation::Horizontal, |this| this.cursor_col_resize())
        .when(orientation == ResizablePanelsOrientation::Vertical, |this| this.cursor_row_resize());

    handle = match orientation {
        ResizablePanelsOrientation::Horizontal => {
            handle.left(px(origin_px)).top(px(0.0)).bottom(px(0.0)).w(px(lane_px))
        }
        ResizablePanelsOrientation::Vertical => handle.top(px(origin_px)).left(px(0.0)).right(px(0.0)).h(px(lane_px)),
    };

    let divider = match orientation {
        ResizablePanelsOrientation::Horizontal => div()
            .absolute()
            .left(px(divider_local_px))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(px(look.root().number("divider_width")))
            .bg(look.root().color("divider")),
        ResizablePanelsOrientation::Vertical => div()
            .absolute()
            .top(px(divider_local_px))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(look.root().number("divider_width")))
            .bg(look.root().color("divider")),
    };

    if handle_active {
        handle = handle.child(render_handle_grip(orientation, look.root().color("grip_emphasis"), handle_metrics));
    }

    let handle_hit_px = px(handle_hit_target_main_axis_px(handle_metrics));
    let interaction_layer = render_handle_interaction_layer(orientation, handle_hit_px, split_px, origin_px, handlers);

    handle.child(divider).child(interaction_layer).into_any_element()
}

pub fn render_slide_panel_inner_resize_handle(
    edge: SlidePanelEdge,
    look: &xgpui_luma_style::Resolved,
    handle_metrics: &ResizeHandleMetrics,
    handle_active: bool,
    handlers: SlidePanelResizeHitHandlers,
) -> AnyElement {
    let orientation = if edge.is_horizontal_main_axis() {
        ResizablePanelsOrientation::Horizontal
    } else {
        ResizablePanelsOrientation::Vertical
    };

    let hit_px = handle_hit_target_main_axis_px(handle_metrics);
    let hit_main = px(hit_px);
    let hit_inset = px(hit_px * 0.5 - 0.5);

    let mut handle = div()
        .id("slide-panel-resize-handle")
        .absolute()
        .occlude()
        .when(orientation == ResizablePanelsOrientation::Horizontal, |this| this.cursor_col_resize())
        .when(orientation == ResizablePanelsOrientation::Vertical, |this| this.cursor_row_resize());

    handle = match edge {
        SlidePanelEdge::Right => handle.left(-hit_inset).top_0().bottom_0().w(hit_main),
        SlidePanelEdge::Left => handle.right(-hit_inset).top_0().bottom_0().w(hit_main),
        SlidePanelEdge::Bottom => handle.top(-hit_inset).left_0().right_0().h(hit_main),
        SlidePanelEdge::Top => handle.bottom(-hit_inset).left_0().right_0().h(hit_main),
    };

    if handle_active {
        handle = handle
            .when(orientation == ResizablePanelsOrientation::Horizontal, |this| {
                this.flex().justify_center().items_center()
            })
            .when(orientation == ResizablePanelsOrientation::Vertical, |this| {
                this.flex().flex_col().justify_center().items_center()
            })
            .child(render_handle_grip(orientation, look.root().color("grip_emphasis"), handle_metrics));
    }

    let SlidePanelResizeHitHandlers { mouse_down, hover } = handlers;

    handle
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        .on_drag(SlidePanelResizeDrag, |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .into_any_element()
}

fn handle_overlay_geometry(split_px: f32, handle_width_px: f32) -> (f32, f32, f32) {
    let width = handle_width_px.max(1.0).round();
    let width_i = width as i32;
    let half = width_i / 2;
    let origin = split_px - half as f32;
    let divider_local = half as f32;
    (origin, width, divider_local)
}

fn render_handle_grip(
    orientation: ResizablePanelsOrientation,
    grip_color: gpui::Hsla,
    metrics: &ResizeHandleMetrics,
) -> Div {
    let cross = metrics.grip_cross_axis_px;
    let main = metrics.grip_main_axis_px;
    div()
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(0.0))
        .bottom(px(0.0))
        .when(orientation == ResizablePanelsOrientation::Horizontal, |this| {
            this.flex().justify_center().items_center()
        })
        .when(orientation == ResizablePanelsOrientation::Vertical, |this| {
            this.flex().flex_col().justify_center().items_center()
        })
        .child(
            div()
                .rounded(px(metrics.grip_radius))
                .bg(grip_color)
                .when(orientation == ResizablePanelsOrientation::Horizontal, |this| this.w(px(cross)).h(px(main)))
                .when(orientation == ResizablePanelsOrientation::Vertical, |this| this.w(px(main)).h(px(cross))),
        )
}

fn render_handle_interaction_layer(
    orientation: ResizablePanelsOrientation,
    handle_hit: Pixels,
    split_px: f32,
    handle_origin_px: f32,
    handlers: SlidePanelResizeHitHandlers,
) -> AnyElement {
    let hit_px = handle_hit.as_f32();
    let split_local = split_px - handle_origin_px;
    let hit_inset = split_local - hit_px * 0.5;

    let SlidePanelResizeHitHandlers { mouse_down, hover } = handlers;

    match orientation {
        ResizablePanelsOrientation::Horizontal => div()
            .id("slide-panel-resize-hit")
            .absolute()
            .left(px(hit_inset))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(handle_hit)
            .cursor_col_resize()
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_drag(SlidePanelResizeDrag, |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .into_any_element(),
        ResizablePanelsOrientation::Vertical => div()
            .id("slide-panel-resize-hit")
            .absolute()
            .top(px(hit_inset))
            .left(px(0.0))
            .right(px(0.0))
            .h(handle_hit)
            .cursor_row_resize()
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_drag(SlidePanelResizeDrag, |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .into_any_element(),
    }
}
