//! Resolved-output baselines and customization boundaries for migrated families.
use super::*;
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};
use xgpui_luma::controls::{
    button_family::ButtonFamilyRole, textfield::TextFieldState, textarea::TextAreaState, toolbar::ToolbarVariant,
};

#[test]
fn size_and_state_baselines_survive_shared_geometry_migration() {
    let look = Look::built_in();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for (size, height, padding, gap, font, line) in [
            (ButtonSize::One, 24.0, 8.0, 4.0, 12.0, 16.0),
            (ButtonSize::Two, 32.0, 12.0, 8.0, 14.0, 20.0),
            (ButtonSize::Three, 40.0, 16.0, 12.0, 16.0, 24.0),
            (ButtonSize::Four, 48.0, 24.0, 12.0, 18.0, 26.0),
        ] {
            for radius in Radius::ALL {
                let box_ = crate::button_layout::button_box_with_stylesheet(&look.common_stylesheet(), size, radius);
                assert_eq!(
                    (box_.height, box_.padding_x, box_.gap, box_.typography.size, box_.typography.line_height),
                    (height, padding, gap, font, line)
                );
                assert_eq!(box_.radius, resolve_button_radius(size, radius));
            }
        }
        for (sdk, checkbox_size, radio_size, edge, glyph, font, line, progress_height) in [
            (ControlSize::Sm, CheckboxSize::One, RadioSize::One, 14.0, 9.0, 12.5, 18.0, 4.0),
            (ControlSize::Md, CheckboxSize::Two, RadioSize::Two, 16.0, 10.0, 14.0, 20.0, 6.0),
            (ControlSize::Lg, CheckboxSize::Three, RadioSize::Three, 20.0, 12.0, 16.0, 22.0, 8.0),
        ] {
            for radius in Radius::ALL {
                for variant in [CheckboxVariant::Classic, CheckboxVariant::Surface, CheckboxVariant::Soft] {
                    let actual =
                        checkbox_theme_for(&look, variant, Paint::accent(), checkbox_size, radius).scale(sdk, 2.0);
                    assert_eq!(
                        (actual.indicator_size, actual.glyph_size, actual.height, actual.gap),
                        (edge, glyph, edge, 8.0)
                    );
                    assert_eq!(actual.indicator_radius, resolve_checkbox_radius(checkbox_size, radius));
                }
            }
            for variant in [RadioVariant::Classic, RadioVariant::Surface, RadioVariant::Soft] {
                let actual = radio_theme_for(&look, variant, Paint::accent(), radio_size).scale(sdk, 2.0);
                assert_eq!(
                    (actual.indicator_size, actual.dot_size, actual.gap, actual.control_radius),
                    (edge, edge * 0.4, 8.0, edge / 2.0)
                );
            }
            let scale = StandardBoxScale::compute(sdk, &look.metrics(), 2.0);
            for bits in 0..16 {
                let hovered = bits & 1 != 0;
                let focused = bits & 2 != 0;
                let invalid = bits & 4 != 0;
                let enabled = bits & 8 == 0;
                for variant in TextFieldVariant::ALL {
                    let field = textfield_theme_with(&look, variant).resolve_look(
                        TextFieldState { hovered, focused, invalid, ..Default::default() },
                        enabled,
                        sdk,
                        &scale,
                    );
                    assert_eq!(
                        (field.min_height, field.padding_x, field.padding_y, field.radius),
                        (scale.height, scale.padding_x, scale.padding_y, scale.radius)
                    );
                    assert_eq!((field.typography.size, field.typography.line_height), (font, line));
                    let area = textarea_theme_with(&look, variant).resolve_look(
                        TextAreaState { hovered, focused, invalid, ..Default::default() },
                        enabled,
                        sdk,
                        &scale,
                    );
                    assert_eq!(
                        (area.min_height, area.padding_x, area.padding_y, area.radius),
                        (scale.height, scale.padding_x, scale.padding_y, scale.radius)
                    );
                    assert_eq!((area.typography.size, area.typography.line_height), (14.0, 20.0));
                }
            }
            for variant in ProgressVariant::ALL {
                for enabled in [true, false] {
                    assert_eq!(
                        progress_theme_with(&look, variant, Paint::accent()).resolve(enabled, sdk).track_height,
                        progress_height
                    );
                }
            }
        }
        let menu = crate::popup_menu::floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        assert_eq!(
            (menu.min_width, menu.item_height, menu.item_padding_x, menu.item_typography.size),
            (180.0, 32.0 * 0.9, 9.0, 14.0)
        );
        let tooltip = tooltip_theme(&look).resolve();
        assert_eq!(
            (tooltip.padding, tooltip.padding_y, tooltip.max_width, tooltip.text_size, tooltip.line_height),
            (8.0, 4.0, 260.0, 12.0, 16.0)
        );
    }
}

