use gpui::{BoxShadow, Hsla};
use xgpui_luma::style::lss::{AttrValue, Color, ControlId, PartId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::ThemeMode;

use super::*;

/// Catalogs the parity tests run over: the built-in one, two tweakcn themes, and one without
/// `--radius`, `--spacing`, `--font-sans` or `--shadow-xs` (the fallbacks).
fn catalogs() -> Vec<(&'static str, String)> {
    let stripped = crate::FALLBACK_CSS
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !["--radius:", "--spacing:", "--font-sans:", "--shadow-xs:"].iter().any(|p| line.starts_with(p))
        })
        .collect::<Vec<_>>()
        .join("\n");
    vec![
        ("fallback", crate::FALLBACK_CSS.to_string()),
        ("astrovista", include_str!("../../tests/fixtures/tweakcn/astrovista.css").to_string()),
        ("retro-arcade", include_str!("../../tests/fixtures/tweakcn/retro-arcade.css").to_string()),
        ("stripped", stripped),
    ]
}

fn close(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
}

#[test]
fn sources_list_every_lss_file() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/style");
    for entry in std::fs::read_dir(&dir).expect("readable").flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".lss") {
            assert!(SOURCE_PATHS.contains(&name.as_str()), "{name} exists but is not listed in style::sources()");
        }
    }
}

#[test]
fn every_axis_combination_resolves_without_problems() {
    for (name, css) in catalogs() {
        let look = ShadcnLook::from_css_str(&css).expect("catalog");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let theme = theme(&look).unwrap_or_else(|d| panic!("{name}: binds:\n{d}"));
            for control in theme.stylesheet().controls() {
                let problems = theme.check(control);
                assert!(problems.is_empty(), "{name} {mode:?} {control}: {problems}");
            }
        }
    }
}

/// A resolved Shadcn checkbox: `attrs` as (attribute, value), `states` by name.
fn checkbox(
    look: &ShadcnLook,
    attrs: &[(&str, &str)],
    states: &[&str],
) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    let config = xgpui_luma::style::config();
    let key = attrs.iter().fold(StyleKey::new(ControlId::new("checkbox")), |key, (attr, value)| {
        key.attr(config.attribute(attr).expect("attribute"), AttrValue::named(value))
    });
    let key = states.iter().fold(key, |key, state| key.state(config.state(state).expect("state")));
    look.style_theme().resolve(&key)
}

fn color(resolved: &xgpui_luma::style::lss::Resolved, part: &'static str, what: &str) -> Hsla {
    let part = resolved.part(PartId::new(part));
    let color = match what {
        "background" => part.background(),
        "border" => part.border(),
        "focus" => part.focus_border(),
        _ => part.foreground(),
    };
    color.map(Hsla::from).unwrap_or_else(|| panic!("{what} is unset"))
}

fn shadow(resolved: &xgpui_luma::style::lss::Resolved) -> Vec<BoxShadow> {
    resolved.part(PartId::new("indicator")).shadow().map(|s| box_shadows(&s)).unwrap_or_default()
}

fn retro_arcade() -> ShadcnLook {
    ShadcnLook::from_css_str(include_str!("../../tests/fixtures/tweakcn/retro-arcade.css")).expect("catalog")
}

// Behaviour of the native Shadcn checkbox, kept as checks on the `.lss` checkbox.

#[test]
fn the_indicator_has_the_catalog_shadow_except_when_disabled() {
    let look = retro_arcade();
    assert_eq!(shadow(&checkbox(&look, &[], &[])).len(), 1, "shadow-xs");
    assert!(shadow(&checkbox(&look, &[], &["disabled"])).is_empty());
}

