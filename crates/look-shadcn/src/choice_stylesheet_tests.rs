use xgpui_luma::controls::checkbox::CheckboxScale;
use xgpui_luma::controls::radio_button::RadioScale;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::stylesheet::{StylesheetConfig, embedded_stylesheet};
use crate::{ShadcnButtonStyle, ShadcnLook};

fn customized_look() -> ShadcnLook {
    let mut config = embedded_stylesheet().clone();
    let common = StylesheetConfig::parse(
        r#"
[common.checkbox.geometry]
indicator_size = 31.25
glyph_size = 13.25
gap = 5.25
[common.checkbox.sizes.md]
indicator_size = 0
glyph_size = 0
gap = 0
[common.radio.geometry]
indicator_size = 29.25
dot_size = 11.25
gap = 7.25
[common.radio.sizes.md]
indicator_size = 0
dot_size = 0
gap = 0
[common.button.sizes.md]
font_size = 17
line_height = 24
"#,
    )
    .unwrap()
    .common;
    config.common.checkbox = common.checkbox;
    config.common.radio = common.radio;
    config.common.button.sizes.insert("md".into(), common.button.sizes.md.clone());
    for rule in &mut config.checkbox.color_rules {
        rule.indicator_background = "secondary".into();
        rule.checkmark_color = "secondary-foreground".into();
        rule.label_color = "destructive".into();
    }
    for rule in &mut config.radio.color_rules {
        rule.indicator_background = "secondary".into();
        rule.selection_ring = "primary".into();
        rule.dot_color = "secondary-foreground".into();
        rule.label_color = "destructive".into();
    }
    for rule in &mut config.checkbox.elevation_rules {
        rule.shadow = "none".into();
    }
    for rule in &mut config.radio.elevation_rules {
        rule.shadow = "shadow-xs".into();
    }
    ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap()
}

#[test]
fn default_choice_geometry_preserves_sdk_scales_and_palette_baselines() {
    for look in [crate::test_support::fallback_look(), crate::test_support::built_in_look("retro-arcade")] {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for factor in [1.0, 1.25, 2.0] {
                    assert_eq!(
                        look.checkbox_theme().scale(size, factor),
                        CheckboxScale::compute(size, &tokens.metrics, factor)
                    );
                    assert_eq!(
                        look.radio_button_theme().scale(size, factor),
                        RadioScale::compute(size, &tokens.metrics, factor)
                    );
                }
                for style in [
                    ShadcnButtonStyle::Primary,
                    ShadcnButtonStyle::Secondary,
                    ShadcnButtonStyle::Outline,
                    ShadcnButtonStyle::Ghost,
                    ShadcnButtonStyle::ContentOnly,
                ] {
                    let checkbox = crate::controls::templates::checkbox_theme_with_style(look.clone(), style);
                    let radio = crate::controls::templates::radio_button_theme_with_style(look.clone(), style);
                    for selected in [false, true] {
                        for bits in 0..32 {
                            let state = state(bits);
                            let old_checkbox = crate::paint::checkbox_look(&tokens, style, selected, state, size);
                            let old_radio = crate::paint::radio_button_look(&tokens, style, selected, state, size);
                            assert_eq!(
                                format!("{:?}", checkbox.resolve(selected, state, size)),
                                format!("{old_checkbox:?}")
                            );
                            assert_eq!(format!("{:?}", radio.resolve(selected, state, size)), format!("{old_radio:?}"));
                        }
                    }
                }
            }
        }
    }
}

fn state(bits: u32) -> InteractionState {
    InteractionState {
        hovered: bits & 1 != 0,
        pressed: bits & 2 != 0,
        focused: bits & 4 != 0,
        disabled: bits & 8 != 0,
        invalid: bits & 16 != 0,
    }
}

