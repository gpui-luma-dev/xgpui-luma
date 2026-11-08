use xgpui_luma::controls::resizable_panels::ResizablePanelsEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, IntoElement, Pixels, Render, Window, div, prelude::*, px, transparent_black};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::resizable_panels::{
    PanelHideMode, ResizablePanelSpec, ResizablePanels, ResizablePanelsEvent, ResizeCollapseBehavior,
    ResizeCollapseDirection, ResizeCollapseMode, ResizeHandleVisibility,
};
use xgpui_luma::controls::sidebar::{SidebarPresentation};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook};
use luma_shell_common::{
    chrome::{HasShellTheme, handle_theme_toggle, render_app_root},
    theme::{ShellThemeChoice, sync_color_control_theme},
};
use lucide_svg_static::Icon;

mod style;
mod titlebar;
#[cfg(test)]
mod tests;

pub(crate) use style::SHELL_TITLEBAR_HEIGHT;
use style::*;
use titlebar::TitlebarControls;

pub struct Shell2026App {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    panels: Entity<ResizablePanels>,
    rail: Entity<xgpui_luma::controls::frame::FrameControl>,
    sidebar_width: Pixels,
    titlebar: TitlebarControls,
}

impl Shell2026App {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        window.focus(&focus_scope, cx);
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let rail = spawn_rail(&look, cx);
        let panels = spawn_panels(&look, cx);
        let titlebar = TitlebarControls::new(&look, cx);

        panels.clone().on_event(cx, |this, event, cx| {
            if let ResizablePanelsEvent::SizesChanged { sizes_px } = event
                && let Some(width) = sizes_px.first()
            {
                this.sidebar_width = px(*width);
                cx.notify();
            }
        });
        titlebar.sidebar_toggle.clone().on_click(cx, |this, cx| {
            this.panels.update(cx, |panels, cx| panels.toggle_panel_hidden(0, PanelHideMode::Completely, cx));
        });
        titlebar.theme_toggle.clone().on_click(cx, |this, cx| {
            handle_theme_toggle(this, cx);
        });

        Self { focus_scope, look, panels, rail, sidebar_width: INITIAL_SIDEBAR_WIDTH, titlebar }
    }
}

impl HasShellTheme for Shell2026App {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }
    fn theme_toggle_button(&self) -> IconButton {
        self.titlebar.theme_toggle.clone()
    }
}

impl Render for Shell2026App {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let body = div()
            .size_full()
            .bg(shell_background(&self.look))
            .flex()
            .child(
                div()
                    .debug_selector(|| "shell-2026-rail-bounds".into())
                    .w(RAIL_WIDTH)
                    .flex_none()
                    .h_full()
                    .child(self.rail.clone()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .mt(CANVAS_TOP_INSET)
                    .mr(CANVAS_EDGE_INSET)
                    .mb(CANVAS_EDGE_INSET)
                    .relative()
                    .rounded(CANVAS_RADIUS)
                    .overflow_hidden()
                    .border(CANVAS_BORDER)
                    .border_color(canvas_border_color(&self.look))
                    // Paint the canvas here: GPUI cannot clip child fills to rounded corners.
                    .bg(chrome.content_background)
                    .child(
                        div()
                            .debug_selector(|| "shell-2026-sidebar-fill".into())
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(self.sidebar_width)
                            .rounded_tl(CANVAS_RADIUS - CANVAS_BORDER)
                            .rounded_bl(CANVAS_RADIUS - CANVAS_BORDER)
                            .bg(self.look.token_color("sidebar").unwrap_or(chrome.panel_background)),
                    )
                    .child(div().relative().size_full().child(self.panels.clone())),
            );
        render_app_root(
            &self.focus_scope,
            &self.look,
            self.titlebar.render(&self.look, self.sidebar_width, window),
            body,
        )
    }
}

fn spawn_rail(
    look: &Arc<ShadcnLook>,
    cx: &mut Context<Shell2026App>,
) -> Entity<xgpui_luma::controls::frame::FrameControl> {
    let mut menu = shadcn::Sidebar::menu("shell-2026-destinations");
    for (id, label, icon) in [
        ("home", "Home", Icon::House),
        ("history", "History", Icon::Clock),
        ("library", "Library", Icon::BookOpen),
        ("gallery", "Gallery", Icon::Image),
        ("activity", "Activity", Icon::AtSign),
        ("more", "More", Icon::Ellipsis),
    ] {
        menu = menu.item(shadcn::Sidebar::menu_item(id, label).icon(icon).active(id == "home"));
    }
    // Keep the rail in icon mode independently of the inner resizable sidebar.
    let navigation = shadcn::Sidebar::new("shell-2026-rail")
        .look(look)
        .presentation(SidebarPresentation::Icons)
        .motion_enabled(false)
        .auto_hide_scrollbar(true)
        .sidebar(
            shadcn::Sidebar::panel("shell-2026-rail-panel")
                .content(shadcn::Sidebar::content().group(shadcn::Sidebar::group().menu(menu)))
                .footer(
                    shadcn::Sidebar::footer()
                        .child(shadcn::Sidebar::menu_item("help", "Help").icon(Icon::CircleQuestionMark))
                        .child(shadcn::Sidebar::menu_item("profile", "Profile").icon(Icon::CircleUser)),
                ),
        )
        .spawn(cx);
    shadcn::Frame::sidebar("shell-2026-rail-frame")
        .look(look)
        .bg(transparent_black())
        .rounded(px(0.0))
        .child(navigation)
        .spawn(cx)
}

fn spawn_panels(look: &ShadcnLook, cx: &mut Context<Shell2026App>) -> Entity<ResizablePanels> {
    let sidebar = shadcn::Sidebar::new("shell-2026-sidebar")
        .look(look)
        .sidebar(shadcn::Sidebar::panel("shell-2026-sidebar-panel"))
        .spawn(cx);
    look.resizable_panels("shell-2026-panels")
        .show_border(false)
        .motion_enabled(true)
        .handle_visibility(ResizeHandleVisibility::Hover)
        .double_click_collapse(Some(ResizeCollapseBehavior::new(
            ResizeCollapseMode::Completely,
            ResizeCollapseDirection::Left,
        )))
        .panel(
            ResizablePanelSpec::new_render(move || div().size_full().p(px(8.0)).child(sidebar.clone()))
                .size(INITIAL_SIDEBAR_WIDTH)
                .min(MIN_SIDEBAR_WIDTH)
                .max(MAX_SIDEBAR_WIDTH),
        )
        .panel(
            ResizablePanelSpec::new_render(|| div().debug_selector(|| "shell-2026-workspace".into()).size_full())
                .weight(1.0)
                .min(MIN_WORKSPACE_WIDTH),
        )
        .spawn(cx)
}
