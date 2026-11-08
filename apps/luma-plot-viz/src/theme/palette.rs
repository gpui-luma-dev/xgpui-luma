//! Chart-owned colors. SDK UI tokens supply chrome, never series colors.
use gpui::Hsla;
use xgpui_luma::theme::ThemeMode;

// Accept degrees and percentages so authored values match stylesheet HSLA.
const fn color(hue: f32, saturation: f32, lightness: f32) -> Hsla {
    Hsla { h: hue / 360.0, s: saturation / 100.0, l: lightness / 100.0, a: 1.0 }
}

pub(crate) const LABEL_BLACK: Hsla = color(0.0, 0.0, 0.0);
pub(crate) const LABEL_WHITE: Hsla = color(0.0, 0.0, 100.0);
pub(crate) const BLUE_PALE: Hsla = color(211.70, 96.36, 78.43);
pub(crate) const BLUE_BRIGHT: Hsla = color(216.23, 100.00, 58.43);
pub(crate) const BLUE_MEDIUM: Hsla = color(221.30, 97.47, 53.53);
pub(crate) const BLUE_DEEP: Hsla = color(225.43, 84.00, 49.02);
pub(crate) const BLUE_DARKEST: Hsla = color(225.93, 70.73, 40.20);
pub(crate) const BLUE_LIGHT_MODE: Hsla = color(200.41, 98.01, 39.41);
pub(crate) const BLUE_LIGHT_SECONDARY: Hsla = color(221.21, 83.19, 53.33);
pub(crate) const BLUE_DARK_SECONDARY: Hsla = color(217.22, 91.22, 59.80);

const CATEGORICAL_LIGHT: [Hsla; 2] = [color(221.21, 83.19, 53.33), color(175.34, 77.44, 26.08)];
const CATEGORICAL_DARK: [Hsla; 2] = [color(213.12, 93.9, 67.84), color(172.46, 66.01, 50.39)];
const BLUE_LIGHT: [Hsla; 2] = [BLUE_LIGHT_MODE, BLUE_LIGHT_SECONDARY];
const BLUE_DARK: [Hsla; 2] = [BLUE_PALE, BLUE_DARK_SECONDARY];
/// Stable categorical scale for pie slices and radial segments.
pub(crate) const BLUE_SCALE: [Hsla; 5] = [BLUE_PALE, BLUE_BRIGHT, BLUE_MEDIUM, BLUE_DEEP, BLUE_DARKEST];

pub(crate) fn categorical(mode: ThemeMode) -> [Hsla; 2] {
    match mode {
        ThemeMode::Light => CATEGORICAL_LIGHT,
        ThemeMode::Dark => CATEGORICAL_DARK,
    }
}
pub(crate) fn blue(mode: ThemeMode) -> [Hsla; 2] {
    match mode {
        ThemeMode::Light => BLUE_LIGHT,
        ThemeMode::Dark => BLUE_DARK,
    }
}
