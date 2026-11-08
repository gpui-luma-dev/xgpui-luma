use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, ParentElement, Render, div, prelude::*};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::focus::LumaFocusScopeExt;
use xgpui_luma::shell::TitleBar;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use xgpui_luma_look_shadcn::{ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::theme::toggle_shell_theme;

pub trait HasShellTheme {
    fn look(&self) -> &Arc<ShadcnLook>;
    fn theme_toggle_button(&self) -> IconButton;
}

pub fn wrap_content_pane(
    content: AnyElement,
    pane_focus: FocusHandle,
    look: &ShadcnLook,
    sans_family: String,
) -> AnyElement {
    let focus = pane_focus.clone();
    let chrome = look.chrome();

    div()
        .size_full()
        .font_family(sans_family)
        .bg(chrome.content_background)
        .track_focus(&pane_focus)
        .capture_any_mouse_down(move |event, window, cx| {
            if event.button == MouseButton::Left {
                window.focus(&focus, cx);
            }
        })
        .child(content)
        .into_any_element()
}

pub fn spawn_theme_toggle_button<T: 'static>(
    id: &'static str,
    look: &Arc<ShadcnLook>,
    cx: &mut Context<T>,
) -> IconButton {
    let button = shadcn::Button::icon_button(id, theme_toggle_icon(look.mode()))
        .look(look.as_ref())
        .content_only()
        .spawn(cx);
    sync_theme_toggle_button(&button, look, cx);
    button
}

pub fn sync_theme_toggle_button<T: 'static>(button: &IconButton, look: &ShadcnLook, cx: &mut Context<T>) {
    button.update(cx, |button, cx| {
        button.set_presenter(theme_toggle_presenter(theme_toggle_icon(look.mode()), shell_icon_color(look)), cx);
    });
}

pub fn handle_theme_toggle<T: Render + HasShellTheme>(app: &mut T, cx: &mut Context<T>) {
    toggle_shell_theme(app.look(), cx);
    sync_theme_toggle_button(&app.theme_toggle_button(), app.look(), cx);
    cx.notify();
}

pub fn render_title_bar<T: Render + HasShellTheme>(title: &str, app: &T, _cx: &mut Context<T>) -> impl IntoElement {
    let look = app.look();
    let chrome = look.chrome();
    let sans_family = look.mode_tokens().typography.font.sans.family.clone();

    TitleBar::new()
        .background_color(chrome.panel_background)
        .border_color(chrome.border)
        .text_color(chrome.title_text)
        .child(
            div()
                .id("shell-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans_family)
                .child(div().child(title.to_string()))
                .child(app.theme_toggle_button()),
        )
}

fn theme_toggle_icon(mode: ThemeMode) -> LucideIcon {
    match mode {
        ThemeMode::Light => LucideIcon::Moon,
        ThemeMode::Dark => LucideIcon::Sun,
    }
}

fn theme_toggle_presenter(icon: LucideIcon, color: gpui::Hsla) -> ControlPresenter<ButtonContentContext<()>> {
    Arc::new(move |_, _| div().child(xgpui_luma::infra::icon::lucide_icon(icon, color, 14.0)).into_any_element())
}

fn shell_icon_color(look: &ShadcnLook) -> gpui::Hsla {
    look.resolve_content_only_button(ButtonFamilyRole::Icon, ControlSize::Sm, InteractionState::default())
        .foreground
}

pub fn render_app_root(
    focus_scope: &gpui::FocusHandle,
    look: &ShadcnLook,
    title_bar: impl IntoElement,
    body: impl IntoElement,
) -> impl IntoElement {
    let chrome = look.chrome();
    let sans_family = look.mode_tokens().typography.font.sans.family.clone();

    div()
        .luma_focus_scope(focus_scope)
        .size_full()
        .flex()
        .flex_col()
        .font_family(sans_family)
        .bg(chrome.app_background)
        .child(title_bar)
        .child(div().flex_1().min_h_0().child(body))
}
