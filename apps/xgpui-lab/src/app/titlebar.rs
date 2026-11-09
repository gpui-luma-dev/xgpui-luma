//! Lab shell composition: window setup, toolbar layout, controls and callbacks.
#[cfg(target_os = "macos")]
mod native_appearance;
use std::{cell::Cell, collections::HashMap, rc::Rc};
use gpui::{prelude::*, *};
use xgpui_shell::{ShellApp, ShellOptions, ShellWindow, ShellWindowTitle, ShellDragRegion};
use xgpui_luma::focus::LumaFocusScopeExt;
use xgpui_luma_look_radix::Look;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::theme::{LumaThemeSyncExt, ThemeMode};
use xgpui_luma::controls::resizable_panels::PanelHideMode;
use crate::theme::ColorVizLookExt;
use lucide_svg_static::Icon;
use super::{XgpuiLabApp, shell_style::*};

const BUTTON_SIZE: f32 = 28.0;
const BUTTON_GAP: f32 = 6.0;
const NAVIGATION_LEADING_INSET: f32 = 4.0;
pub(super) const CLOSED_TITLEBAR_WIDTH: Pixels =
    px(BUTTON_SIZE * 4.0 + BUTTON_GAP * 3.0 + NAVIGATION_LEADING_INSET + 12.0);

pub(crate) fn open_shell(cx: &mut App) -> anyhow::Result<()> {
    open_shell_window(cx)?;
    cx.activate(true);
    Ok(())
}