#[test]
fn numeric_ranges_remain_independent_of_inherited_sdk_presets() {
    let stylesheet = Look::built_in().common_stylesheet();
    for (sizes, count) in [
        (stylesheet.button.sizes.keys().collect::<Vec<_>>(), 4),
        (stylesheet.avatar.sizes.keys().collect(), 9),
        (stylesheet.checkbox.sizes.keys().collect(), 3),
        (stylesheet.radio.sizes.keys().collect(), 3),
        (stylesheet.badge.sizes.keys().collect(), 3),
    ] {
        assert_eq!(sizes.iter().filter(|key| key.parse::<usize>().is_ok()).count(), count);
        for index in 1..=count {
            assert!(sizes.iter().any(|key| **key == index.to_string()));
        }
    }
    // SDK helper recipes are inherited alongside Radix's own semantic sizes.
    assert_eq!(stylesheet.tooltip.sizes.custom.keys().map(String::as_str).collect::<Vec<_>>(), vec!["compact"]);
    assert_eq!(stylesheet.toolbar.sizes, Default::default());
}

#[test]
fn live_edits_zero_and_local_overrides_keep_forks_isolated() {
    let look = Look::built_in();
    let fork = look.fork();
    let checkbox =
        checkbox_theme_for(&fork, CheckboxVariant::Surface, Paint::accent(), CheckboxSize::One, Radius::Medium);
    let radio = radio_theme_for(&fork, RadioVariant::Surface, Paint::accent(), RadioSize::One);
    let field = textfield_theme(&fork);
    let area = textarea_theme(&fork);
    let progress = progress_theme(&fork);
    let toolbar = toolbar_theme(&fork);
    let explicit_toolbar = toolbar_theme_with(&fork, ToolbarStyle { gap: 17.0, ..Default::default() });
    let tooltip = tooltip_theme(&fork);
    let card = Card::new(&fork).variant(CardVariant::Ghost);
    let explicit_card = Card::new(&fork).style_override(|style| style.padding = 21.0);
    let avatar = Avatar::new(&fork, "AB").size(AvatarSize::Nine).radius(Radius::Full);
    let explicit_avatar = Avatar::new(&fork, "A").style_override(|style| style.diameter = 33.0);
    let mut config = fork.common_stylesheet();
    config.checkbox.sizes.get_mut("1").unwrap().indicator_size = Some(25.0);
    config.radio.sizes.get_mut("1").unwrap().dot_size = Some(0.0);
    config.textfield.geometry.padding_x = Some(0.0);
    config.textarea.geometry.min_height = Some(73.0);
    config.progress.sizes.get_mut("2").unwrap().track_height = Some(11.0);
    config.toolbar.geometry.gap = Some(0.0);
    config.tooltip.geometry.max_width = Some(420.0);
    config.card.sizes.get_mut("1").unwrap().padding = Some(18.0);
    config.avatar.sizes.get_mut("9").unwrap().diameter = Some(190.0);
    config.avatar.sizes.get_mut("9").unwrap().two_letter_font_size = Some(44.0);
    config.floating_menu.geometry.min_width = Some(222.0);
    config.context_menu.geometry.min_width = Some(240.0);
    config.overlay_window.sizes.get_mut("sm").unwrap().min_width = Some(300.0);
    config.button.sizes.get_mut("4").unwrap().height = Some(57.0);
    fork.set_common_stylesheet(config.clone());
    assert_ne!(look.common_stylesheet(), config);
    assert_eq!(checkbox.scale(ControlSize::Lg, 1.0).indicator_size, 25.0);
    assert_eq!(radio.scale(ControlSize::Lg, 1.0).dot_size, 0.0);
    let scale = StandardBoxScale::compute(ControlSize::Md, &fork.metrics(), 1.0);
    assert_eq!(field.resolve_look(Default::default(), true, ControlSize::Md, &scale).padding_x, 0.0);
    assert_eq!(area.resolve_look(Default::default(), true, ControlSize::Md, &scale).min_height, 73.0);
    assert_eq!(progress.resolve(true, ControlSize::Md).track_height, 11.0);
    assert_eq!(toolbar.resolve(true, ControlSize::Md, ToolbarVariant::Outline).gap, 0.0);
    assert_eq!(explicit_toolbar.resolve(true, ControlSize::Md, ToolbarVariant::Outline).gap, 17.0);
    assert_eq!(tooltip.resolve().max_width, 420.0);
    assert_eq!((card.resolve_style().padding, card.resolve_style().margin), (18.0, -18.0));
    assert_eq!(explicit_card.resolve_style().padding, 21.0);
    assert_eq!(
        (avatar.resolve_style().diameter, avatar.resolve_style().radius, avatar.resolve_style().font_size),
        (190.0, 95.0, 44.0)
    );
    assert_eq!(explicit_avatar.resolve_style().diameter, 33.0);
    assert_eq!(crate::popup_menu::floating_menu_look(&fork, PopupMenuVariant::Soft, Tone::Gray).min_width, 222.0);
    assert_eq!(
        context_menu_theme(&fork, ContextMenuVariant::Soft, Tone::Gray)
            .resolve(InteractionState::default())
            .target_min_width,
        240.0
    );
    assert_eq!(
        overlay_window_theme(&fork)
            .resolve(ControlSize::Sm, xgpui_luma::controls::overlay_window::OverlayWindowMode::Modal)
            .min_width,
        300.0
    );
    let box_ =
        crate::button_layout::button_box_with_stylesheet(&fork.common_stylesheet(), ButtonSize::Four, Radius::Full);
    assert_eq!((box_.height, box_.radius), (57.0, 28.5));
    assert_eq!(button_box_for(ButtonSize::Three, Radius::Full).height, 40.0);
    let theme = button_family_theme(&fork, ButtonVariant::Soft);
    assert_eq!(
        theme
            .resolve_look(ButtonFamilyRole::Icon, ControlSize::Md, Default::default(), &scale, 999.0)
            .unwrap()
            .height,
        32.0
    );
}

