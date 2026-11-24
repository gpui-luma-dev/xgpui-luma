use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Decorations, Hsla, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, Point, Render, RenderOnce, StatefulInteractiveElement as _,
    StyleRefinement, Styled, TitlebarOptions, Window, WindowControlArea, div, point, prelude::*, px,
};
use crate::infra::icon::{IconSource, render_icon_source};
use crate::theme::stylesheet::{CommonStylesheet, IconRole, ShellGeometry};

pub const TITLE_BAR_HEIGHT: Pixels = px(34.0);
#[cfg(target_os = "macos")]
pub const TITLE_BAR_LEFT_PADDING: Pixels = px(80.0);
#[cfg(not(target_os = "macos"))]
pub const TITLE_BAR_LEFT_PADDING: Pixels = px(12.0);
const TITLE_BAR_DRAG_THRESHOLD_PX: f64 = 4.0;

/// Check if the left mouse button is currently pressed at the OS level.
///
/// On macOS, `window.start_window_move()` delegates to `-[NSWindow performWindowDragWithEvent:]`,
/// passing `[NSApp currentEvent]`. If the drag is initiated after the mouse button has already been
/// released (e.g. rapid flick/click or delayed runloop dispatch), AppKit logs:
/// "Warning: Window move completed without beginning".
///
/// Checking the real-time button state ensures we only initiate native window dragging while the
/// left mouse button is physically held down.
#[inline]
pub fn is_left_mouse_down() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn CGEventSourceButtonState(state_id: i32, button: u32) -> bool;
        }
        unsafe { CGEventSourceButtonState(0, 0) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

type CloseWindowHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// TitleBar used to customize the look of the title bar.
///
/// You can put arbitrary elements inside the title bar.
#[derive(IntoElement)]
pub struct TitleBar {
    style: StyleRefinement,
    height: Option<Pixels>,
    children: Vec<AnyElement>,
    on_close_window: Option<CloseWindowHandler>,
    background_color: Option<Hsla>,
    border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    stylesheet: CommonStylesheet,
}

impl TitleBar {
    /// Create a new TitleBar.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            height: None,
            children: Vec::new(),
            on_close_window: None,
            background_color: None,
            border_color: None,
            text_color: None,
            stylesheet: CommonStylesheet::sdk_default().clone(),
        }
    }

    /// Capture shared shell chrome; explicit color and height setters remain final.
    pub fn stylesheet(mut self, stylesheet: &CommonStylesheet) -> Self {
        self.stylesheet = stylesheet.with_sdk_defaults();
        self
    }

    /// Default title bar options compatible with this `TitleBar`.
    pub fn title_bar_options() -> TitlebarOptions {
        TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some({
                let inset = ShellGeometry::sdk().traffic_light_inset;
                point(px(inset), px(inset))
            }),
        }
    }

    /// Set the title bar height. Defaults to [`TITLE_BAR_HEIGHT`].
    /// Adjust native traffic-light positioning in the window options when needed.
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Set custom close-window behavior.
    ///
    /// Linux only; on other platforms this is ignored.
    pub fn on_close_window(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        if cfg!(target_os = "linux") {
            self.on_close_window = Some(Rc::new(f));
        }
        self
    }

    /// Set a custom title bar background color.
    pub fn background_color(mut self, color: Hsla) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set a custom title bar bottom border color.
    pub fn border_color(mut self, color: Hsla) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set a custom title bar text color (used for window control icons on Windows/Linux).
    pub fn text_color(mut self, color: Hsla) -> Self {
        self.text_color = Some(color);
        self
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
enum ControlIcon {
    Minimize,
    Restore,
    Maximize,
    Close { on_close_window: Option<CloseWindowHandler> },
}

impl ControlIcon {
    fn minimize() -> Self {
        Self::Minimize
    }

    fn restore() -> Self {
        Self::Restore
    }

    fn maximize() -> Self {
        Self::Maximize
    }

    fn close(on_close_window: Option<CloseWindowHandler>) -> Self {
        Self::Close { on_close_window }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Restore => "restore",
            Self::Maximize => "maximize",
            Self::Close { .. } => "close",
        }
    }

    fn icon(&self, stylesheet: &CommonStylesheet) -> IconSource {
        stylesheet.icons.resolve(match self {
            Self::Minimize => IconRole::WindowMinimize,
            Self::Restore => IconRole::WindowRestore,
            Self::Maximize => IconRole::WindowMaximize,
            Self::Close { .. } => IconRole::WindowClose,
        })
    }

    fn window_control_area(&self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Restore | Self::Maximize => WindowControlArea::Max,
            Self::Close { .. } => WindowControlArea::Close,
        }
    }

    fn is_close(&self) -> bool {
        matches!(self, Self::Close { .. })
    }
}

