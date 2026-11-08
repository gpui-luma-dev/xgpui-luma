use gpui::{Hsla, Pixels, px, rgb};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_shadcn::ShadcnLook;

pub(super) const RAIL_WIDTH: Pixels = px(52.0);
pub(crate) const SHELL_TITLEBAR_HEIGHT: Pixels = px(40.0);
pub(super) const CANVAS_BORDER: Pixels = px(1.0);
pub(super) const CANVAS_RADIUS: Pixels = px(10.0);
const DARK_SHELL_BACKGROUND: u32 = 0x272727;
const DARK_CANVAS_BORDER_COLOR: u32 = 0x363636;
pub(super) const INITIAL_SIDEBAR_WIDTH: Pixels = px(260.0);
pub(super) const MIN_SIDEBAR_WIDTH: Pixels = px(200.0);
pub(super) const MAX_SIDEBAR_WIDTH: Pixels = px(480.0);
pub(super) const MIN_WORKSPACE_WIDTH: Pixels = px(380.0);
pub(super) const CANVAS_TOP_INSET: Pixels = px(2.0);
pub(super) const CANVAS_EDGE_INSET: Pixels = px(3.0);

pub(super) fn shell_background(look: &ShadcnLook) -> Hsla {
    match look.mode() {
        ThemeMode::Dark => rgb(DARK_SHELL_BACKGROUND).into(),
        ThemeMode::Light => look.chrome().app_background,
    }
}

pub(super) fn canvas_border_color(look: &ShadcnLook) -> Hsla {
    match look.mode() {
        ThemeMode::Dark => rgb(DARK_CANVAS_BORDER_COLOR).into(),
        ThemeMode::Light => look.chrome().border,
    }
}
