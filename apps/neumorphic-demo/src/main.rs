mod app;
mod app_shell;
mod background;
mod components;
mod power_toggle_template;
mod radio_group_template;
mod slider_template;
mod switch_template;
#[path = "assets/assets.rs"]
mod assets;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};

actions!(neumorphic_demo_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(|cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Neumorphic Demo").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| app_shell::open(cx)) {
            eprintln!("failed to open neumorphic demo: {error:?}");
        }
    });
}
