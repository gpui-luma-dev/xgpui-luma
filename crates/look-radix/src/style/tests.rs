use gpui::{BoxShadow, Hsla};
use xgpui_luma::style::lss::{AttrValue, ControlId, PartId, StyleKey, Value, box_shadows};
use xgpui_luma::theme::ThemeMode;

use super::*;
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
    // `Theme::check` covers every declared mode for each control, and the native tokens hold both
    // schemes, so one bound theme per look is enough. Looks are independent, so they run in parallel.
    std::thread::scope(|scope| {
        let runs: Vec<_> = looks()
            .into_iter()
            .map(|(name, look)| {
                scope.spawn(move || {
                    let theme = theme(&look).unwrap_or_else(|d| panic!("{name}: binds:\n{d}"));
                    theme
                        .stylesheet()
                        .controls()
                        .filter_map(|control| {
                            let problems = theme.check(control);
                            (!problems.is_empty()).then(|| format!("{name} {control}: {problems}"))
                        })
                        .collect::<Vec<String>>()
                })
            })
            .collect();
        let problems: Vec<String> = runs.into_iter().flat_map(|run| run.join().expect("no panic")).collect();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    });
}

/// A resolved Radix checkbox: `attrs` as (attribute, value), `states` by name.
fn resolve(
    control: &'static str,
    look: &Look,
    attrs: &[(&str, &str)],
    states: &[&str],
) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    let config = config();
    let key = attrs.iter().fold(StyleKey::new(ControlId::new(control)), |key, (attr, value)| {
        key.attr(config.attribute(attr).expect("attribute"), AttrValue::named(value))
    });
    let key = states.iter().fold(key, |key, state| key.state(config.state(state).expect("state")));
    look.style_theme().resolve(&key)
}

fn checkbox(look: &Look, attrs: &[(&str, &str)], states: &[&str]) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    resolve("checkbox", look, attrs, states)
}

fn radio(look: &Look, attrs: &[(&str, &str)], states: &[&str]) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    resolve("radio", look, attrs, states)
}

fn color(resolved: &xgpui_luma::style::lss::Resolved, part: &'static str, what: &str) -> Hsla {
    let part = resolved.part(PartId::new(part));
    let color = match what {
        "background" => part.background(),
        "border" => part.border(),
        _ => part.foreground(),
    };
    color.map(Hsla::from).unwrap_or_else(|| panic!("{what} is unset"))
}

fn shadow(resolved: &xgpui_luma::style::lss::Resolved) -> Vec<BoxShadow> {
    resolved.part(PartId::new("indicator")).shadow().map(|s| box_shadows(&s)).unwrap_or_default()
}

fn step(look: &Look, family: crate::scale::ScaleFamily, n: u8) -> Hsla {
    look.resolve_step(family, n).hsla()
}

// Behaviour of the native Radix checkbox, kept as checks on the `.lss` checkbox.

#[test]
fn pressed_keeps_the_resting_fill_and_only_hover_changes_the_checked_fill() {
    use crate::scale::ScaleFamily::Color as Accent;
    for (name, look) in looks() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [&[][..], &[("variant", "classic")], &[("variant", "soft")]] {
                for checked in [&[][..], &["checked"]] {
                    let at = |extra: &[&str]| checkbox(&look, variant, &[checked, extra].concat());
                    let focused = at(&["focused"]);
                    for pressed in [&["focused", "pressed"][..], &["focused", "pressed", "hover"]] {
                        let pressed = at(pressed);
                        for (part, what) in [("indicator", "background"), ("indicator", "border"), ("glyph", "fg")] {
                            assert!(
                                close(color(&pressed, part, what), color(&focused, part, what)),
                                "{name} {mode:?} {variant:?} {checked:?}: pressed {part} {what}"
                            );
                        }
                    }
                }
            }
            let hovered = checkbox(&look, &[], &["checked", "hover"]);
            assert!(close(color(&hovered, "indicator", "background"), step(&look, Accent, 10)), "{name} {mode:?}");
        }
    }
}

