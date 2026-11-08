use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};

use xgpui_luma::shell::TitleBar;

use crate::studio::LumaStudioApp;
use crate::theme::LumaStudioLaunchOptions;

const DEFAULT_WINDOW_WIDTH_PX: f32 = 1200.0;
const DEFAULT_WINDOW_HEIGHT_PX: f32 = 700.0;

pub fn open(cx: &mut App, launch_options: LumaStudioLaunchOptions) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(DEFAULT_WINDOW_WIDTH_PX), px(DEFAULT_WINDOW_HEIGHT_PX)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            app_owns_titlebar_drag: true,
            #[cfg(target_os = "macos")]
            window_background: gpui::WindowBackgroundAppearance::Blurred,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| LumaStudioApp::new(window, cx, launch_options)),
    )?;

    cx.activate(true);
    Ok(())
}
