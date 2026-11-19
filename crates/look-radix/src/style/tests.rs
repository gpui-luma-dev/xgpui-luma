use gpui::{BoxShadow, Hsla};
use xgpui_luma::style::lss::{AttrValue, Color, ControlId, PartId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::{InteractionState, ThemeMode};

use super::*;
use crate::button::Paint;
use crate::button_layout::Radius;
use crate::checkbox::{CheckboxSize, CheckboxVariant, checkbox_theme_for};
use crate::palette::{Accent, Gray};

/// The built-in look and a bright accent whose solid faces take dark labels.
fn looks() -> Vec<(&'static str, Look)> {
    vec![
        ("indigo", Look::built_in()),
        (
            "lime",
            Look::from_palettes(Accent::Lime, Gray::Auto, crate::button_layout::metric_tokens(), ThemeMode::Light),
        ),
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
    for (name, look) in looks() {
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
    let config = config();
    let state = |name: &str| config.state(name).expect("state");
    let attr = |name: &str| config.attribute(name).expect("attribute");
    let (variant_attr, size_attr, tone_attr, radius_attr) =
        (attr("variant"), attr("size"), attr("tone"), attr("radius"));
    let checkbox = ControlId::new("checkbox");
    let (indicator, glyph, label) = (PartId::new("indicator"), PartId::new("glyph"), PartId::new("label"));
    let variants = [
        (CheckboxVariant::Surface, None),
        (CheckboxVariant::Classic, Some("classic")),
        (CheckboxVariant::Soft, Some("soft")),
    ];
    let sizes = [(CheckboxSize::One, Some("1")), (CheckboxSize::Two, None), (CheckboxSize::Three, Some("3"))];
    let radii = [
        (Radius::None, Some("none")),
        (Radius::Small, Some("small")),
        (Radius::Medium, None),
        (Radius::Large, Some("large")),
        (Radius::Full, Some("full")),
    ];
    let paints = [
        (Paint::accent(), None, false),
        (Paint::gray(), Some("gray"), false),
        (Paint::accent().high_contrast(), None, true),
        (Paint::gray().high_contrast(), Some("gray"), true),
    ];

    for (name, look) in looks() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let theme = theme(&look).unwrap_or_else(|d| panic!("{name}: binds:\n{d}"));
            let sheet = theme.stylesheet();
            let length = |name: &str| sheet.prop::<f32>(checkbox, name).expect("declared");
            for (variant, variant_key) in variants {
                for (size, size_key) in sizes {
                    let sized = look.with_size(size.control_size());
                    let border_width = sized.metrics_for(size.control_size()).border_width;
                    for (radius, radius_key) in radii {
                        for (paint, tone_key, high_contrast) in paints {
                            let native = checkbox_theme_for(&sized, variant, paint, size, radius);
                            for bits in 0..32u8 {
                                let s = InteractionState {
                                    hovered: bits & 1 != 0,
                                    pressed: bits & 2 != 0,
                                    focused: bits & 4 != 0,
                                    disabled: bits & 8 != 0,
                                    invalid: bits & 16 != 0,
                                };
                                for checked in [false, true] {
                                    let palette = native.resolve(checked, s);
                                    for scale in [1.0, 1.5] {
                                        let case = format!(
                                            "{name} {mode:?} {variant:?} {size:?} {radius:?} {paint:?} {s:?} checked={checked} @{scale}x"
                                        );
                                        let key = StyleKey::new(checkbox)
                                            .attr_opt(variant_attr, variant_key.map(AttrValue::new))
                                            .attr_opt(size_attr, size_key.map(AttrValue::new))
                                            .attr_opt(radius_attr, radius_key.map(AttrValue::new))
                                            .attr_opt(tone_attr, tone_key.map(AttrValue::new))
                                            .state_if(state("high_contrast"), high_contrast)
                                            .state_if(state("hover"), s.hovered)
                                            .state_if(state("pressed"), s.pressed)
                                            .state_if(state("focused"), s.focused)
                                            .state_if(state("disabled"), s.disabled)
                                            .state_if(state("invalid"), s.invalid)
                                            .state_if(state("checked"), checked)
                                            .scale(scale);
                                        let resolved = theme.resolve(&key);

                                        let color = |got: Option<Color>, want: Hsla, what: &str| {
                                            let got = got
                                                .map(Hsla::from)
                                                .unwrap_or_else(|| panic!("{case}: {what} is unset"));
                                            assert!(close(got, want), "{case}: {what}: {got:?} != {want:?}");
                                        };
                                        color(
                                            resolved.part(indicator).background(),
                                            palette.indicator_background,
                                            "indicator",
                                        );
                                        color(
                                            resolved.part(indicator).border(),
                                            palette.indicator_border,
                                            "indicator border",
                                        );
                                        color(resolved.part(glyph).foreground(), palette.checkmark_color, "checkmark");
                                        color(resolved.part(label).foreground(), palette.label_color, "label");
                                        color(resolved.root().focus_border(), palette.focus_border, "focus border");
                                        assert_eq!(resolved.root().background(), None, "{case}: control background");
                                        assert_eq!(resolved.root().border(), None, "{case}: control border");
                                        let shadows = resolved
                                            .part(indicator)
                                            .shadow()
                                            .map(|s| box_shadows(&s))
                                            .unwrap_or_default();
                                        let want = palette.indicator_shadow.clone().unwrap_or_default();
                                        assert!(
                                            same_shadows(&shadows, &want),
                                            "{case}: shadow {shadows:?} != {want:?}"
                                        );

                                        // Native Radix geometry is unsnapped; the `.lss` checkbox snaps lengths
                                        // to device pixels like the SDK.
                                        let metrics = native.scale(scale);
                                        let snap = |v: f32| xgpui_luma::theme::snap_to_pixel(v, scale);
                                        let root = resolved.root();
                                        let number = |name: &str| root.value(name).and_then(Value::as_number);
                                        let near = |got: Option<f32>, want: f32, what: &str| {
                                            assert!(
                                                got.is_some_and(|g| (g - want).abs() < 1e-4),
                                                "{case}: {what}: {got:?} != {want}"
                                            );
                                        };
                                        near(
                                            root.get(length("indicator_size")),
                                            snap(metrics.indicator_size),
                                            "indicator_size",
                                        );
                                        near(root.get(length("glyph_size")), snap(metrics.glyph_size), "glyph_size");
                                        near(
                                            root.get(length("indicator_radius")),
                                            snap(metrics.indicator_radius),
                                            "indicator_radius",
                                        );
                                        near(
                                            root.get(length("label_baseline_shift")),
                                            metrics.label_baseline_shift,
                                            "baseline",
                                        );
                                        near(root.height(), snap(metrics.height), "height");
                                        near(root.gap(), snap(metrics.gap), "gap");
                                        near(root.radius(), snap(metrics.control_radius), "control radius");
                                        near(number("padding_x"), metrics.control_padding_x, "padding x");
                                        near(number("padding_y"), metrics.control_padding_y, "padding y");
                                        near(number("disabled_opacity"), metrics.disabled_opacity, "disabled opacity");
                                        near(number("border_width"), border_width, "border width");

                                        let text = resolved.part(label);
                                        let typography = palette.label_typography;
                                        near(text.font_size(), typography.size, "font size");
                                        near(text.line_height(), typography.line_height, "line height");
                                        near(
                                            text.value("font_weight").and_then(Value::as_number),
                                            typography.weight.0,
                                            "weight",
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
    }
}

#[test]
fn style_theme_follows_the_mode_and_palettes() {
    let checkbox = ControlId::new("checkbox");
    let checked = config().state("checked").expect("checked");
    fn color(look: &Look, key: StyleKey, part: &'static str) -> Hsla {
        let resolved = look.style_theme().resolve(&key);
        let part = resolved.part(PartId::new(part));
        part.background().or(part.foreground()).map(Hsla::from).expect("color")
    }
    let look = Look::built_in().fork();
    let light = color(&look, StyleKey::new(checkbox), "label");
    look.set_mode(ThemeMode::Dark);
    let dark = color(&look, StyleKey::new(checkbox), "label");
    assert!(!close(light, dark), "the cached theme follows the mode");
    assert!(close(dark, look.resolve_role(SemanticRole::Foreground).hsla()));
    look.set_palettes(Accent::Crimson, Gray::Auto);
    let fill = color(&look, StyleKey::new(checkbox).state(checked), "indicator");
    assert!(close(fill, look.resolve_role(SemanticRole::Primary).hsla()), "and the palettes");
}
