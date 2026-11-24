#[cfg(test)]
use gpui::Hsla;

#[cfg(test)]
use super::catalog::CssTokenMap;

/// A catalog color, for tests that compare resolved styles with the CSS.
#[cfg(test)]
pub(crate) fn resolve_color(catalog: &CssTokenMap, token: &str) -> anyhow::Result<Hsla> {
    catalog.color(token)
}
