use std::sync::Arc;

use gpui::{
    Context, Entity, FocusHandle, Hsla, IntoElement, Render, Window, WindowBackgroundAppearance, div, prelude::*, px,
    rgb, transparent_black,
};
use xgpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation,
};
use xgpui_luma::focus::LumaFocusScopeExt;
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_shadcn::ShadcnLook;
use luma_shell_common::theme::{ShellThemeChoice, sync_color_control_theme};

use crate::column::SplitColumn;

pub struct SplitTitlebarShellApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    panels: Entity<ResizablePanels>,
}

impl SplitTitlebarShellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let chrome = look.chrome();
        // Let the panel own the fill so its title bar and body share one transparency layer.
        let left_background = transparent_black();
        let right_background = chrome.content_background;

        let left_column = cx.new(|_| SplitColumn::new(left_background, true));
        let right_column = cx.new(|_| SplitColumn::new(right_background, false));

        let panels = look
            .resizable_panels("shell-full-window")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_border(false)
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render({
                    let left_column = left_column.clone();
                    move || left_column.clone()
                })
                .weight(3.0)
                .bg(left_background)
                .min(px(180.0)),
                ResizablePanelSpec::new_render({
                    let right_column = right_column.clone();
                    move || right_column.clone()
                })
                .weight(7.0)
                .bg(right_background)
                .min(px(180.0)),
            ])
            .spawn(cx);

        let mut app = Self { focus_scope, look, panels };
        app.sync_window_background(window, cx);
        cx.observe_window_activation(window, Self::sync_window_background).detach();
        app
    }

    fn sync_window_background(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut background: Hsla = rgb(0x222222).into();
        let appearance = if window.is_window_active() {
            background.a = 0.9;
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };
        window.set_background_appearance(appearance);
        #[cfg(target_os = "macos")]
        if window.is_window_active()
            && let Err(error) = crate::backdrop::configure_sidebar_blur(window)
        {
            eprintln!("failed to configure sidebar blur: {error:#}");
        }
        self.panels.update(cx, |panels, cx| {
            panels.set_panel_background(0, Some(background), cx);
        });
    }
}

impl Render for SplitTitlebarShellApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let sans_family = self.look.mode_tokens().typography.font.sans.family.clone();

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .font_family(sans_family)
            .child(self.panels.clone())
    }
}
