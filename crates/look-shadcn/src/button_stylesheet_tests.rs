use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

use crate::controls::button::{ButtonRadiusPreset, button_look_with_stylesheet};
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet};
use crate::{ShadcnButtonStyle, ShadcnLook};

fn legacy_stylesheet() -> StylesheetConfig {
    let mut stylesheet = embedded_stylesheet().clone();
    stylesheet.common.button = Default::default();
    stylesheet.common.toggle = Default::default();
    stylesheet.button.tokens.clear();
    stylesheet.toggle.tokens.clear();
    for (key, height, padding, toggle_height, toggle_padding, font, icon) in [
        ("sm", "metrics.control.sm", 12.0, "36", 10.0, 12.0, 14.0),
        ("md", "metrics.control.md", 16.0, "40", 12.0, 14.0, 16.0),
        ("lg", "metrics.control.lg", 20.0, "44", 20.0, 16.0, 18.0),
    ] {
        let legacy = StylesheetConfig::parse(&format!(
            r#"
[button.metrics.{key}]
height = "{height}"
padding_horizontal = {padding}
font_size = {font}
icon_size = {icon}
corner_radius = "radius"
[toggle.metrics.{key}]
height = "{toggle_height}"
padding_horizontal = {toggle_padding}
font_size = {font}
icon_size = {icon}
corner_radius = "radius"
"#
        ))
        .unwrap();
        stylesheet.button.metrics.extend(legacy.button.metrics);
        stylesheet.toggle.metrics.extend(legacy.toggle.metrics);
    }
    stylesheet
}