/// The production composition is also used by headless shell interaction tests.
pub(super) fn open_shell_window(cx: &mut App) -> anyhow::Result<(WindowHandle<ShellWindow>, Entity<XgpuiLabApp>)> {
    let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
    let mut content = None;
    let window = cx.open_shell_window_with_titlebar(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            #[cfg(target_os = "macos")]
            window_background: WindowBackgroundAppearance::Opaque,
            window_min_size: Some(size(px(720.0), px(480.0))),
            ..Default::default()
        },
        ShellOptions::default(),
        |window, cx| {
            window.set_shell_title("XGPUI Lab");
            let app = cx.new(|cx| XgpuiLabApp::new(window, cx));
            content = Some(app.clone());
            let titlebar = cx.new(|cx| {
                LabTitlebar::new(&app, window, cx, |titlebar, app, window, cx| {
                    let chrome = app.look.chrome();
                    let compartment_width =
                        titlebar_compartment_width(app.sidebar_width, titlebar.content_origin - app.body_origin);
                    div()
                        .size_full()
                        // The app chooses its toolbar background from Radix.
                        .bg(shell_background(&app.look, window.is_window_active()))
                        // Optical alignment with the native macOS traffic lights.
                        // Bottom padding raises the row center without shifting its background.
                        .pb(if cx.platform_info().os == OperatingSystem::Mac && !window.is_fullscreen() {
                            px(4.)
                        } else {
                            px(0.)
                        })
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .h_full()
                                .w(compartment_width)
                                .flex_none()
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .pl(px(NAVIGATION_LEADING_INSET))
                                        .flex()
                                        .items_center()
                                        .gap(px(BUTTON_GAP))
                                        .child(titlebar.button("shell-2026-back", cx, |cx| {
                                            icon_button("shell-2026-back", Icon::ArrowLeft, &app.look)
                                                .enabled(false)
                                                .spawn(cx)
                                        }))
                                        .child(titlebar.button("shell-2026-forward", cx, |cx| {
                                            icon_button("shell-2026-forward", Icon::ArrowRight, &app.look)
                                                .enabled(false)
                                                .spawn(cx)
                                        }))
                                        .child(titlebar.button("shell-2026-toggle-sidebar", cx, |cx| {
                                            icon_button("shell-2026-toggle-sidebar", Icon::PanelLeft, &app.look)
                                                .on_click(cx, |app: &mut XgpuiLabApp, cx| {
                                                    app.panels.update(cx, |panels, cx| {
                                                        panels.toggle_panel_hidden(0, PanelHideMode::Completely, cx)
                                                    });
                                                })
                                                .spawn(cx)
                                        }))
                                        .child(titlebar.button("shell-2026-compose", cx, |cx| {
                                            icon_button("shell-2026-compose", Icon::SquarePen, &app.look)
                                                .enabled(false)
                                                .spawn(cx)
                                        })),
                                )
                                .child(ShellDragRegion::new("lab-navigation-drag", div().size_full()))
                                // The short separator is independent of the full-height body splitter.
                                .child(
                                    div()
                                        .debug_selector(|| "shell-2026-separator".into())
                                        .flex_none()
                                        .w(px(1.0))
                                        .h(px(20.0))
                                        .bg(chrome.border),
                                ),
                        )
                        .child(ShellDragRegion::new(
                            "lab-title-drag",
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .px_3()
                                .text_size(px(13.0))
                                .text_color(chrome.title_text)
                                .child(
                                    div()
                                        .debug_selector(|| "lab-title-label".into())
                                        .min_w_0()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child("XGPUI Lab"),
                                ),
                        ))
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(BUTTON_GAP))
                                .pr_2()
                                .child(titlebar.button("shell-2026-theme", cx, |cx| {
                                    icon_button("shell-2026-theme", Icon::Sun, &app.look)
                                        .content({
                                            let look = app.look.clone();
                                            move |model, _| {
                                                let icon = if look.mode() == ThemeMode::Dark {
                                                    Icon::Sun
                                                } else {
                                                    Icon::Moon
                                                };
                                                xgpui_luma::infra::icon::lucide_icon(
                                                    icon,
                                                    model.look.foreground,
                                                    model.look.icon_size,
                                                )
                                            }
                                        })
                                        .on_click(cx, |app: &mut XgpuiLabApp, cx| {
                                            let mode = match app.look.mode() {
                                                ThemeMode::Light => ThemeMode::Dark,
                                                ThemeMode::Dark => ThemeMode::Light,
                                            };
                                            app.look.set_mode(mode);
                                            super::sync_color_control_theme(&app.look);
                                            cx.bump_luma_theme_revision();
                                            cx.notify();
                                        })
                                        .spawn(cx)
                                }))
                                .child(titlebar.button("shell-2026-more", cx, |cx| {
                                    icon_button("shell-2026-more", Icon::Ellipsis, &app.look).enabled(false).spawn(cx)
                                }))
                                .child(titlebar.button("shell-2026-layout", cx, |cx| {
                                    icon_button("shell-2026-layout", Icon::LayoutGrid, &app.look)
                                        .enabled(false)
                                        .spawn(cx)
                                }))
                                .child(div().w(px(1.0)).h(px(16.0)).bg(chrome.border))
                                .child(titlebar.button("shell-2026-new", cx, |cx| {
                                    icon_button("shell-2026-new", Icon::Plus, &app.look).enabled(false).spawn(cx)
                                })),
                        )
                        .into_any_element()
                })
            });
            (titlebar, app)
        },
    )?;
    // The shell invokes its content builder synchronously on successful open.
    let content = content.ok_or_else(|| anyhow::anyhow!("shell did not construct Lab content"))?;
    Ok((window, content))
}

// The splitter's one-pixel stroke starts at the panel boundary. The titlebar
// stroke occupies the last pixel of its compartment, hence the extra pixel.
pub(super) fn titlebar_compartment_width(sidebar_width: Pixels, content_origin: Pixels) -> Pixels {
    // Follow the shrinking pane until reaching the parked position, keeping
    // the separator clear of the navigation controls throughout the transition.
    (RAIL_WIDTH + CANVAS_BORDER + sidebar_width - content_origin + px(1.0)).max(CLOSED_TITLEBAR_WIDTH)
}

fn icon_button(id: &'static str, icon: Icon, look: &Look) -> xgpui_luma_look_radix::Button {
    xgpui_luma_look_radix::Button::new(id)
        .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .content(move |model, _| {
            xgpui_luma::infra::icon::lucide_icon(icon, model.look.foreground, model.look.icon_size)
        })
        .look(look)
        .content_only()
        .size(xgpui_luma_look_radix::ButtonSize::Two)
        .with_template_modifier(move |root, _| root.debug_selector(move || id.into()).size(px(BUTTON_SIZE)))
}

type Button = xgpui_luma::controls::button::Button;
type TitlebarTemplate =
    Box<dyn Fn(&mut TitlebarState, &mut XgpuiLabApp, &mut Window, &mut Context<XgpuiLabApp>) -> AnyElement>;

