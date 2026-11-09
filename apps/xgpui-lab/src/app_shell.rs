use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use xgpui_shell::{ShellApp, ShellOptions, ShellWindowTitle};

use crate::app::{XgpuiLabApp, LabTitlebar};

pub fn open(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
    cx.open_shell_window_with_titlebar(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            #[cfg(target_os = "macos")]
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            window_min_size: Some(size(px(720.0), px(480.0))),
            ..Default::default()
        },
        ShellOptions::default(),
        |window, cx| {
            window.set_shell_title("XGPUI Lab");
            let content = cx.new(|cx| XgpuiLabApp::new(window, cx));
            let titlebar = cx.new(|cx| LabTitlebar::new(&content, cx));
            (titlebar, content)
        },
    )?;
    cx.activate(true);
    Ok(())
}