fn render_control_icon(icon: ControlIcon, text_color: Hsla, stylesheet: &CommonStylesheet) -> AnyElement {
    let is_linux = cfg!(target_os = "linux");
    let is_windows = cfg!(target_os = "windows");

    let tokens = stylesheet
        .theme_tokens(crate::theme::ThemeMode::Light, crate::color::GamutMapping::CssLocalMinde)
        .unwrap_or_default();
    let resolved_palette = stylesheet.recipes.palette("shell", &tokens.palette);
    let palette = &resolved_palette.shell;
    let hover_fg = if icon.is_close() {
        palette.close_foreground
    } else {
        text_color
    };
    let hover_bg = if icon.is_close() {
        palette.close_hover_background
    } else {
        palette.hover_background
    };
    let active_bg = if icon.is_close() {
        palette.close_active_background
    } else {
        palette.active_background
    };
    let icon_for_click = icon.clone();
    let on_close_window = match &icon {
        ControlIcon::Close { on_close_window } => on_close_window.clone(),
        _ => None,
    };

    div()
        .id(icon.id())
        .flex()
        .w(px(ShellGeometry::sdk().height))
        .h_full()
        .flex_shrink_0()
        .justify_center()
        .items_center()
        .text_color(text_color)
        .hover(|style| style.bg(hover_bg).text_color(hover_fg))
        .active(|style| style.bg(active_bg).text_color(hover_fg))
        .when(is_windows, |this| this.window_control_area(icon.window_control_area()))
        .when(is_linux, |this| {
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                match &icon_for_click {
                    ControlIcon::Minimize => window.minimize_window(),
                    ControlIcon::Restore | ControlIcon::Maximize => window.zoom_window(),
                    ControlIcon::Close { .. } => {
                        if let Some(f) = on_close_window.as_ref() {
                            f(&ClickEvent::default(), window, cx);
                        } else {
                            window.remove_window();
                        }
                    }
                }
            })
        })
        .child(render_icon_source(&icon.icon(stylesheet), text_color, ShellGeometry::sdk().icon_size))
        .into_any_element()
}

fn render_window_controls(
    window: &mut Window,
    on_close_window: Option<CloseWindowHandler>,
    text_color: Hsla,
    stylesheet: &CommonStylesheet,
) -> AnyElement {
    if cfg!(target_os = "macos") {
        return div().id("window-controls").into_any_element();
    }

    div()
        .id("window-controls")
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .h_full()
        .child(render_control_icon(ControlIcon::minimize(), text_color, stylesheet))
        .child(render_control_icon(
            if window.is_maximized() {
                ControlIcon::restore()
            } else {
                ControlIcon::maximize()
            },
            text_color,
            stylesheet,
        ))
        .child(render_control_icon(ControlIcon::close(on_close_window), text_color, stylesheet))
        .into_any_element()
}

impl Styled for TitleBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct TitleBarState {
    drag_start_position: Option<Point<Pixels>>,
}

// Keep this lightweight state render to satisfy `use_state` requirements.
impl Render for TitleBarState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_client_decorated = matches!(window.window_decorations(), Decorations::Client { .. });
        let is_linux = cfg!(target_os = "linux");
        let is_macos = cfg!(target_os = "macos");
        let tokens = self
            .stylesheet
            .theme_tokens(crate::theme::ThemeMode::Light, crate::color::GamutMapping::CssLocalMinde)
            .unwrap_or_default();
        let resolved_palette = self.stylesheet.recipes.palette("shell", &tokens.palette);
        let palette = &resolved_palette.shell;
        let border_color = self.border_color.unwrap_or(palette.border);
        let background_color = self.background_color.unwrap_or(palette.background);
        let text_color = self.text_color.unwrap_or(palette.foreground);
        let geometry = ShellGeometry::sdk();

        let state = window.use_state(cx, |_, _| TitleBarState { drag_start_position: None });

        div().flex_shrink_0().child(
            div()
                .id("title-bar")
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(self.height.unwrap_or_else(|| px(geometry.height)))
                .pl(px(if is_macos {
                    geometry.macos_left_padding
                } else {
                    geometry.left_padding
                }))
                .border_b(px(geometry.border_width))
                .border_color(border_color)
                .bg(background_color)
                .text_color(text_color)
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
                })
                .child(
                    div()
                        .id("bar")
                        .window_control_area(WindowControlArea::Drag)
                        .when(window.is_fullscreen(), |this| this.pl(px(geometry.fullscreen_left_padding)))
                        .h_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .flex_shrink_0()
                        .flex_1()
                        .when(is_linux && is_client_decorated, |this| {
                            this.child(
                                div()
                                    .top_0()
                                    .left_0()
                                    .absolute()
                                    .size_full()
                                    .h_full()
                                    .on_mouse_down(MouseButton::Right, move |ev, window, _| {
                                        window.show_window_menu(ev.position)
                                    }),
                            )
                        })
                        .children(self.children),
                )
                .child(render_window_controls(window, self.on_close_window, text_color, &self.stylesheet)),
        )
    }
}