#[test]
fn surface_fills_with_the_accent_behind_a_gray_edge() {
    let look = Look::built_in();
    let unchecked = checkbox(&look, &[], &[]);
    let checked = checkbox(&look, &[], &["checked"]);
    assert!(close(color(&unchecked, "indicator", "border"), step(&look, crate::scale::ScaleFamily::Gray, 7)));
    assert!(close(color(&checked, "indicator", "background"), look.resolve_role(SemanticRole::Primary).hsla()));
    assert!(shadow(&unchecked).is_empty() && shadow(&checked).is_empty(), "surface stays flat");
}

#[test]
fn soft_keeps_one_accent_tint_in_both_check_states() {
    use crate::scale::ScaleFamily::Color as Accent;
    let look = Look::built_in();
    for checked in [&[][..], &["checked"]] {
        let soft = checkbox(&look, &[("variant", "soft")], checked);
        assert!(close(color(&soft, "indicator", "background"), step(&look, Accent, 5)));
    }
    let checked = checkbox(&look, &[("variant", "soft")], &["checked"]);
    assert!(close(color(&checked, "glyph", "fg"), step(&look, Accent, 11)));
}

#[test]
fn classic_is_the_only_variant_that_carries_a_shadow() {
    let look = Look::built_in();
    for checked in [&[][..], &["checked"]] {
        assert!(!shadow(&checkbox(&look, &[("variant", "classic")], checked)).is_empty());
        assert!(shadow(&checkbox(&look, &[], checked)).is_empty());
        assert!(shadow(&checkbox(&look, &[("variant", "soft")], checked)).is_empty());
    }
}

#[test]
fn high_contrast_uses_step_twelve() {
    use crate::scale::ScaleFamily::Color as Accent;
    let look = Look::built_in();
    let surface = checkbox(&look, &[], &["checked", "high_contrast"]);
    assert!(close(color(&surface, "indicator", "background"), step(&look, Accent, 12)));
    assert!(close(color(&surface, "glyph", "fg"), step(&look, Accent, 1)));
    let soft = checkbox(&look, &[("variant", "soft")], &["checked", "high_contrast"]);
    assert!(close(color(&soft, "indicator", "background"), step(&look, Accent, 5)), "soft keeps its tint");
    assert!(close(color(&soft, "glyph", "fg"), step(&look, Accent, 12)), "and darkens only the checkmark");
}

#[test]
fn gray_tone_reads_the_gray_scale() {
    use crate::scale::ScaleFamily::Gray;
    let look = Look::built_in();
    let soft = checkbox(&look, &[("variant", "soft"), ("tone", "gray")], &[]);
    assert!(close(color(&soft, "indicator", "background"), step(&look, Gray, 5)));
}

#[test]
fn checkbox_radius_follows_size_and_radius_but_never_pills() {
    let look = Look::built_in();
    let radius = |attrs: &[(&str, &str)]| {
        let resolved = checkbox(&look, attrs, &[]);
        resolved.root().value("indicator_radius").and_then(Value::as_number).expect("radius")
    };
    let size = |attrs: &[(&str, &str)]| {
        checkbox(&look, attrs, &[]).root().value("indicator_size").and_then(Value::as_number).expect("size")
    };
    assert_eq!(radius(&[]), 3.0, "size 2, medium: --radius-1");
    assert_eq!(size(&[]), 16.0);
    assert_eq!(radius(&[("radius", "full")]), radius(&[("radius", "large")]));
    assert!(radius(&[("radius", "full")]) < size(&[]) / 2.0);
    assert_eq!(radius(&[("radius", "none")]), 0.0);
    assert_eq!(size(&[("size", "1")]), 14.0);
    assert_eq!(size(&[("size", "3")]), 20.0);
}

