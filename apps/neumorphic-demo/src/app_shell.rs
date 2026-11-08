use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use xgpui_luma::shell::TitleBar;

use crate::app::NeumorphicDemoApp;

pub fn open(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            // No custom TitleBar is rendered; let the OS handle title bar dragging.
            app_owns_titlebar_drag: false,
            #[cfg(target_os = "macos")]
            window_background: gpui::WindowBackgroundAppearance::Blurred,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| NeumorphicDemoApp::new(window, cx)),
    )?;

    cx.activate(true);
    Ok(())
}
