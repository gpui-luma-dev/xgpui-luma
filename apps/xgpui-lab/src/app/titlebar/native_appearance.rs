//! Per-window native appearance; app theme remains independent of the system theme.
use anyhow::Context as _;
use gpui::Window;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSView};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use xgpui_luma::theme::ThemeMode;

pub(super) fn sync(window: &Window, mode: ThemeMode) -> anyhow::Result<()> {
    let _main_thread = MainThreadMarker::new().context("native appearance requires the main thread")?;
    let handle = HasWindowHandle::window_handle(window)?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return Ok(());
    };
    // SAFETY: the borrowed raw handle guarantees a live NSView for this window.
    // This function runs on the main thread and never retains the borrowed pointer.
    let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
    let native_window = view.window().context("Lab view is not attached to a native window")?;
    // SAFETY: AppKit's exported appearance-name constants are immutable NSStrings.
    let name = unsafe {
        match mode {
            ThemeMode::Light => NSAppearanceNameAqua,
            ThemeMode::Dark => NSAppearanceNameDarkAqua,
        }
    };
    let appearance = NSAppearance::appearanceNamed(name).context("native appearance is unavailable")?;
    native_window.setAppearance(Some(&appearance));
    Ok(())
}
