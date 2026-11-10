//! Data colors of the canvas scene and the Mandelbrot backdrop. The scene is the
//! subject of these screens, so it keeps its own dark palette in both themes.
//! Chrome (page, overlays, text) comes from the look.

/// World background under the grid; also the map backdrop.
pub(super) const STAGE: u32 = 0x101824;
pub(super) const CARD: u32 = 0x24354b;
pub(super) const CARD_SELECTED: u32 = 0x316eab;
pub(super) const CARD_EDGE: u32 = 0x6484a8;
pub(super) const CARD_TEXT: u32 = 0xffffff;
pub(super) const PROXY: u32 = 0x6484a8;
pub(super) const PROXY_SELECTED: u32 = 0x59b0ff;
/// Marquee outline; its fill is the same color at low alpha.
pub(super) const MARQUEE: u32 = 0x59b0ff;
pub(super) const MARQUEE_FILL: u32 = 0x59b0ff22;
pub(super) const SNAP_GUIDE: u32 = 0xff79db;
/// The visible-area rectangle on the whole-scene map.
pub(super) const MAP_VIEW: u32 = 0xffd479;
pub(super) const MAP_VIEW_FILL: u32 = 0xffd47922;
/// Painted under the fractal frames before the first one is ready.
pub(super) const FRACTAL_BACKDROP: u32 = 0x050810;
