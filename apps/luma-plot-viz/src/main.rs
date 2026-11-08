#![windows_subsystem = "windows"]

mod app;
mod app_shell;
mod assets;
mod theme;
mod examples;

use gpui::{App, KeyBinding, Menu, MenuItem, actions};

actions!(plot_viz_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    gpui_platform::application().with_assets(assets::Assets).run(|cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None), KeyBinding::new("ctrl-q", Quit, None)]);
        cx.set_menus([Menu::new("Plot Viz").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| {
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            app_shell::open(cx)
        }) {
            eprintln!("failed to open Plot Viz: {error:?}");
            cx.quit();
        }
    });
}