#[test]
fn hover_and_press_do_not_recolor_the_indicator_glyph_or_label() {
    for (name, css) in catalogs() {
        let look = ShadcnLook::from_css_str(&css).expect("catalog");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for checked in [&[][..], &["checked"]] {
                let rest = checkbox(&look, &[], checked);
                for extra in [&["hover"][..], &["pressed"], &["hover", "pressed"]] {
                    let moved = checkbox(&look, &[], &[checked, extra].concat());
                    for (part, what) in
                        [("indicator", "background"), ("indicator", "border"), ("glyph", "fg"), ("label", "fg")]
                    {
                        assert!(
                            close(color(&moved, part, what), color(&rest, part, what)),
                            "{name} {mode:?} {checked:?} {extra:?}: {part} {what}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn content_only_keeps_the_primary_indicator_without_shadow_or_ring() {
    let look = retro_arcade();
    let primary = checkbox(&look, &[], &["checked"]);
    let content_only = checkbox(&look, &[("variant", "content_only")], &["checked", "focused"]);
    for (part, what) in [("indicator", "background"), ("indicator", "border"), ("glyph", "fg")] {
        assert!(close(color(&content_only, part, what), color(&primary, part, what)), "{part} {what}");
    }
    assert!(shadow(&content_only).is_empty());
    assert_eq!(color(&content_only, "root", "focus").a, 0.0);
}

#[test]
fn checked_indicators_fill_with_the_style_color() {
    let look = retro_arcade();
    let tokens = look.mode_tokens();
    let css = |name: &str| crate::resolve::resolve_color(&tokens.catalog, name).expect("catalog color");
    for (variant, fill) in [(None, "primary"), (Some("secondary"), "secondary"), (Some("outline"), "foreground")] {
        let attrs: Vec<(&str, &str)> = variant.map(|v| ("variant", v)).into_iter().collect();
        let checked = checkbox(&look, &attrs, &["checked"]);
        assert!(close(color(&checked, "indicator", "background"), css(fill)), "{variant:?}");
        assert!(close(color(&checked, "indicator", "border"), css(fill)), "{variant:?} border follows the fill");
    }
}

/// Looks require `--ring` and `--border`, but mode tokens edited without them still draw: the focus ring
/// and the indicator border fall back to the incomplete-palette foreground.
#[test]
fn missing_ring_and_border_fall_back() {
    let look = retro_arcade();
    let mut tokens = (*look.mode_tokens()).clone();
    tokens.catalog.tokens.remove("ring");
    tokens.catalog.tokens.remove("border");
    let tokens = std::sync::Arc::new(tokens);
    let source = ShadcnTokens { light: tokens.clone(), dark: tokens };
    let theme = Theme::bind(&stylesheet(), std::sync::Arc::new(source), "light").expect("binds");
    let config = xgpui_luma::style::config();
    let resolved =
        theme.resolve(&StyleKey::new(ControlId::new("checkbox")).state(config.state("focused").expect("state")));
    let fallback = Hsla::from(Color::BLACK);
    assert!(close(color(&resolved, "indicator", "border"), fallback));
    assert!(close(color(&resolved, "root", "focus"), fallback));
}

#[test]
fn sizes_scale_the_indicator() {
    let look = retro_arcade();
    let size = |attrs: &[(&str, &str)]| {
        checkbox(&look, attrs, &[]).root().value("indicator_size").and_then(Value::as_number).expect("size")
    };
    assert_eq!(size(&[("size", "sm")]), 13.0, "28 × 0.45, snapped");
    assert_eq!(size(&[]), 18.0, "36 × 0.5");
    assert_eq!(size(&[("size", "lg")]), 24.0, "44 × 0.55, snapped");
}

#[test]
fn style_theme_follows_the_mode() {
    let label = |look: &ShadcnLook| {
        look.style_theme()
            .resolve(&StyleKey::new(ControlId::new("checkbox")))
            .part(PartId::new("label"))
            .foreground()
            .map(Hsla::from)
            .expect("label color")
    };
    let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("catalog");
    let light = label(&look);
    look.set_mode(ThemeMode::Dark);
    let dark = label(&look);
    assert!(!close(light, dark), "the cached theme follows the mode");
    look.set_mode(ThemeMode::Light);
    assert!(close(label(&look), light));
}