/// Retain controls across renders, so inline declarations bind callbacks only once.
#[derive(Default)]
struct TitlebarState {
    buttons: HashMap<&'static str, Entity<Button>>,
    content_origin: Pixels,
}

impl TitlebarState {
    fn button(
        &mut self,
        id: &'static str,
        cx: &mut Context<XgpuiLabApp>,
        build: impl FnOnce(&mut Context<XgpuiLabApp>) -> Entity<Button>,
    ) -> Entity<Button> {
        self.buttons.entry(id).or_insert_with(|| build(cx)).clone()
    }
}

/// Rendering/lifecycle adapter; all application UI is declared above at the shell call.
struct LabTitlebar {
    app: WeakEntity<XgpuiLabApp>,
    state: TitlebarState,
    template: TitlebarTemplate,
    focus_scope: FocusHandle,
    _subscription: Subscription,
    _activation_subscription: Subscription,
    #[cfg(target_os = "macos")]
    native_mode: Option<ThemeMode>,
}

impl LabTitlebar {
    fn new(
        app: &Entity<XgpuiLabApp>,
        window: &mut Window,
        cx: &mut Context<Self>,
        template: impl Fn(&mut TitlebarState, &mut XgpuiLabApp, &mut Window, &mut Context<XgpuiLabApp>) -> AnyElement
        + 'static,
    ) -> Self {
        Self {
            app: app.downgrade(),
            state: TitlebarState::default(),
            template: Box::new(template),
            focus_scope: cx.focus_handle(),
            _subscription: cx.observe(app, |_, _, cx| cx.notify()),
            _activation_subscription: cx.observe_window_activation(window, |_, _, cx| cx.notify()),
            #[cfg(target_os = "macos")]
            native_mode: None,
        }
    }
}

impl Render for LabTitlebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(app) = self.app.upgrade() else {
            return div().into_any_element();
        };
        #[cfg(target_os = "macos")]
        {
            let mode = app.read(cx).look.mode();
            if !cx.is_test() && self.native_mode != Some(mode) {
                self.native_mode = Some(mode);
                // Apply after rendering, outside AppKit's layout callbacks. This changes
                // this window's native controls, never the system or other app windows.
                _window.defer(cx, move |window, _cx| {
                    if let Err(error) = native_appearance::sync(window, mode) {
                        log::warn!("could not synchronize Lab native appearance: {error:#}");
                    }
                    window.refresh();
                });
            }
        }
        let composition = app.update(cx, |app, cx| (self.template)(&mut self.state, app, _window, cx));
        let header = cx.entity().downgrade();
        let background = shell_background(&app.read(cx).look, _window.is_window_active());
        let mac_native = cx.platform_info().os == OperatingSystem::Mac;
        let measured_toolbar = Rc::new(Cell::new(Bounds::default()));
        let background_bounds = measured_toolbar.clone();
        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .font_family("System UI")
            .relative()
            .debug_selector(|| "lab-toolbar-bounds".into())
            .child(composition)
            .when(mac_native, |root| {
                // Deferred painting escapes the shell's custom-content clip, while
                // restricting paint to empty gutters and the bottom seam.
                root.child(deferred(
                    canvas(
                        |_, _, _| (),
                        move |_, _, window, _| {
                            for edge in titlebar_background_edges(background_bounds.get(), window.viewport_size()) {
                                window.paint_quad(fill(edge, background));
                            }
                        },
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                ))
            })
            .child(
                canvas(
                    move |bounds, _, cx| {
                        measured_toolbar.set(bounds);
                        let _ = header.update(cx, |header, cx| {
                            if header.state.content_origin != bounds.origin.x {
                                header.state.content_origin = bounds.origin.x;
                                cx.notify();
                            }
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .into_any_element()
    }
}

/// Exposed macOS background only: native-control gutter, trailing inset and seam.
fn titlebar_background_edges(toolbar: Bounds<Pixels>, viewport: Size<Pixels>) -> [Bounds<Pixels>; 3] {
    [
        Bounds::new(point(px(0.), toolbar.top()), size(toolbar.left().max(px(0.)), toolbar.size.height)),
        Bounds::new(
            point(toolbar.right(), toolbar.top()),
            size((viewport.width - toolbar.right()).max(px(0.)), toolbar.size.height),
        ),
        Bounds::new(point(px(0.), toolbar.bottom()), size(viewport.width, px(1.))),
    ]
}
