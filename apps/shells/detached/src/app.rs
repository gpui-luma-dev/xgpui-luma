use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FocusHandle, IntoElement, ParentElement, Render, Styled, Subscription, Window, div,
    px, rgb,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::split_view::{SplitView, SplitViewSeparatorVisibility, render_pane};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use luma_shell_common::{
    chrome::{
        HasShellTheme, handle_theme_toggle, render_app_root, render_title_bar, spawn_theme_toggle_button,
        wrap_content_pane,
    },
    nav_sample,
    split_sync::wire_split_nav_sync,
    theme::{ShellThemeChoice, sync_color_control_theme},
};
use lucide_svg_static::Icon as LucideIcon;

const SIDEBAR_PADDING: f32 = 10.0;
// Keep the icon rail inside the detached card visible when collapsed.
const SIDEBAR_COLLAPSED_WIDTH: f32 = 56.0 + 2.0 * SIDEBAR_PADDING;

pub struct DetachedShellApp {
    focus_scope: FocusHandle,
    pane_focus: FocusHandle,
    look: Arc<ShadcnLook>,
    split_view: Entity<SplitView>,
    sidebar: Entity<xgpui_luma::controls::frame::FrameControl>,
    toggle_button: IconButton,
    theme_toggle_button: IconButton,
    _subscriptions: Vec<Subscription>,
}

impl DetachedShellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let pane_focus = cx.focus_handle().tab_stop(true);
        window.focus(&focus_scope, cx);
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let sidebar = nav_sample::spawn_properties_sidebar(look.clone(), "shell-nav", cx);
        let toggle_button = shadcn::Button::icon_button("shell-detached-toggle", LucideIcon::Menu)
            .look(look.as_ref())
            .ghost()
            .spawn(cx);
        let theme_toggle_button = spawn_theme_toggle_button("shell-titlebar-theme-toggle-detached", &look, cx);
        let split_view = look
            .split_view("shell-detached")
            .sidebar_width(px(560.0))
            .sidebar_min_width(px(200.0))
            .sidebar_max_width(px(840.0))
            .sidebar_collapsed_width(px(SIDEBAR_COLLAPSED_WIDTH))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .spawn(cx);

        let subscriptions = Vec::new();
        wire_split_nav_sync(split_view.clone(), sidebar.clone(), cx);
        theme_toggle_button.clone().on_click(cx, |this, cx| {
            handle_theme_toggle(this, cx);
        });
        toggle_button.clone().on_click(cx, {
            let split_view = split_view.downgrade();
            move |_, cx| {
                let Some(split_view) = split_view.upgrade() else {
                    return;
                };

                split_view.update(cx, |split_view, cx| {
                    split_view.toggle_collapsed(cx);
                });
            }
        });

        let sidebar = shadcn::Frame::sidebar("detached-sidebar-frame")
            .look(&look)
            .rounded(px(16.0))
            .overflow_hidden()
            .child(sidebar)
            .spawn(cx);
        Self {
            focus_scope,
            pane_focus,
            look,
            split_view,
            sidebar,
            toggle_button,
            theme_toggle_button,
            _subscriptions: subscriptions,
        }
    }
}

impl HasShellTheme for DetachedShellApp {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }

    fn theme_toggle_button(&self) -> IconButton {
        self.theme_toggle_button.clone()
    }
}

impl Render for DetachedShellApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let sidebar = self.sidebar.clone();
        let look = self.look.clone();
        let sans_family = look.mode_tokens().typography.font.sans.family.clone();
        let toggle_button = self.toggle_button.clone();

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || detached_nav_pane_with_sidebar(sidebar.clone())),
                render_pane(move || {
                    wrap_content_pane(
                        detached_content_pane(toggle_button.clone()),
                        pane_focus.clone(),
                        &look,
                        sans_family.clone(),
                    )
                }),
                cx,
            );
        });

        render_app_root(
            &self.focus_scope,
            &self.look,
            render_title_bar("Shell: Detached", self, cx),
            self.split_view.clone(),
        )
    }
}

fn detached_nav_pane_with_sidebar(sidebar: Entity<xgpui_luma::controls::frame::FrameControl>) -> AnyElement {
    div().size_full().p(px(SIDEBAR_PADDING)).child(sidebar).into_any_element()
}

fn detached_content_pane(toggle_button: IconButton) -> AnyElement {
    div()
        .size_full()
        .p_3()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(div().w_full().flex().items_center().child(toggle_button))
                .child(div().flex_1().mt_2().bg(rgb(0x800080)).rounded_md()),
        )
        .into_any_element()
}