#[test]
fn toggle_size_four_is_not_collapsed_into_sdk_lg() {
    let look = Look::built_in();
    let theme =
        crate::button::button_family_theme_for_size(&look, ButtonVariant::Soft, Paint::accent(), ButtonSize::Four);
    let scale = StandardBoxScale::compute(ControlSize::Lg, &look.metrics(), 1.0);
    let resolved = theme
        .resolve_look(ButtonFamilyRole::Toggle { selected: false }, ControlSize::Lg, Default::default(), &scale, 999.0)
        .unwrap();
    assert_eq!((resolved.height, resolved.padding_x, resolved.typography.size), (48.0, 24.0, 18.0));
}

#[test]
fn sparse_configuration_keeps_composition_fallbacks_visible_and_zero_distinct() {
    let look = Look::built_in();
    let mut stylesheet = xgpui_luma::theme::stylesheet::CommonStylesheet::default();
    look.set_common_stylesheet(stylesheet.clone());
    assert!(crate::callout::resolve_geometry(&look).font_size > 0.0);
    assert!(crate::segmented::resolve_geometry(&look).height > 0.0);
    stylesheet.callout.geometry.font_size = Some(0.0);
    stylesheet.segmented.geometry.height = Some(0.0);
    look.set_common_stylesheet(stylesheet);
    assert_eq!(crate::callout::resolve_geometry(&look).font_size, 0.0);
    assert_eq!(crate::segmented::resolve_geometry(&look).height, 0.0);
}

#[test]
fn slide_panel_uses_captured_common_visual_defaults() {
    use xgpui_luma::controls::slide_panel::{SlidePanelSizeConfig, SlidePanelTopAnchor};
    let look = Look::built_in().fork();
    let mut sheet = look.common_stylesheet();
    sheet.slide_panel.geometry.side_width = Some(501.0);
    sheet.resizable_panels.geometry.grip_radius = Some(0.0);
    look.set_common_stylesheet(sheet);
    let panel = look.slide_panel_state(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 0.0, 900.0));
    look.set_common_stylesheet(xgpui_luma::theme::stylesheet::CommonStylesheet::default());
    assert_eq!(panel.geometry().side_width, Some(501.0));
    assert_eq!(panel.resize_handle_metrics().grip_radius, 0.0);
}
