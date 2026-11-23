use xgpui_luma::style::lss::Theme;
use xgpui_luma::theme::{LumaTypography, MetricTokens};

use crate::ShadcnRadii;
use crate::size::{ShadcnControlMetrics, ShadcnSize};

use super::CssTokenMap;

#[derive(Clone, Copy)]
pub enum SpacingField {
    PaddingX,
    PaddingY,
    Gap,
}

const GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
];

/// Shared metrics (the `md` size) and Shadcn's per-size control metrics, read from the look's `.lss`
/// (`metrics.*`, with `size sm` / `size lg` scopes) bound to a mode's catalog. The `.lss` is their one
/// source: it derives padding, gap and radius from `--spacing` and `--radius`, with its own fallbacks.
pub(crate) fn metrics_from_theme(theme: &Theme) -> anyhow::Result<(MetricTokens, ShadcnControlMetrics)> {
    let read = |size: ShadcnSize| MetricTokens::from_theme(theme, &[("size", size.key())]);
    let (sm, md, lg) = (read(ShadcnSize::Sm)?, read(ShadcnSize::Md)?, read(ShadcnSize::Lg)?);
    Ok((md, ShadcnControlMetrics { sm: sm.control, md: md.control, lg: lg.control }))
}

/// Shadcn's named radii from `--radius`, or the defaults without one.
pub(crate) fn radii_from_catalog(catalog: &CssTokenMap) -> ShadcnRadii {
    catalog.get("radius").and_then(parse_length_px).map(ShadcnRadii::from_base).unwrap_or_default()
}

pub(crate) fn typography_from_catalog(catalog: &CssTokenMap, mut scaffold: LumaTypography) -> LumaTypography {
    if let Some(raw) = catalog.get("font-sans") {
        scaffold.font.sans.family = first_font_family(raw);
    }
    if let Some(raw) = catalog.get("font-mono") {
        scaffold.font.mono.family = first_font_family(raw);
    }
    if let Some(raw) = catalog.get("font-serif") {
        scaffold.font.serif.family = first_font_family(raw);
    }
    scaffold
}

pub(crate) fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

fn normalize_font_family(family: &str) -> String {
    // Fontshare variable TTF typographic family (name ID 16); tweakcn exports "Rajdhani".
    if family.eq_ignore_ascii_case("Rajdhani") {
        return "Rajdhani Variable".to_string();
    }
    family.to_string()
}

pub(crate) fn first_font_family(raw: &str) -> String {
    for part in raw.split(',') {
        let candidate = part.trim().trim_matches(['\'', '"']);
        if candidate.is_empty() {
            continue;
        }
        if GENERIC_FAMILIES.iter().any(|generic| candidate.eq_ignore_ascii_case(generic)) {
            continue;
        }
        return normalize_font_family(candidate);
    }

    normalize_font_family(raw.split(',').next().unwrap_or(raw).trim().trim_matches(['\'', '"']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rem_radius_to_pixels() {
        assert_eq!(parse_length_px("0.625rem"), Some(10.0));
    }

    #[test]
    fn picks_first_non_generic_font() {
        assert_eq!(first_font_family("ui-sans-serif, system-ui, 'Rajdhani', sans-serif"), "Rajdhani Variable");
    }

    #[test]
    fn maps_rajdhani_css_export_to_variable_family() {
        assert_eq!(first_font_family("Rajdhani, sans-serif"), "Rajdhani Variable");
    }

    #[test]
    fn radius_token_sets_the_named_radii() {
        let catalog = CssTokenMap::from_map(std::collections::BTreeMap::from([("radius".into(), "0.25rem".into())]));
        let radii = radii_from_catalog(&catalog);
        assert_eq!((radii.small, radii.medium, radii.large, radii.xl), (0.0, 2.0, 4.0, 8.0));
    }
}
