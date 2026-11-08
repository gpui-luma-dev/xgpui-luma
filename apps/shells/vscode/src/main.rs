#![windows_subsystem = "windows"]

mod app;
mod app_shell;
mod customize_layout_dialog;
mod layout_config;
mod shell_state;
mod workspace_layout;
mod workbench_layout;
#[path = "assets/assets.rs"]
mod assets;

use assets::Assets;
use app::{TogglePanel, TogglePrimarySideBar};
use gpui::{actions, App, KeyBinding, Menu, MenuItem};
use luma_shell_common::{fonts, ShellThemeChoice};

use crate::layout_config::LayoutRegion;

actions!(shell_vscode_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let theme_choice = ShellThemeChoice::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        let layout_bindings = [
            KeyBinding::new(
                LayoutRegion::PrimarySideBar.shortcut_keystroke().unwrap_or("ctrl-b"),
                TogglePrimarySideBar,
                None,
            ),
            KeyBinding::new(LayoutRegion::Panel.shortcut_keystroke().unwrap_or("ctrl-j"), TogglePanel, None),
        ];
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.bind_keys(layout_bindings);
        cx.set_menus([Menu::new("Shell: VS Code").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = xgpui_luma::init(cx).and_then(|_| {
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            fonts::register_all(cx)?;
            app_shell::open(cx, theme_choice)
        }) {
            eprintln!("failed to open Shell: VS Code: {error:?}");
        }
    });
}