#[test]
fn bright_accents_take_dark_checkmarks() {
    let (_, lime) = looks().pop().expect("lime");
    let checked = checkbox(&lime, &[], &["checked"]);
    assert!(close(color(&checked, "glyph", "fg"), lime.resolve_role(SemanticRole::PrimaryForeground).hsla()));
    assert!(color(&checked, "glyph", "fg").l < 0.5);
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

// Behaviour of the native Radix radio, kept as checks on the `.lss` radio.

#[test]
fn radio_press_keeps_the_resting_fill() {
    let look = Look::built_in();
    for variant in [&[][..], &[("variant", "classic")], &[("variant", "soft")]] {
        for selected in [&[][..], &["selected"]] {
            let focused = radio(&look, variant, &[selected, &["focused"]].concat());
            let pressed = radio(&look, variant, &[selected, &["focused", "pressed", "hover"]].concat());
            for (part, what) in [("indicator", "background"), ("indicator", "border"), ("dot", "fg")] {
                assert!(
                    close(color(&pressed, part, what), color(&focused, part, what)),
                    "{variant:?} {selected:?} {part}"
                );
            }
        }
    }
}

#[test]
fn radio_soft_keeps_one_tint_and_only_classic_has_a_shadow() {
    use crate::scale::ScaleFamily::Color as Accent;
    let look = Look::built_in();
    for selected in [&[][..], &["selected"]] {
        let soft = radio(&look, &[("variant", "soft")], selected);
        assert!(close(color(&soft, "indicator", "background"), step(&look, Accent, 4)));
        assert!(shadow(&soft).is_empty());
        assert!(shadow(&radio(&look, &[], selected)).is_empty());
        assert!(!shadow(&radio(&look, &[("variant", "classic")], selected)).is_empty());
    }
}

#[test]
fn radio_high_contrast_and_bright_accents() {
    use crate::scale::ScaleFamily::Color as Accent;
    let look = Look::built_in();
    let hc = radio(&look, &[], &["selected", "high_contrast"]);
    assert!(close(color(&hc, "indicator", "background"), step(&look, Accent, 12)));
    assert!(close(color(&hc, "dot", "fg"), step(&look, Accent, 1)));
    let (_, lime) = looks().pop().expect("lime");
    let selected = radio(&lime, &[], &["selected"]);
    assert!(color(&selected, "dot", "fg").l < 0.5, "dark dot on a bright accent");
}

#[test]
fn radio_is_round_at_every_size() {
    let look = Look::built_in();
    for (attrs, edge) in [(&[("size", "1")][..], 14.0), (&[], 16.0), (&[("size", "3")], 20.0)] {
        let resolved = radio(&look, attrs, &[]);
        let root = resolved.root();
        let number = |name: &str| root.value(name).and_then(Value::as_number).expect("number");
        assert_eq!(number("indicator_size"), edge);
        assert_eq!(number("indicator_radius"), edge);
        assert_eq!(root.radius(), Some(edge / 2.0));
    }
}

// Behaviour of the native Radix switch, kept as checks on the `.lss` switch.

#[test]
fn switch_thumb_is_white_and_light_gray_when_disabled_in_both_modes() {
    let look = Look::built_in().fork();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        assert!(close(color(&switch(&look, &[], &["checked"]), "thumb", "background"), gpui::white()));
        let disabled = switch(&look, &[], &["disabled"]);
        assert!(close(color(&disabled, "thumb", "background"), look.light_gray_step(2)), "{mode:?}");
    }
}

#[test]
fn switch_variants_fill_as_radix_does() {
    use crate::scale::ScaleFamily::Color as Accent;
    let look = Look::built_in();
    let on = switch(&look, &[], &["checked"]);
    assert!(close(color(&on, "track", "background"), look.resolve_role(SemanticRole::Primary).hsla()));
    let hc = switch(&look, &[], &["checked", "high_contrast"]);
    assert!(close(color(&hc, "track", "background"), step(&look, Accent, 12)));
    let soft = switch(&look, &[("variant", "soft")], &["checked"]);
    assert!(close(color(&soft, "track", "background"), step(&look, Accent, 4)));
    let soft_hc = switch(&look, &[("variant", "soft")], &["checked", "high_contrast"]);
    assert!(close(color(&soft_hc, "track", "background"), step(&look, Accent, 6)));
    let classic = switch(&look, &[("variant", "classic")], &["checked"]);
    assert!(
        !close(color(&classic, "track", "border"), color(&classic, "track", "background")),
        "classic keeps an edge"
    );
    for on in [&[][..], &["checked"]] {
        assert_eq!(track_shadow(&switch(&look, &[], on)).len(), 2);
    }
}

