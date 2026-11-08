use xgpui_luma::theme::{ControlSize, LumaTypography, MetricTokens};

use super::CssTokenMap;

struct ControlSpacingScale {
    padding_x: f32,
    padding_y: f32,
    gap: f32,
}

fn control_spacing_scale(size: ControlSize) -> ControlSpacingScale {
    match size {
        ControlSize::Sm => ControlSpacingScale { padding_x: 2.5, padding_y: 1.25, gap: 1.5 },
        ControlSize::Md => ControlSpacingScale { padding_x: 3.5, padding_y: 2.0, gap: 2.0 },
        ControlSize::Lg => ControlSpacingScale { padding_x: 4.5, padding_y: 2.5, gap: 2.5 },
    }
}

fn apply_spacing_to_control(
    metrics: &mut xgpui_luma::theme::ControlMetricTokens,
    spacing_px: f32,
    scale: ControlSpacingScale,
) {
    metrics.padding_x = spacing_px * scale.padding_x;
    metrics.padding_y = spacing_px * scale.padding_y;
    metrics.gap = spacing_px * scale.gap;
}

pub fn spacing_multiplier(size: ControlSize, field: SpacingField) -> f32 {
    let scale = control_spacing_scale(size);
    match field {
        SpacingField::PaddingX => scale.padding_x,
        SpacingField::PaddingY => scale.padding_y,
        SpacingField::Gap => scale.gap,
    }
}

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

pub(crate) fn metrics_from_catalog(catalog: &CssTokenMap, mut scaffold: MetricTokens) -> MetricTokens {
    if let Some(spacing_raw) = catalog.get("spacing") {
        if let Some(spacing_px) = parse_length_px(spacing_raw) {
            apply_spacing_to_control(&mut scaffold.control.sm, spacing_px, control_spacing_scale(ControlSize::Sm));
            apply_spacing_to_control(&mut scaffold.control.md, spacing_px, control_spacing_scale(ControlSize::Md));
            apply_spacing_to_control(&mut scaffold.control.lg, spacing_px, control_spacing_scale(ControlSize::Lg));
            scaffold.spacing.s1 = spacing_px;
        }
    }

    let Some(radius_raw) = catalog.get("radius") else {
        return scaffold;
    };
    let Some(base_px) = parse_length_px(radius_raw) else {
        return scaffold;
    };
    let scale = base_px / scaffold.radius.md;
    scaffold.radius.sm *= scale;
    scaffold.radius.md = base_px;
    scaffold.radius.lg *= scale;
    scaffold.radius.xl *= scale;
    // Apply the standard Shadcn/Tailwind offsets:
    // Sm gets rounded-sm (radius - 4px)
    // Md gets rounded-md (radius - 2px)
    // Lg gets rounded-lg (radius)
    let sm_radius = (base_px - 4.0).max(0.0);
    let md_radius = (base_px - 2.0).max(0.0);
    let lg_radius = base_px;
    scale_control_metrics(&mut scaffold.control.sm, scale, sm_radius);
    scale_control_metrics(&mut scaffold.control.md, scale, md_radius);
    scale_control_metrics(&mut scaffold.control.lg, scale, lg_radius);
    scaffold
}

fn scale_control_metrics(metrics: &mut xgpui_luma::theme::ControlMetricTokens, scale: f32, md_radius: f32) {
    metrics.radius = md_radius;
    let _ = scale;
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

fn first_font_family(raw: &str) -> String {
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
    fn spacing_token_scales_control_padding_and_gap() {
        use xgpui_luma::theme::ControlSize;

        let catalog = CssTokenMap::from_map(std::collections::BTreeMap::from([
            ("spacing".into(), "0.25rem".into()),
            ("radius".into(), "0.25rem".into()),
        ]));
        let metrics = metrics_from_catalog(&catalog, MetricTokens::default());

        assert_eq!(metrics.control.md.padding_x, 14.0);
        assert_eq!(metrics.control.md.padding_y, 8.0);
        assert_eq!(metrics.control.md.gap, 8.0);
        assert_eq!(metrics.control.sm.padding_x, 10.0);
        assert_eq!(metrics.control.lg.padding_x, 18.0);
        assert_eq!(spacing_multiplier(ControlSize::Md, SpacingField::PaddingX), 3.5);
    }
}
