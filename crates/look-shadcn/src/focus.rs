use super::resolve::resolve_color_or_fallback;
use super::catalog::CssTokenMap;

/// Paint-safe focus ring. Missing `--ring` uses the incomplete-palette foreground.
pub fn focus_ring_or_fallback(catalog: &CssTokenMap) -> gpui::Hsla {
    resolve_color_or_fallback(catalog, "ring")
}
