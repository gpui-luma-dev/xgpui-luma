use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use xgpui_luma::theme::{InteractionState, ThemeMode};

use crate::controls::button::{ButtonRadiusPreset, button_look_with_stylesheet};
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet};
use crate::ShadcnButtonStyle;
use crate::size::ShadcnSize;

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
            for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
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
        ShadcnSize::Md,
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
        ShadcnSize::Md,
        None,
        InteractionState::default(),
    );
    assert_eq!(fallback.typography.size, tokens.typography.text.label.size);
    assert_eq!(fallback.icon_size, fallback.typography.size);
}