#[test]
fn switch_sizes_and_radius() {
    let look = Look::built_in();
    let number = |attrs: &[(&str, &str)], name: &str| {
        switch(&look, attrs, &[]).root().value(name).and_then(Value::as_number).expect("number")
    };
    for (attrs, width, height, thumb) in [
        (&[("size", "1")][..], 28.0, 16.0, 14.0),
        (&[], 35.0, 20.0, 18.0),
        (&[("size", "3")], 42.0, 24.0, 22.0),
    ] {
        assert_eq!(
            (number(attrs, "track_width"), number(attrs, "track_height"), number(attrs, "thumb_size")),
            (width, height, thumb)
        );
    }
    assert_eq!(number(&[], "track_radius"), 10.0, "medium: a pill");
    assert_eq!(number(&[("radius", "small")], "track_radius"), 3.0, "4 × 0.75");
    assert!(number(&[("radius", "none")], "track_radius") <= 1.0, "nearly square");
}

fn switch(look: &Look, attrs: &[(&str, &str)], states: &[&str]) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    resolve("switch", look, attrs, states)
}

fn track_shadow(resolved: &xgpui_luma::style::lss::Resolved) -> Vec<BoxShadow> {
    resolved.part(PartId::new("track")).shadow().map(|s| box_shadows(&s)).unwrap_or_default()
}

/// The SDK keeps only behaviour variants (#287), so the look draws the visual ones: each differs
/// from the base trigger or shell, in both modes.
#[test]
fn look_defines_the_visual_variants() {
    let look = Look::built_in().fork();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let theme = look.style_theme();
        let variant = theme.stylesheet().config().attribute("variant").expect("variant attribute");
        for (control, name) in [
            ("pager", "minimal"),
            ("popup_menu", "outline"),
            ("popup_menu", "ghost"),
            ("scrollbar", "soft"),
            ("selector", "outline"),
            ("selector", "ghost"),
            ("toolbar", "ghost"),
        ] {
            let base = StyleKey::new(ControlId::new(control));
            let styled = base.attr(variant, AttrValue::named(name));
            assert_ne!(theme.resolve(&styled), theme.resolve(&base), "{mode:?} {control} [{name}]");
        }
    }
}

/// Radix's radio group: side-by-side rows 8 apart and disabled rows at full opacity (the radio template
/// dims the whole row); the stacked and item gaps are the SDK's (#287).
#[test]
fn radio_group_layout_comes_from_the_look_lss() {
    use xgpui_luma::controls::radio_group::RadioGroupGeometry;
    let look = Look::built_in().fork();
    let geometry = RadioGroupGeometry::from_binding(&xgpui_luma::style::StyleBinding::new(
        "radio_group",
        crate::textfield::live_source(&look),
    ));
    let sdk = RadioGroupGeometry::sdk();
    assert_eq!((geometry.horizontal_gap, geometry.disabled_opacity), (8.0, 1.0));
    assert_eq!((geometry.vertical_gap, geometry.item_gap), (sdk.vertical_gap, sdk.item_gap));
}

/// The SDK's button family draws `:disabled` without fading; Radix keeps its fade on every control that
/// extends the button, the text field, the choices, groups and bars (#397).
#[test]
fn disabled_controls_keep_the_radix_fade() {
    let theme = Look::built_in().style_theme();
    let opacity = |control: &str| {
        let key = theme.key_named(control, &[], &[]).expect("key");
        theme.resolve(&key).root().value("disabled_opacity").and_then(Value::as_number)
    };
    for control in [
        "button",
        "toggle",
        "popup_menu",
        "selector",
        "textfield",
        "textarea",
        "checkbox",
        "radio",
        "switch",
        "control_group",
        "context_menu",
        "scrollbar",
        "slider",
        "resizable_panels",
    ] {
        assert!(opacity(control).is_some_and(|value| value < 1.0), "{control}: {:?}", opacity(control));
    }
}
