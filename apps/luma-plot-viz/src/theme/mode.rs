use xgpui_luma::theme::ThemeMode;

pub(crate) fn id(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

pub(crate) fn mode_for_id(id: &str) -> ThemeMode {
    if id == "light" {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    }
}
