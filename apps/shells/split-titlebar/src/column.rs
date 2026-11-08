use gpui::{
    Context, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Render, Window, WindowControlArea, div,
    prelude::*, px,
};
use xgpui_luma::shell::{TITLE_BAR_HEIGHT, TITLE_BAR_LEFT_PADDING, is_left_mouse_down};

const TITLE_BAR_DRAG_THRESHOLD_PX: f64 = 4.0;

struct TitleBarDragState {
    drag_start_position: Option<Point<Pixels>>,
}

impl Render for TitleBarDragState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
    }
}

pub struct SplitColumn {
    background: Hsla,
    traffic_light_inset: bool,
}

impl SplitColumn {
    pub fn new(background: Hsla, traffic_light_inset: bool) -> Self {
        Self { background, traffic_light_inset }
    }
}

impl Render for SplitColumn {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let background = self.background;
        let traffic_light_inset = self.traffic_light_inset;
        let state = window.use_state(cx, |_, _| TitleBarDragState { drag_start_position: None });

        let is_linux = cfg!(target_os = "linux");
        let is_macos = cfg!(target_os = "macos");

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .id(if traffic_light_inset {
                        "split-column-title-left"
                    } else {
                        "split-column-title-right"
                    })
                    .h(TITLE_BAR_HEIGHT)
                    .flex_shrink_0()
                    .w_full()
                    .when(traffic_light_inset, |title| title.pl(TITLE_BAR_LEFT_PADDING))
                    .bg(background)
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
                        state.drag_start_position = None;
                    }))
                    // Clear drag tracking if the left mouse button was released outside the titlebar element.
                    .on_mouse_up_out(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.drag_start_position = None;
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        window.listener_for(&state, |state, event: &MouseDownEvent, _, _| {
                            state.drag_start_position = (event.click_count < 2).then_some(event.position);
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.drag_start_position = None;
                        }),
                    )
                    .on_mouse_move(window.listener_for(&state, |state, event: &MouseMoveEvent, window, _| {
                        // Only initiate window move if the user is actively dragging with the left button
                        // and the button is physically held down. Calling `start_window_move` after button
                        // release triggers macOS AppKit "Warning: Window move completed without beginning".
                        if !event.dragging() || !is_left_mouse_down() {
                            state.drag_start_position = None;
                            return;
                        }

                        if let Some(origin) = state.drag_start_position
                            && event.position.relative_to(&origin).magnitude() >= TITLE_BAR_DRAG_THRESHOLD_PX
                        {
                            state.drag_start_position = None;
                            window.start_window_move();
                        }
                    }))
                    .when(is_macos, |this| {
                        this.on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                            if ev.click_count >= 2 {
                                cx.stop_propagation();
                                if !window.is_fullscreen() {
                                    window.zoom_window();
                                }
                            }
                        })
                    })
                    .when(is_linux, |this| {
                        this.on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                            if ev.click_count >= 2 {
                                cx.stop_propagation();
                                window.zoom_window();
                            }
                        })
                    }),
            )
            .child(div().flex_1().min_h(px(0.0)).bg(background))
    }
}
