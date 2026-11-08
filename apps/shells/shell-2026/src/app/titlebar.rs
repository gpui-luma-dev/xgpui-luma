use std::sync::Arc;

use gpui::{Context, IntoElement, Pixels, Window, div, prelude::*, px};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::shell::{TITLE_BAR_LEFT_PADDING, TitleBar};
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, ShadcnSize};
use luma_shell_common::chrome::spawn_theme_toggle_button;
use lucide_svg_static::Icon;

use super::{Shell2026App, style::*};

const BUTTON_SIZE: f32 = 28.0;
const BUTTON_GAP: f32 = 6.0;
const NAVIGATION_LEADING_INSET: f32 = 4.0;
// Four controls, three gaps, and space on either side of the navigation group.
pub(super) const CLOSED_TITLEBAR_WIDTH: Pixels =
    px(BUTTON_SIZE * 4.0 + BUTTON_GAP * 3.0 + NAVIGATION_LEADING_INSET + 12.0);

pub(super) struct TitlebarControls {
    back: IconButton,
    forward: IconButton,
    pub(super) sidebar_toggle: IconButton,
    compose: IconButton,
    pub(super) theme_toggle: IconButton,
    more: IconButton,
    layout: IconButton,
    new_item: IconButton,
}

impl TitlebarControls {
    pub(super) fn new(look: &Arc<ShadcnLook>, cx: &mut Context<Shell2026App>) -> Self {
        // Only the sidebar and theme toggles are wired in this shell demo.
        Self {
            back: icon_button("shell-2026-back", Icon::ArrowLeft, false, look, cx),
            forward: icon_button("shell-2026-forward", Icon::ArrowRight, false, look, cx),
            sidebar_toggle: icon_button("shell-2026-toggle-sidebar", Icon::PanelLeft, true, look, cx),
            compose: icon_button("shell-2026-compose", Icon::SquarePen, false, look, cx),
            theme_toggle: spawn_theme_toggle_button("shell-2026-theme", look, cx),
            more: icon_button("shell-2026-more", Icon::Ellipsis, false, look, cx),
            layout: icon_button("shell-2026-layout", Icon::LayoutGrid, false, look, cx),
            new_item: icon_button("shell-2026-new", Icon::Plus, false, look, cx),
        }
    }

    pub(super) fn render(&self, look: &ShadcnLook, sidebar_width: Pixels, window: &Window) -> impl IntoElement {
        let chrome = look.chrome();
        // TitleBar adds native traffic-light padding and an extra inset in fullscreen.
        let content_origin = TITLE_BAR_LEFT_PADDING
            + if window.is_fullscreen() {
                window.rem_size() * 0.75
            } else {
                px(0.0)
            };
        let compartment_width = titlebar_compartment_width(sidebar_width, content_origin);

        TitleBar::new()
            .height(SHELL_TITLEBAR_HEIGHT)
            .background_color(shell_background(look))
            .border_color(shell_background(look))
            .text_color(chrome.title_text)
            .child(
                div()
                    .size_full()
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
                                    .child(self.back.clone())
                                    .child(self.forward.clone())
                                    .child(self.sidebar_toggle.clone())
                                    .child(self.compose.clone()),
                            )
                            .child(div().flex_1())
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
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px_3()
                            .text_size(px(13.0))
                            .text_color(chrome.title_text)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child("shell-2026"),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(BUTTON_GAP))
                            .pr_2()
                            .child(self.theme_toggle.clone())
                            .child(self.more.clone())
                            .child(self.layout.clone())
                            .child(div().w(px(1.0)).h(px(16.0)).bg(chrome.border))
                            .child(self.new_item.clone()),
                    ),
            )
    }
}

// The splitter's one-pixel stroke starts at the panel boundary. The titlebar
// stroke occupies the last pixel of its compartment, hence the extra pixel.
pub(super) fn titlebar_compartment_width(sidebar_width: Pixels, content_origin: Pixels) -> Pixels {
    // Follow the shrinking pane until reaching the parked position, keeping
    // the separator clear of the navigation controls throughout the transition.
    (RAIL_WIDTH + CANVAS_BORDER + sidebar_width - content_origin + px(1.0)).max(CLOSED_TITLEBAR_WIDTH)
}

fn icon_button(
    id: &'static str,
    icon: Icon,
    enabled: bool,
    look: &ShadcnLook,
    cx: &mut Context<Shell2026App>,
) -> IconButton {
    shadcn::Button::icon_button(id, icon)
        .look(look)
        .content_only()
        .size(ShadcnSize::Sm)
        .enabled(enabled)
        .with_template_modifier(move |root, _| root.debug_selector(move || id.into()).size(px(BUTTON_SIZE)))
        .spawn(cx)
}
