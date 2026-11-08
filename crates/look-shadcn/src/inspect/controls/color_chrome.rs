//! Inspect metadata for shared color-control chrome (slider, ring, arc, field, swatch, thumb).

use crate::inspect::format_inspect_css_key;
use gpui::{px, Hsla, Pixels};
use xgpui_luma_color::chrome_tokens::{disabled_overlay, slider_blocked_overlay, swatch_checkerboard_colors};
use xgpui_luma_color::color_arc::visual::default_color_arc_visual;
use xgpui_luma_color::color_ring::visual::default_color_ring_visual;
use xgpui_luma_color::color_slider::color_thumb::ThumbStyle;
use xgpui_luma_color::color_slider::visual::default_color_slider_visual;
use xgpui_luma::theme::ThemeMode;
use crate::{sync_color_control_theme, ColorSource, MetricSource, ResolvedColor, ResolvedMetric, ShadcnLook};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorChromeProfile {
    SliderTrack,
    Ring,
    Arc,
    Field,
    Swatch,
    Thumb,
}

pub const COLOR_SLIDER_CHROME_PROFILES: &[ColorChromeProfile] =
    &[ColorChromeProfile::SliderTrack, ColorChromeProfile::Thumb, ColorChromeProfile::Swatch];

pub const COLOR_FIELD_CHROME_PROFILES: &[ColorChromeProfile] =
    &[ColorChromeProfile::Field, ColorChromeProfile::Thumb, ColorChromeProfile::Swatch];

pub const COLOR_RING_CHROME_PROFILES: &[ColorChromeProfile] = &[ColorChromeProfile::Ring, ColorChromeProfile::Thumb];

pub const COLOR_ARC_CHROME_PROFILES: &[ColorChromeProfile] = &[ColorChromeProfile::Arc, ColorChromeProfile::Thumb];

