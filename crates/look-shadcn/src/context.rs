use std::cell::RefCell;

use xgpui_luma_color::style::{ColorControlTheme, active_color_control_theme, set_active_color_control_theme};
use xgpui_luma::theme::ThemeMode;

use crate::look::ShadcnLook;

thread_local! {
    static ACTIVE_LOOK: RefCell<Option<ShadcnLook>> = const { RefCell::new(None) };
}

/// Syncs SDK color-control chrome (field/slider/ring borders) from the active shadcn look.
pub fn sync_color_control_theme(look: &ShadcnLook) {
    let chrome = crate::tables::resolve_color_chrome(look);

    set_active_color_control_theme(ColorControlTheme::new(
        chrome.border.hsla(),
        chrome.background.hsla(),
        matches!(look.mode(), ThemeMode::Dark),
    ));
}

/// Binds the active look to the thread's scope during layout/render block execution.
pub fn with_look<R>(look: &ShadcnLook, f: impl FnOnce() -> R) -> R {
    let previous_color_theme = active_color_control_theme();

    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = Some(look.clone());
    });
    sync_color_control_theme(look);

    let result = f();

    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = None;
    });
    set_active_color_control_theme(previous_color_theme);

    result
}

pub(crate) fn with_active_look<R>(f: impl FnOnce(Option<ShadcnLook>) -> R) -> R {
    ACTIVE_LOOK.with(|cell| f(cell.borrow().clone()))
}
