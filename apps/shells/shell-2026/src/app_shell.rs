use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use xgpui_luma::shell::{TITLE_BAR_HEIGHT, TitleBar};
use luma_shell_common::ShellThemeChoice;

use crate::app::{SHELL_TITLEBAR_HEIGHT, Shell2026App};

pub fn open(cx: &mut App, theme_choice: ShellThemeChoice) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1350.0), px(900.0)), cx);
    let mut titlebar = TitleBar::title_bar_options();
    if let Some(position) = &mut titlebar.traffic_light_position {
        position.y += (SHELL_TITLEBAR_HEIGHT - TITLE_BAR_HEIGHT) / 2.0;
    }

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(720.0), px(480.0))),
            titlebar: Some(titlebar),
            app_owns_titlebar_drag: true,
            #[cfg(target_os = "macos")]
            window_background: gpui::WindowBackgroundAppearance::Blurred,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| Shell2026App::new(window, cx, theme_choice)),
    )?;

    cx.activate(true);
    Ok(())
}