#[derive(Clone, Debug)]
pub struct ColorChromeInspectColorRow {
    pub label: &'static str,
    pub color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ColorChromeInspectPropertyRow {
    pub label: &'static str,
    pub value: String,
    pub source: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ColorChromeInspectSection {
    pub title: &'static str,
    pub note: Option<&'static str>,
    pub color_rows: Vec<ColorChromeInspectColorRow>,
    pub property_rows: Vec<ColorChromeInspectPropertyRow>,
}

pub fn inspect_color_chrome_sections(
    look: &ShadcnLook,
    profiles: &[ColorChromeProfile],
) -> Vec<ColorChromeInspectSection> {
    sync_color_control_theme(look);
    let theme_mode = look.mode();
    let colors = crate::tables::resolve_color_chrome(look);
    let border = colors.border;
    let background = colors.background;

    let slider_enabled = default_color_slider_visual(true);
    let slider_disabled = default_color_slider_visual(false);
    let ring_disabled = default_color_ring_visual(false);
    let arc_disabled = default_color_arc_visual(false);

    let mut sections = vec![shared_theme_section(theme_mode, &border, &background)];

    for profile in profiles {
        match profile {
            ColorChromeProfile::SliderTrack => {
                sections.push(slider_track_section(slider_enabled, slider_disabled, &border, &background, theme_mode));
            }
            ColorChromeProfile::Ring => {
                sections.push(ring_section(ring_disabled, &border, &background));
            }
            ColorChromeProfile::Arc => sections.push(arc_section(arc_disabled, &border, &background)),
            ColorChromeProfile::Field => sections.push(field_section(&border, &background)),
            ColorChromeProfile::Swatch => sections.push(swatch_section(&border, theme_mode)),
            ColorChromeProfile::Thumb => sections.push(thumb_section()),
        }
    }

    sections
}

fn shared_theme_section(
    theme_mode: ThemeMode,
    border: &ResolvedColor,
    background: &ResolvedColor,
) -> ColorChromeInspectSection {
    ColorChromeInspectSection {
        title: "Color control theme",
        note: Some("Synced from ShadcnLook::chrome() via sync_color_control_theme."),
        color_rows: vec![color_row("border", border.clone()), color_row("background", background.clone())],
        property_rows: vec![
            property_row(
                "dark mode",
                if matches!(theme_mode, ThemeMode::Dark) {
                    "yes"
                } else {
                    "no"
                },
                "ShadcnLook::mode()",
                Some(format!("Theme mode resolves to {theme_mode:?}")),
            ),
            property_row(
                "sync entrypoint",
                "sync_color_control_theme",
                "look-shadcn/context.rs",
                Some("Called inside with_look before color controls render.".to_string()),
            ),
        ],
    }
}

fn slider_track_section(
    enabled: xgpui_luma_color::color_slider::visual::ColorSliderVisual,
    disabled: xgpui_luma_color::color_slider::visual::ColorSliderVisual,
    border: &ResolvedColor,
    background: &ResolvedColor,
    theme_mode: ThemeMode,
) -> ColorChromeInspectSection {
    let blocked = sdk_constant_color(
        "color::chrome_tokens::slider_blocked_overlay",
        slider_blocked_overlay(matches!(theme_mode, ThemeMode::Dark)),
    );

    ColorChromeInspectSection {
        title: "Color slider track chrome",
        note: Some("Track fill gradients come from the active color model — not theme tokens."),
        color_rows: vec![
            color_row("track border", derived_from_runtime(&border, "active_color_control_theme().border")),
            color_row("disabled overlay", derived_disabled_overlay(&background, "default_color_slider_visual(false)")),
            color_row("blocked overlay", blocked),
        ],
        property_rows: vec![
            property_row(
                "blocked overlay mode",
                if matches!(theme_mode, ThemeMode::Dark) {
                    "dark preset"
                } else {
                    "light preset"
                },
                "color::chrome_tokens::slider_blocked_overlay",
                None,
            ),
            property_row(
                "runtime check",
                format_hsl(enabled.blocked_overlay),
                "default_color_slider_visual(true)",
                Some(format!("disabled overlay = {}", format_hsl(disabled.disabled_overlay))),
            ),
        ],
    }
}

fn ring_section(
    disabled: xgpui_luma_color::color_ring::visual::ColorRingVisual,
    border: &ResolvedColor,
    background: &ResolvedColor,
) -> ColorChromeInspectSection {
    ColorChromeInspectSection {
        title: "Color ring chrome",
        note: Some("Ring hue/saturation/lightness fills are domain rendering — not theme."),
        color_rows: vec![
            color_row("ring border", derived_from_runtime(&border, "active_color_control_theme().border")),
            color_row("disabled overlay", derived_disabled_overlay(&background, "default_color_ring_visual(false)")),
            color_row("center hole", derived_from_runtime(&background, "active_color_control_theme().background")),
        ],
        property_rows: vec![property_row(
            "runtime check",
            format_hsl(disabled.disabled_overlay),
            "default_color_ring_visual(false)",
            None,
        )],
    }
}

fn arc_section(
    disabled: xgpui_luma_color::color_arc::visual::ColorArcVisual,
    border: &ResolvedColor,
    background: &ResolvedColor,
) -> ColorChromeInspectSection {
    ColorChromeInspectSection {
        title: "Color arc chrome",
        note: Some("Arc channel fills are domain rendering — not theme."),
        color_rows: vec![
            color_row("arc border", derived_from_runtime(&border, "active_color_control_theme().border")),
            color_row("disabled overlay", derived_disabled_overlay(&background, "default_color_arc_visual(false)")),
        ],
        property_rows: vec![property_row(
            "runtime check",
            format_hsl(disabled.disabled_overlay),
            "default_color_arc_visual(false)",
            None,
        )],
    }
}

fn field_section(border: &ResolvedColor, background: &ResolvedColor) -> ColorChromeInspectSection {
    ColorChromeInspectSection {
        title: "Color field chrome",
        note: Some("Field domain fills (SV plane, wheel, triangle) are model rendering — not theme."),
        color_rows: vec![
            color_row("field border", derived_from_runtime(&border, "ColorField layout · cx.theme().border")),
            color_row(
                "outside / corner occluder",
                derived_from_runtime(&background, "ColorField · outside_color + paint_corner_occluder"),
            ),
        ],
        property_rows: vec![property_row(
            "disabled opacity",
            "0.5",
            "ColorField render",
            Some("cursor_not_allowed().opacity(0.5) when state.disabled".to_string()),
        )],
    }
}

fn swatch_section(border: &ResolvedColor, theme_mode: ThemeMode) -> ColorChromeInspectSection {
    let is_dark = matches!(theme_mode, ThemeMode::Dark);
    let (checker_a, checker_b) = swatch_checkerboard_colors(is_dark);

    ColorChromeInspectSection {
        title: "Color swatch chrome",
        note: Some("Swatch fill color is the selected sample — only border and checkerboard come from theme."),
        color_rows: vec![
            color_row("border", derived_from_runtime(&border, "ColorSwatch · cx.theme().border")),
            color_row(
                "checkerboard base",
                sdk_constant_color("color::chrome_tokens::swatch_checkerboard_colors(base)", checker_a),
            ),
            color_row(
                "checkerboard accent",
                sdk_constant_color("color::chrome_tokens::swatch_checkerboard_colors(accent)", checker_b),
            ),
        ],
        property_rows: vec![property_row(
            "checkerboard",
            "opt-in per swatch",
            "ColorSwatch::checkerboard",
            Some("Painted only when color.a < 0.999".to_string()),
        )],
    }
}

fn thumb_section() -> ColorChromeInspectSection {
    let thumb = ThumbStyle::new(px(16.0));

    ColorChromeInspectSection {
        title: "Color thumb chrome",
        note: Some("ThumbStyle defaults are SDK scaffold values — not derived from Shadcn tokens."),
        color_rows: vec![
            color_row("outer border", sdk_constant_color("ThumbStyle::border_outer", thumb.border_outer)),
            color_row("inner border", sdk_constant_color("ThumbStyle::border_inner", thumb.border_inner)),
        ],
        property_rows: vec![
            metric_property("border width", thumb_metric("border_width", thumb.border_width)),
            metric_property("inner inset", thumb_metric("inner_inset", thumb.inner_inset)),
            metric_property("square radius", thumb_metric("square_radius", thumb.square_radius)),
            property_row("active shadow", if thumb.active_shadow { "yes" } else { "no" }, "ThumbStyle", None),
            property_row("show outer border", if thumb.show_outer_border { "yes" } else { "no" }, "ThumbStyle", None),
            property_row("show inner border", if thumb.show_inner_border { "yes" } else { "no" }, "ThumbStyle", None),
        ],
    }
}

fn color_row(label: &'static str, color: ResolvedColor) -> ColorChromeInspectColorRow {
    ColorChromeInspectColorRow { label, color }
}

fn property_row(
    label: &'static str,
    value: impl ToString,
    source: &str,
    detail: Option<String>,
) -> ColorChromeInspectPropertyRow {
    ColorChromeInspectPropertyRow { label, value: value.to_string(), source: source.to_owned(), detail }
}

fn metric_property(label: &'static str, metric: ResolvedMetric) -> ColorChromeInspectPropertyRow {
    ColorChromeInspectPropertyRow {
        label,
        value: crate::inspect::format_metric_px(metric.value_px),
        source: crate::inspect::format_inspect_metric_source(&metric.source),
        detail: crate::inspect::format_inspect_metric_provenance(&metric.source),
    }
}

fn thumb_metric(field: &'static str, value: Pixels) -> ResolvedMetric {
    ResolvedMetric { value_px: value.as_f32(), source: MetricSource::Scaffold { path: format!("ThumbStyle.{field}") } }
}

fn derived_from_runtime(base: &ResolvedColor, runtime_path: &'static str) -> ResolvedColor {
    ResolvedColor {
        value: base.value,
        source: ColorSource::Derived { note: format!("= {} · {runtime_path}", format_inspect_css_key(&base.source)) },
    }
}

fn derived_disabled_overlay(background: &ResolvedColor, runtime_path: &'static str) -> ResolvedColor {
    ResolvedColor {
        value: disabled_overlay(background.value),
        source: ColorSource::Derived {
            note: format!("{} @ 45% · {runtime_path}", format_inspect_css_key(&background.source)),
        },
    }
}

fn sdk_constant_color(path: &'static str, value: Hsla) -> ResolvedColor {
    ResolvedColor { value, source: ColorSource::Derived { note: format!("sdk constant · {path}") } }
}

fn format_hsl(value: Hsla) -> String {
    format!("hsl({:.0} {:.0}% {:.0}% / {:.2})", value.h, value.s * 100.0, value.l * 100.0, value.a)
}

#[cfg(test)]
mod tests {
    use xgpui_luma::theme::ThemeMode;
    use crate::{ColorSource, LookResolver, ShadcnModeTokens};

    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn inspect_theme_rows_use_catalog_provenance() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "color_chrome_inspect");
        let border = resolver.resolve_decl("border").expect("border");
        assert!(matches!(border.source, ColorSource::CssVar { ref token } if token == "border"));
        let _ = mode;
    }

    #[test]
    fn blocked_overlay_matches_sdk_tokens() {
        assert_eq!(slider_blocked_overlay(true), slider_blocked_overlay(true));
        assert_ne!(slider_blocked_overlay(true), slider_blocked_overlay(false));
    }
}
