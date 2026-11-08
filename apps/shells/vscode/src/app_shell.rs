use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use xgpui_luma::shell::TitleBar;
use luma_shell_common::ShellThemeChoice;

use crate::app::VscodeShellApp;

pub fn open(cx: &mut App, theme_choice: ShellThemeChoice) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1350.0), px(900.0)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            app_owns_titlebar_drag: true,
            #[cfg(target_os = "macos")]
            window_background: gpui::WindowBackgroundAppearance::Blurred,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| VscodeShellApp::new(window, cx, theme_choice)),
    )?;

    cx.activate(true);
    Ok(())
}
