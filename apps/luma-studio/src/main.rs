#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![windows_subsystem = "windows"]

mod app_shell;
#[path = "assets/assets.rs"]
mod assets;
mod studio;
mod theme;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};
use theme::LumaStudioLaunchOptions;

actions!(luma_studio_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let launch_options = LumaStudioLaunchOptions::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Luma Studio").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| {
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            luma_app_common::register_all(cx)?;
            app_shell::open(cx, launch_options.clone())
        }) {
            eprintln!("failed to open Luma Studio: {error:?}");
        }
    });
}