fn assert_same_look(actual: &ButtonFamilyLook, expected: &ButtonFamilyLook) {
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

#[test]
fn migrated_button_and_toggle_match_legacy_output_across_axes() {
    let legacy = legacy_stylesheet();
    for look in [crate::test_support::fallback_look(), crate::test_support::built_in_look("retro-arcade")] {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for style in [
                    ShadcnButtonStyle::Primary,
                    ShadcnButtonStyle::Secondary,
                    ShadcnButtonStyle::Outline,
                    ShadcnButtonStyle::Ghost,
                    ShadcnButtonStyle::ContentOnly,
                ] {
                    for role in [
                        ButtonFamilyRole::Text,
                        ButtonFamilyRole::Icon,
                        ButtonFamilyRole::Toggle { selected: false },
                        ButtonFamilyRole::Toggle { selected: true },
                    ] {
                        for bits in 0..32 {
                            let state = InteractionState {
                                hovered: bits & 1 != 0,
                                pressed: bits & 2 != 0,
                                focused: bits & 4 != 0,
                                disabled: bits & 8 != 0,
                                invalid: bits & 16 != 0,
                            };
                            let actual = button_look_with_stylesheet(
                                &tokens,
                                embedded_stylesheet(),
                                mode,
                                style,
                                role,
                                size,
                                None,
                                state,
                            );
                            let expected =
                                button_look_with_stylesheet(&tokens, &legacy, mode, style, role, size, None, state);
                            assert_same_look(&actual, &expected);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn common_geometry_beats_legacy_and_unset_keeps_role_fallbacks() {
    let look = crate::test_support::fallback_look();
    let tokens = look.mode_tokens();
    let mut config = legacy_stylesheet();
    let override_config = StylesheetConfig::parse(
        "[common.button.sizes.md]\nheight=50\npadding_x=0\nicon_size=0\nfont_size=0\nline_height=0",
    )
    .unwrap();
    config.common.button = override_config.common.button;
    let paint = button_look_with_stylesheet(
        &tokens,
        &config,
        look.mode(),
        ShadcnButtonStyle::Primary,
        ButtonFamilyRole::Text,
        ControlSize::Md,
        Some(ButtonRadiusPreset::Full),
        InteractionState::default(),
    );
    assert_eq!((paint.height, paint.padding_x, paint.icon_size, paint.radius), (50.0, 0.0, 0.0, 25.0));
    assert_eq!((paint.typography.size, paint.typography.line_height), (0.0, 0.0));
    let sparse = StylesheetConfig::default();
    let fallback = button_look_with_stylesheet(
        &tokens,
        &sparse,
        look.mode(),
        ShadcnButtonStyle::Primary,
        ButtonFamilyRole::Text,
        ControlSize::Md,
        None,
        InteractionState::default(),
    );
    assert_eq!(fallback.typography.size, tokens.typography.text.label.size);
    assert_eq!(fallback.icon_size, fallback.typography.size);
}

#[cfg(feature = "inspect")]
#[test]
fn selected_stylesheet_reaches_theme_paint_and_inspection() {
    let mut config = embedded_stylesheet().clone();
    let overrides = StylesheetConfig::parse(
        r#"
[common.button.sizes.md]
height = 52
padding_x = 7
padding_y = 3
gap = 9
icon_size = 21
font_size = 17
line_height = 23
[common.toggle.sizes.md]
height = 46
padding_x = 5
icon_size = 19
font_size = 15
line_height = 22
"#,
    )
    .unwrap();
    config.common.button = overrides.common.button;
    config.common.toggle = overrides.common.toggle;
    for rule in &mut config.button.color_rules {
        rule.background = "secondary".into();
    }
    for rule in &mut config.button.elevation_rules {
        if rule.style == "primary" {
            rule.shadow = "shadow-sm".into();
        }
    }
    for rule in &mut config.toggle.elevation_rules {
        rule.shadow = "none".into();
    }
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let family = crate::controls::templates::styled_button_family_theme(look.clone(), ShadcnButtonStyle::Primary);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        let scale = StandardBoxScale::compute(ControlSize::Md, &tokens.metrics, 1.0);
        let inspect = crate::inspect::ShadcnInspect::new(&look);
        for role in [ButtonFamilyRole::Text, ButtonFamilyRole::Icon, ButtonFamilyRole::Toggle { selected: true }] {
            let paint = family
                .resolve_look(role, ControlSize::Md, InteractionState::default(), &scale, tokens.metrics.radius.pill)
                .unwrap();
            let palette = family.resolve(role, ControlSize::Md, InteractionState::default());
            assert_eq!(palette.typography.size, paint.typography.size);
            assert_eq!(palette.typography.line_height, paint.typography.line_height);
            let metrics = inspect.inspect_button_metrics(
                ShadcnButtonStyle::Primary,
                role,
                ControlSize::Md,
                InteractionState::default(),
            );
            assert_eq!(metrics.height.value_px, paint.height);
            assert_eq!(metrics.padding_x.value_px, paint.padding_x);
            assert_eq!(metrics.icon_size.value_px, paint.icon_size);
            assert_eq!(metrics.radius.value_px, paint.radius);
            assert_eq!(metrics.gap.value_px, paint.gap);
            assert!(format!("{:?}", metrics.height.source).contains("common."));
            let typography = inspect.inspect_button_typography_for_size(ControlSize::Md, role);
            assert_eq!(typography.font_size.value.parse::<f32>().unwrap(), paint.typography.size);
            assert_eq!(typography.line_height.value.parse::<f32>().unwrap(), paint.typography.line_height);
            let colors =
                inspect.inspect_button_color_palette(ShadcnButtonStyle::Primary, role, InteractionState::default());
            assert_eq!(colors.background.hsla(), paint.background);
            let elevation =
                inspect.inspect_button_elevation(ShadcnButtonStyle::Primary, role, InteractionState::default());
            assert_eq!(format!("{:?}", elevation.shadows), format!("{:?}", paint.shadow));
            if matches!(role, ButtonFamilyRole::Toggle { .. }) {
                assert_eq!(paint.height, 46.0);
                assert!(paint.shadow.is_none());
            } else {
                assert_eq!(paint.height, 52.0);
                assert!(paint.shadow.is_some());
            }
        }
    }
}

#[test]
fn existing_button_theme_observes_replacement_without_changing_other_looks() {
    let look = crate::test_support::fallback_look();
    let independent = crate::test_support::fallback_look();
    let family = look.button_family_theme();
    let render = || {
        let tokens = look.mode_tokens();
        let scale = StandardBoxScale::compute(ControlSize::Md, &tokens.metrics, 1.0);
        family
            .resolve_look(
                ButtonFamilyRole::Text,
                ControlSize::Md,
                InteractionState::default(),
                &scale,
                tokens.metrics.radius.pill,
            )
            .unwrap()
    };
    let initial = render();
    let mut config = embedded_stylesheet().clone();
    config.common.button.sizes.get_mut("md").unwrap().height = Some(60.0);
    let replacement = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    look.replace_theme(&replacement);
    assert_eq!(render().height, 60.0);
    let tokens = independent.mode_tokens();
    let unaffected = button_look_with_stylesheet(
        &tokens,
        independent.stylesheet().as_ref(),
        independent.mode(),
        ShadcnButtonStyle::Secondary,
        ButtonFamilyRole::Text,
        ControlSize::Md,
        None,
        InteractionState::default(),
    );
    assert_same_look(&initial, &unaffected);
}