#[test]
fn common_choice_geometry_supports_general_size_zero_and_pixel_snapping() {
    let look = customized_look();
    let checkbox = look.checkbox_theme();
    let radio = look.radio_button_theme();
    let zero_checkbox = checkbox.scale(ControlSize::Md, 2.0);
    let zero_radio = radio.scale(ControlSize::Md, 2.0);
    assert_eq!((zero_checkbox.indicator_size, zero_checkbox.glyph_size, zero_checkbox.gap), (0.0, 0.0, 0.0));
    assert_eq!((zero_radio.indicator_size, zero_radio.dot_size, zero_radio.gap), (0.0, 0.0, 0.0));
    for size in [ControlSize::Sm, ControlSize::Lg] {
        let checkbox = checkbox.scale(size, 2.0);
        let radio = radio.scale(size, 2.0);
        assert_eq!((checkbox.indicator_size, checkbox.glyph_size, checkbox.gap), (31.5, 13.5, 5.5));
        assert_eq!((radio.indicator_size, radio.dot_size, radio.gap), (29.5, 11.5, 7.5));
        let sdk_checkbox = CheckboxScale::compute(size, &look.mode_tokens().metrics, 2.0);
        assert_eq!(checkbox.height, sdk_checkbox.height);
        assert_eq!(checkbox.indicator_radius, sdk_checkbox.indicator_radius);
        assert_eq!(checkbox.control_radius, sdk_checkbox.control_radius);
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selected_choice_stylesheet_matches_paint_and_inspection_across_states() {
    let look = customized_look();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let inspect = crate::inspect::ShadcnInspect::new(&look);
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let checkbox_scale = look.checkbox_theme().scale(size, 1.0);
            let radio_scale = look.radio_button_theme().scale(size, 1.0);
            let c = inspect.inspect_checkbox_metrics(size);
            let r = inspect.inspect_radio_button_metrics(size);
            assert_eq!(c.indicator_size.value_px, checkbox_scale.indicator_size);
            assert_eq!(c.glyph_size.value_px, checkbox_scale.glyph_size);
            assert_eq!(c.gap.value_px, checkbox_scale.gap);
            assert_eq!(c.height.value_px, checkbox_scale.height);
            assert_eq!(r.indicator_size.value_px, radio_scale.indicator_size);
            assert_eq!(r.dot_size.value_px, radio_scale.dot_size);
            assert_eq!(r.gap.value_px, radio_scale.gap);
            assert_eq!(r.height.value_px, radio_scale.height);
            assert!(format!("{:?}", c.indicator_size.source).contains("common.checkbox."));
            assert!(format!("{:?}", r.dot_size.source).contains("common.radio."));
            for style in [
                ShadcnButtonStyle::Primary,
                ShadcnButtonStyle::Secondary,
                ShadcnButtonStyle::Outline,
                ShadcnButtonStyle::Ghost,
                ShadcnButtonStyle::ContentOnly,
            ] {
                let checkbox = crate::controls::templates::checkbox_theme_with_style(look.clone(), style);
                let radio = crate::controls::templates::radio_button_theme_with_style(look.clone(), style);
                for selected in [false, true] {
                    for bits in 0..32 {
                        let state = state(bits);
                        let paint_c = checkbox.resolve(selected, state, size);
                        let paint_r = radio.resolve(selected, state, size);
                        let c = inspect.inspect_checkbox_color_palette(style, selected, state);
                        let r = inspect.inspect_radio_button_color_palette(style, selected, state);
                        assert_eq!(c.indicator_background.hsla(), paint_c.indicator_background);
                        assert_eq!(c.indicator_border.hsla(), paint_c.indicator_border);
                        assert_eq!(c.checkmark_color.hsla(), paint_c.checkmark_color);
                        assert_eq!(c.label_color.hsla(), paint_c.label_color);
                        assert_eq!(r.indicator_background.hsla(), paint_r.indicator_background);
                        assert_eq!(r.indicator_border.hsla(), paint_r.indicator_border);
                        assert_eq!(r.dot_color.hsla(), paint_r.dot_color);
                        assert_eq!(r.label_color.hsla(), paint_r.label_color);
                        assert_eq!(paint_c.label_color, look.mode_tokens().catalog.color("destructive").unwrap());
                        if size == ControlSize::Md {
                            assert_eq!(
                                (paint_c.label_typography.size, paint_c.label_typography.line_height),
                                (17.0, 24.0)
                            );
                            assert_eq!(
                                (paint_r.label_typography.size, paint_r.label_typography.line_height),
                                (17.0, 24.0)
                            );
                        }
                        let c = inspect.inspect_checkbox_elevation(style, selected, state);
                        let r = inspect.inspect_radio_button_elevation(style, selected, state);
                        assert_eq!(format!("{:?}", c.shadows), format!("{:?}", paint_c.indicator_shadow));
                        assert_eq!(format!("{:?}", r.shadows), format!("{:?}", paint_r.indicator_shadow));
                        assert!(paint_c.indicator_shadow.is_none());
                        assert_eq!(
                            r.applied,
                            style != ShadcnButtonStyle::ContentOnly
                                && !state.disabled
                                && paint_r.indicator_shadow.is_some()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn radio_indicator_defaults_and_live_replacement_use_selected_stylesheet() {
    let look = crate::test_support::fallback_look();
    let independent = crate::test_support::fallback_look();
    let checkbox = look.checkbox_theme();
    let radio = look.radio_button_theme();
    let old_checkbox = checkbox.scale(ControlSize::Md, 1.0);
    let old_radio = radio.scale(ControlSize::Md, 1.0);
    let mut config = embedded_stylesheet().clone();
    config.radio.indicator_defaults.primary = Some("dot".into());
    config.common.checkbox.geometry.indicator_size = Some(27.0);
    config.common.radio.geometry.dot_size = Some(6.0);
    let replacement = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    look.replace_theme(&replacement);
    assert_eq!(checkbox.scale(ControlSize::Md, 1.0).indicator_size, 27.0);
    assert_eq!(radio.scale(ControlSize::Md, 1.0).dot_size, 6.0);
    assert_eq!(independent.checkbox_theme().scale(ControlSize::Md, 1.0), old_checkbox);
    assert_eq!(independent.radio_button_theme().scale(ControlSize::Md, 1.0), old_radio);
    let palette = radio.resolve(true, InteractionState::default(), ControlSize::Md);
    assert_eq!(palette.dot_color, look.mode_tokens().catalog.color("primary").unwrap());
    assert_ne!(
        palette.dot_color,
        independent
            .radio_button_theme()
            .resolve(true, InteractionState::default(), ControlSize::Md)
            .dot_color
    );
}

#[test]
fn sparse_choice_overrides_keep_unset_sdk_dimensions() {
    let mut config = embedded_stylesheet().clone();
    config.common.checkbox.geometry.gap = Some(0.0);
    config.common.radio.geometry.gap = Some(0.0);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        let mut checkbox = CheckboxScale::compute(size, &look.mode_tokens().metrics, 1.0);
        let mut radio = RadioScale::compute(size, &look.mode_tokens().metrics, 1.0);
        checkbox.gap = 0.0;
        radio.gap = 0.0;
        assert_eq!(look.checkbox_theme().scale(size, 1.0), checkbox);
        assert_eq!(look.radio_button_theme().scale(size, 1.0), radio);
    }
}
