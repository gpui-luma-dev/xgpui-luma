use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use xgpui_luma::shell::{TITLE_BAR_HEIGHT, TitleBar};

use crate::app::PlotVizApp;

pub(crate) const SHELL_TITLEBAR_HEIGHT: gpui::Pixels = px(40.0);

pub(crate) fn open(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
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
        |_, cx| cx.new(PlotVizApp::new),
    )?;
    cx.activate(true);
    Ok(())
}
