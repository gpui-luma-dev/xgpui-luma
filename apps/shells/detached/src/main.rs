mod app;
mod app_shell;
#[path = "assets/assets.rs"]
mod assets;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};
use luma_shell_common::{ShellThemeChoice, fonts};

actions!(shell_detached_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let theme_choice = ShellThemeChoice::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Shell: Detached").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| {
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            fonts::register_all(cx)?;
            app_shell::open(cx, theme_choice)
        }) {
            eprintln!("failed to open Shell: Detached: {error:?}");
        }
    });
}
