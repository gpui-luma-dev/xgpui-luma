use super::*;
use super::gpui_bridge::{SrgbRenderCache, from_hsla, to_hsla, to_rgba};

#[test]
fn persistence_preserves_source_space_and_extended_components() {
    let colors = [
        ColorValue::display_p3(1.0, 0.0, 0.0, 0.37),
        ColorValue::oklch(0.72, 0.4, 725.0, 0.6),
        ColorValue::linear_srgb(-0.25, 2.0, 0.5, 1.0),
        ColorValue::srgb(1.2, -0.1, 0.4, 0.0),
    ];
    for source in colors {
        let text = toml::to_string(&source).unwrap();
        let restored: ColorValue = toml::from_str(&text).unwrap();
        assert_eq!(restored.components(), source.components());
        let before = source.components();
        let _preview = source.to_srgba_fallback(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(source.components(), before);
    }
    let encoded = toml::to_string(&colors[0]).unwrap();
    assert!(encoded.contains("space = \"display-p3\""));
}

#[test]
fn p3_red_is_retained_beyond_srgb() {
    let source = ColorValue::display_p3(1.0, 0.0, 0.0, 0.4);
    assert!(source.is_in_gamut(Gamut::DisplayP3).unwrap());
    assert!(!source.is_in_gamut(Gamut::Srgb).unwrap());
    let extended = source.to_srgba_unclamped().unwrap();
    assert!(extended.red > 1.0);
    assert!(extended.green < 0.0);
    let clipped = source.to_srgba_fallback(GamutMapping::Clip).unwrap();
    let mapped = source.to_srgba_fallback(GamutMapping::CssLocalMinde).unwrap();
    assert_eq!(mapped.alpha, 0.4);
    assert!(mapped.green > clipped.green);
    for v in [mapped.red, mapped.green, mapped.blue] {
        assert!((0.0..=1.0).contains(&v));
    }
    assert_eq!(source, ColorValue::display_p3(1.0, 0.0, 0.0, 0.4));
}

#[test]
fn in_gamut_srgb_is_unchanged_by_either_policy() {
    let source = ColorValue::srgb(0.2, 0.4, 0.7, 0.3);
    for policy in [GamutMapping::Clip, GamutMapping::CssLocalMinde] {
        let result = source.to_srgba_fallback(policy).unwrap();
        assert_eq!(result, Srgba::new(0.2, 0.4, 0.7, 0.3));
        let gpui = to_rgba(source, policy).unwrap();
        assert_eq!([gpui.r, gpui.g, gpui.b, gpui.a], [0.2, 0.4, 0.7, 0.3]);
    }
}

#[test]
fn linear_rgb_is_encoded_only_at_the_bridge() {
    let source = ColorValue::linear_srgb(0.5, 0.5, 0.5, 1.0);
    let preview = to_rgba(source, GamutMapping::Clip).unwrap();
    assert!((preview.r - 0.735357).abs() < 0.00001);
    assert_eq!(source.components().1, [0.5, 0.5, 0.5, 1.0]);
}

#[test]
fn hue_units_and_achromatic_source_hue_survive() {
    let source = ColorValue::oklch(0.6, 0.1, 30.0, 1.0);
    let wrapped = ColorValue::oklch(0.6, 0.1, 390.0, 1.0);
    let a = source.to_srgba_unclamped().unwrap();
    let b = wrapped.to_srgba_unclamped().unwrap();
    assert!((a.red - b.red).abs() < 0.00001);
    assert!((a.green - b.green).abs() < 0.00001);
    let gray = ColorValue::oklch(0.6, 0.0, 275.0, 1.0);
    let text = toml::to_string(&gray).unwrap();
    let restored: ColorValue = toml::from_str(&text).unwrap();
    assert_eq!(restored.components().1[2], 275.0);
    let blue = from_hsla(gpui::hsla(2.0 / 3.0, 1.0, 0.5, 0.5));
    let backend = to_hsla(blue, GamutMapping::Clip).unwrap();
    assert!((backend.h - 2.0 / 3.0).abs() < 0.00001);
    assert_eq!(backend.a, 0.5);
}

#[test]
fn mapping_lightness_extremes_preserves_alpha() {
    for (lightness, expected) in [(-0.1, 0.0), (1.1, 1.0)] {
        let source = ColorValue::oklch(lightness, 0.3, 90.0, 0.2);
        let result = source.to_srgba_fallback(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(result, Srgba::new(expected, expected, expected, 0.2));
    }
}

#[test]
fn css_local_minde_matches_independent_f64_reference_samples() {
    // Evaluated independently using the published Oklab matrices and CSS §14.2.2
    // pseudocode in f64. A tolerance accommodates Palette's f32 conversion matrices.
    let samples = [
        ([0.7, 0.3, 30.0], [1.0, 0.34513507, 0.2645751]),
        ([0.6, 0.3, 145.0], [0.0, 0.6236596, 0.0]),
        ([0.8, 0.25, 280.0], [0.6822987, 0.7073379, 1.0]),
    ];
    for ([l, c, h], expected) in samples {
        let result = ColorValue::oklch(l, c, h, 0.7).to_srgba_fallback(GamutMapping::CssLocalMinde).unwrap();
        for (actual, expected) in [result.red, result.green, result.blue].into_iter().zip(expected) {
            assert!((actual - expected).abs() < 0.001, "{l}/{c}/{h}: {actual} != {expected}");
        }
        assert_eq!(result.alpha, 0.7);
    }
}

#[test]
fn invalid_values_are_rejected_instead_of_clipped_or_cached() {
    for source in [
        ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0),
        ColorValue::display_p3(f32::INFINITY, 0.0, 0.0, 1.0),
        ColorValue::srgb(0.0, 0.0, 0.0, 1.1),
        ColorValue::oklch(0.5, -0.1, 90.0, 1.0),
    ] {
        let mut cache = SrgbRenderCache::default();
        assert!(source.validate().is_err());
        assert!(toml::to_string(&source).is_err());
        assert!(cache.resolve(source, GamutMapping::CssLocalMinde).is_err());
        assert_eq!(cache.len(), 0);
    }
    assert!(toml::from_str::<ColorValue>("space = 'srgb'\ncomponents = [0.0, 0.0, 0.0, 2.0]").is_err());
}

#[test]
fn cache_reuses_source_and_separates_space_alpha_and_policy() {
    let mut cache = SrgbRenderCache::default();
    let source = ColorValue::display_p3(1.0, 0.0, 0.0, 1.0);
    let first = cache.resolve(source, GamutMapping::CssLocalMinde).unwrap();
    assert_eq!(cache.resolve(source, GamutMapping::CssLocalMinde).unwrap(), first);
    assert_eq!(cache.len(), 1);
    let clipped = cache.resolve(source, GamutMapping::Clip).unwrap();
    assert_ne!(clipped, first);
    cache.resolve(source.with_alpha(0.5).unwrap(), GamutMapping::Clip).unwrap();
    cache.resolve(ColorValue::srgb(1.0, 0.0, 0.0, 1.0), GamutMapping::Clip).unwrap();
    assert_eq!(cache.len(), 4);
    cache.clear();
    assert_eq!(cache.len(), 0);
    for i in 0..300 {
        cache.resolve(ColorValue::srgb(i as f32 / 300.0, 0.0, 0.0, 1.0), GamutMapping::Clip).unwrap();
    }
    assert!(cache.len() <= 256);
}

#[test]
fn editor_hsl_bridge_retains_achromatic_hue_and_turn_endpoint() {
    for hue in [0.0, 30.0, 360.0] {
        for lightness in [0.0, 0.5, 1.0] {
            let color = palette::Hsla::new(hue, 0.0, lightness, 0.4);
            let preview = gpui_bridge::from_palette_hsla(color);
            assert_eq!(preview.h, hue / 360.0);
            let restored = gpui_bridge::to_palette_hsla(preview);
            assert_eq!(restored, color);
        }
    }
}

#[test]
fn display_p3_fallback_keeps_p3_sources_exact_and_maps_wider_sources() {
    // In-gamut P3, including P3-only red, is returned with its exact components.
    let p3_red = ColorValue::display_p3(1.0, 0.0, 0.0, 0.4);
    for policy in [GamutMapping::Clip, GamutMapping::CssLocalMinde] {
        assert_eq!(p3_red.to_display_p3_fallback(policy).unwrap(), DisplayP3Color::new(1.0, 0.0, 0.0, 0.4));
    }
    // sRGB red lies inside P3: no mapping, just a primaries conversion.
    let srgb_red = ColorValue::srgb(1.0, 0.0, 0.0, 1.0).to_display_p3_fallback(GamutMapping::CssLocalMinde).unwrap();
    assert!(srgb_red.red < 1.0 && srgb_red.green > 0.0, "{srgb_red:?}");
    let back = ColorValue::DisplayP3(srgb_red).to_srgba_unclamped().unwrap();
    assert!((back.red - 1.0).abs() < 1e-4 && back.green.abs() < 1e-4 && back.blue.abs() < 1e-4);
    // Rec. 2020-like chroma beyond P3 is bounded into the P3 cube by either policy.
    let wide = ColorValue::oklch(0.65, 0.4, 145.0, 1.0);
    assert!(!wide.is_in_gamut(Gamut::DisplayP3).unwrap());
    for policy in [GamutMapping::Clip, GamutMapping::CssLocalMinde] {
        let mapped = wide.to_display_p3_fallback(policy).unwrap();
        for v in [mapped.red, mapped.green, mapped.blue] {
            assert!((0.0..=1.0).contains(&v), "{policy:?}: {mapped:?}");
        }
    }
    // Unclamped conversion retains channels outside P3; the source is untouched.
    let unclamped = wide.to_display_p3_unclamped().unwrap();
    assert!([unclamped.red, unclamped.green, unclamped.blue].iter().any(|v| !(0.0..=1.0).contains(v)));
    assert_eq!(wide, ColorValue::oklch(0.65, 0.4, 145.0, 1.0));
}

#[test]
fn display_p3_fallback_retains_more_chroma_than_srgb_fallback() {
    let source = ColorValue::display_p3(0.0, 1.0, 0.0, 1.0);
    let srgb = ColorValue::Srgb(source.to_srgba_fallback(GamutMapping::CssLocalMinde).unwrap());
    let p3 = ColorValue::DisplayP3(source.to_display_p3_fallback(GamutMapping::CssLocalMinde).unwrap());
    let chroma = |c: ColorValue| c.to_oklcha_unclamped().unwrap().chroma;
    assert!(chroma(p3) > chroma(srgb) + 0.02, "{} vs {}", chroma(p3), chroma(srgb));
}
