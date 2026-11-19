use gpui::{BoxShadow, Hsla};
use xgpui_luma::style::lss::{AttrValue, Color, ControlId, PartId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::{InteractionState, ThemeMode};

use super::*;
use crate::ShadcnSize;
use crate::controls::checkbox::{checkbox_look, checkbox_scale_with_stylesheet};

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

fn same_shadows(got: &[BoxShadow], want: &[BoxShadow]) -> bool {
    got.len() == want.len()
        && got.iter().zip(want).all(|(g, w)| {
            close(g.color, w.color)
                && g.offset == w.offset
                && g.blur_radius == w.blur_radius
                && g.spread_radius == w.spread_radius
                && g.inset == w.inset
        })
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

#[test]
fn checkbox_matches_the_native_look() {
    let config = xgpui_luma::style::config();
    let state = |name: &str| config.state(name).expect("state");
    let (variant_attr, size_attr) =
        (config.attribute("variant").expect("variant"), config.attribute("size").expect("size"));
    let checkbox = ControlId::new("checkbox");
    let (indicator, glyph, label) = (PartId::new("indicator"), PartId::new("glyph"), PartId::new("label"));
    let variants = [
        (ShadcnButtonStyle::Primary, None),
        (ShadcnButtonStyle::Secondary, Some("secondary")),
        (ShadcnButtonStyle::Outline, Some("outline")),
        (ShadcnButtonStyle::Ghost, Some("ghost")),
        (ShadcnButtonStyle::ContentOnly, Some("content_only")),
    ];
    let sizes = [(ShadcnSize::Sm, Some("sm")), (ShadcnSize::Md, None), (ShadcnSize::Lg, Some("lg"))];

    for (catalog, css) in catalogs() {
        let look = ShadcnLook::from_css_str(&css).expect("catalog");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let theme = theme(&look).unwrap_or_else(|d| panic!("{catalog}: binds:\n{d}"));
            let sheet = theme.stylesheet();
            let length = |name: &str| sheet.prop::<f32>(checkbox, name).expect("declared");
            let tokens = look.mode_tokens();
            let stylesheet = look.stylesheet();
            for (style, variant) in variants {
                for (size, size_key) in sizes {
                    for bits in 0..32u8 {
                        let s = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            invalid: bits & 16 != 0,
                        };
                        for checked in [false, true] {
                            let palette = checkbox_look(&tokens, style, checked, s, size);
                            for scale in [1.0, 1.5, 2.0] {
                                let case =
                                    format!("{catalog} {mode:?} {style:?} {size:?} {s:?} checked={checked} @{scale}x");
                                let key = StyleKey::new(checkbox)
                                    .attr_opt(variant_attr, variant.map(AttrValue::new))
                                    .attr_opt(size_attr, size_key.map(AttrValue::new))
                                    .state_if(state("hover"), s.hovered)
                                    .state_if(state("pressed"), s.pressed)
                                    .state_if(state("focused"), s.focused)
                                    .state_if(state("disabled"), s.disabled)
                                    .state_if(state("invalid"), s.invalid)
                                    .state_if(state("checked"), checked)
                                    .scale(scale);
                                let resolved = theme.resolve(&key);

                                let color = |got: Option<Color>, want: Hsla, what: &str| {
                                    let got = got.map(Hsla::from).unwrap_or_else(|| panic!("{case}: {what} is unset"));
                                    assert!(close(got, want), "{case}: {what}: {got:?} != {want:?}");
                                };
                                color(resolved.part(indicator).background(), palette.indicator_background, "indicator");
                                color(resolved.part(indicator).border(), palette.indicator_border, "indicator border");
                                color(resolved.part(glyph).foreground(), palette.checkmark_color, "checkmark");
                                color(resolved.part(label).foreground(), palette.label_color, "label");
                                color(resolved.root().focus_border(), palette.focus_border, "focus border");
                                assert_eq!(resolved.root().background(), None, "{case}: control background");
                                assert_eq!(resolved.root().border(), None, "{case}: control border");
                                let shadows =
                                    resolved.part(indicator).shadow().map(|s| box_shadows(&s)).unwrap_or_default();
                                let want = palette.indicator_shadow.clone().unwrap_or_default();
                                assert!(same_shadows(&shadows, &want), "{case}: shadow {shadows:?} != {want:?}");

                                let ctx = crate::LookContext::new(&tokens, mode, InteractionState::default());
                                let (metrics, _) = checkbox_scale_with_stylesheet(&ctx, &stylesheet, size, scale);
                                let root = resolved.root();
                                let number = |name: &str| root.value(name).and_then(Value::as_number);
                                assert_eq!(
                                    root.get(length("indicator_size")),
                                    Some(metrics.indicator_size),
                                    "{case}: indicator_size"
                                );
                                assert_eq!(
                                    root.get(length("glyph_size")),
                                    Some(metrics.glyph_size),
                                    "{case}: glyph_size"
                                );
                                assert_eq!(
                                    root.get(length("indicator_radius")),
                                    Some(metrics.indicator_radius),
                                    "{case}: indicator_radius"
                                );
                                assert_eq!(
                                    root.get(length("label_baseline_shift")),
                                    Some(metrics.label_baseline_shift),
                                    "{case}: baseline"
                                );
                                assert_eq!(root.height(), Some(metrics.height), "{case}: height");
                                assert_eq!(root.gap(), Some(metrics.gap), "{case}: gap");
                                assert_eq!(root.radius(), Some(metrics.control_radius), "{case}: control radius");
                                assert_eq!(number("padding_x"), Some(metrics.control_padding_x), "{case}: padding x");
                                assert_eq!(number("padding_y"), Some(metrics.control_padding_y), "{case}: padding y");
                                assert_eq!(
                                    number("disabled_opacity"),
                                    Some(metrics.disabled_opacity),
                                    "{case}: disabled opacity"
                                );
                                assert_eq!(
                                    number("border_width"),
                                    Some(tokens.metrics.border_width),
                                    "{case}: border width"
                                );

                                let text = resolved.part(label);
                                let typography = palette.label_typography;
                                assert_eq!(text.font_size(), Some(typography.size), "{case}: font size");
                                let lh = text.line_height().expect("line height");
                                assert!(
                                    (lh - typography.line_height).abs() < 1e-3,
                                    "{case}: line height {lh} != {}",
                                    typography.line_height
                                );
                                assert_eq!(
                                    text.value("font_weight").and_then(Value::as_number),
                                    Some(typography.weight.0),
                                    "{case}: weight"
                                );
                                assert_eq!(
                                    text.font_family(),
                                    Some(palette.label_font_family.as_ref()),
                                    "{case}: font family"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
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
