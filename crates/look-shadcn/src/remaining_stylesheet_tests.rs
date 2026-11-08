//! Cross-family regression coverage for selected configuration and common geometry.
use std::collections::HashMap;
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};
use crate::{ShadcnLook, ShadcnTextFieldStyle, StylesheetConfig};

const OVERRIDES: &str = r#"
[common.textfield.geometry]
min_height=47
padding_x=7
padding_y=3
gap=4
icon_size=19
font_size=17
line_height=25
[common.textarea.geometry]
min_height=71
padding_x=9
padding_y=6
font_size=18
line_height=29
[common.progress.geometry]
size=83
stroke_width=3.5
track_height=9
thumb_size=22
[common.stepper.geometry]
step_badge_size=43
track_thickness=4
[common.scrollbar.geometry]
length_h=331
length_v=221
thickness=15
track_thickness=5
thumb_thickness=10
min_thumb_length=34
[common.floating_menu.geometry]
padding=5
min_width=245
item_height=31
item_padding_x=8
item_gap=6
item_icon_size=17
font_size=16
line_height=22
submenu_offset_x=5
[common.popup_menu.geometry]
height=51
padding_x=11
padding_y=4
gap=7
icon_size=21
font_size=17
line_height=25
[common.selector.geometry]
height=49
padding_x=9
padding_y=5
gap=8
icon_size=23
font_size=18
line_height=26
[common.context_menu.geometry]
min_width=231
padding_x=12
padding_y=8
font_size=19
line_height=26
[common.card.geometry]
padding=23
section_gap=11
header_gap=7
body_gap=12
[common.badge.geometry]
min_height=29
padding_x=13
padding_y=5
gap=7
icon_size=17
font_size=15
line_height=21
[common.toolbar.geometry]
padding_x=10
padding_y=6
gap=12
[common.sidebar.geometry]
section_height=24
item_height=34
item_padding_x=11
item_gap=10
item_icon_size=19
[common.control_group.geometry]
padding_x=12
padding_y=7
gap=10
[common.table.geometry]
padding_x=4
padding_y=7
[common.overlay_window.geometry]
padding=19
min_width=450
max_width=570
estimated_height=300
font_size=18
line_height=26
[common.tooltip.geometry]
padding=11
padding_y=6
max_width=345
font_size=15
line_height=22
[common.pager.geometry]
button_size=39
button_min_width=41
control_height=43
padding_x=10
padding_y=12
gap=8
group_gap=19
font_size=14
line_height=20
"#;

fn custom_look() -> ShadcnLook {
    // Leave per-size geometry empty so general overrides exercise every family.
    let mut config = crate::embedded_stylesheet().clone();
    config.common = StylesheetConfig::parse(OVERRIDES).unwrap().common;
    ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap()
}

#[test]
fn input_families_resolve_independent_dimensions_and_typography() {
    let look = custom_look();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
            for style in [
                ShadcnTextFieldStyle::Outline,
                ShadcnTextFieldStyle::Input,
                ShadcnTextFieldStyle::Primary,
                ShadcnTextFieldStyle::Surface,
            ] {
                for bits in 0..32 {
                    let state = xgpui_luma::controls::textfield::TextFieldState {
                        hovered: bits & 1 != 0,
                        focused: bits & 2 != 0,
                        focus_visible: bits & 4 != 0,
                        invalid: bits & 8 != 0,
                        ..Default::default()
                    };
                    let enabled = bits & 16 == 0;
                    let field =
                        crate::controls::textfield::textfield_look(&tokens, mode, style, state, enabled, size, &scale);
                    let area = crate::controls::textarea::textarea_look(
                        &tokens,
                        mode,
                        style,
                        xgpui_luma::controls::textarea::TextAreaState {
                            hovered: state.hovered,
                            focused: state.focused,
                            focus_visible: state.focus_visible,
                            invalid: state.invalid,
                            ..Default::default()
                        },
                        enabled,
                        size,
                        &scale,
                    );
                    assert_eq!(
                        (field.min_height, field.padding_x, field.padding_y, field.icon_size, field.gap),
                        (47.0, 7.0, 3.0, 19.0, 4.0)
                    );
                    assert_eq!((field.typography.size, field.typography.line_height), (17.0, 25.0));
                    assert_eq!((area.min_height, area.padding_x, area.padding_y), (71.0, 9.0, 6.0));
                    assert_eq!((area.typography.size, area.typography.line_height), (18.0, 29.0));
                    #[cfg(feature = "inspect")]
                    {
                        let inspect = crate::inspect::ShadcnInspect::new(&look);
                        let f = inspect.inspect_textfield_metrics(size);
                        let a = inspect.inspect_textarea_metrics(size);
                        assert_eq!(f.min_height.value_px, field.min_height);
                        assert_eq!(f.padding_x.value_px, field.padding_x);
                        assert_eq!(a.min_height.value_px, area.min_height);
                        assert_eq!(a.padding_y.value_px, area.padding_y);
                        assert!(format!("{:?}", a.min_height.source).contains("common.textarea"));
                        let colors = inspect.inspect_textfield_color_palette(style, state, enabled);
                        assert_eq!(colors.background.value, field.background);
                        assert_eq!(colors.border.value, field.border);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "inspect")]
#[test]
fn remaining_common_geometry_reaches_paint_and_inspection() {
    use xgpui_luma::controls::{
        scrollbar::{ScrollbarOrientation, ScrollbarStyle},
        overlay_window::OverlayWindowMode,
        pager::PagerStyle,
        toolbar::ToolbarVariant,
        selector::SelectorTriggerStyle,
    };
    let look = custom_look();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        let inspect = crate::inspect::ShadcnInspect::new(&look);
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let progress = look.progress_theme().resolve(true, size);
            let p = inspect.inspect_progress_metrics_for_size(size);
            assert_eq!((progress.size, progress.track_height, progress.thumb_size), (83.0, 9.0, 22.0));
            assert_eq!(p.track_height.value_px, progress.track_height);
            assert!(format!("{:?}", p.size.source).contains("common.progress"));
            let stepper = look.stepper_theme().resolve(true, size);
            assert_eq!((stepper.step_badge_size, stepper.track_thickness), (43.0, 4.0));
            assert_eq!(inspect.inspect_stepper_metrics_for_size(size).step_badge_size.value_px, 43.0);
            for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                let scrollbar = crate::paint::scrollbar_look(
                    &tokens,
                    InteractionState::default(),
                    orientation,
                    size,
                    ScrollbarStyle::Ghost,
                );
                let s = inspect.inspect_scrollbar_metrics_for_size(orientation, ScrollbarStyle::Ghost, size);
                assert_eq!(
                    (scrollbar.thickness, scrollbar.thumb_thickness, scrollbar.min_thumb_length),
                    (15.0, 10.0, 34.0)
                );
                assert_eq!(s.length.value_px, scrollbar.length);
                assert!(format!("{:?}", s.length.source).contains("common.scrollbar"));
            }
            let menu = crate::paint::floating_menu_look(&tokens, mode, size);
            let m = inspect.inspect_floating_menu_metrics(size);
            assert_eq!((menu.min_width, menu.item_height, menu.item_icon_size), (245.0, 31.0, 17.0));
            assert_eq!((menu.item_typography.size, menu.item_typography.line_height), (16.0, 22.0));
            assert_eq!(m.item_icon_size.value_px, menu.item_icon_size);
            assert!(format!("{:?}", m.item_icon_size.source).contains("common.floating_menu"));
            let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
            let selector = crate::controls::selector::selector_look(
                &tokens,
                mode,
                SelectorTriggerStyle::Outline,
                InteractionState::default(),
                size,
                &scale,
                false,
            );
            let s = inspect.inspect_selector_metrics(size);
            assert_eq!(
                (selector.trigger_height, selector.trigger_padding_x, selector.trigger_icon_size),
                (49.0, 9.0, 23.0)
            );
            assert_eq!(s.trigger.padding_x.value_px, selector.trigger_padding_x);
            let popup_scale = crate::controls::popup_menu::popup_menu_trigger_scale(
                &tokens,
                mode,
                size,
                InteractionState::default(),
                1.0,
            );
            let popup = crate::controls::popup_menu::popup_menu_palette(
                &tokens,
                mode,
                xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline,
                xgpui_luma::controls::popup_menu::PopupMenuTriggerMetrics { size, ..Default::default() },
                InteractionState::default(),
            );
            assert_eq!((popup_scale.height, popup_scale.padding_x, popup_scale.icon_size), (51.0, 11.0, 21.0));
            assert_eq!((popup.trigger_typography.size, popup.trigger_typography.line_height), (17.0, 25.0));
            let p = inspect
                .inspect_popup_menu_metrics(xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline, size);
            assert_eq!(p.trigger.height.value_px, popup_scale.height);
            assert!(format!("{:?}", p.trigger.icon_size.source).contains("common.popup_menu"));
            let card = crate::controls::card::card_look(&look, size);
            assert_eq!((card.padding, card.section_gap, card.header_gap, card.body_gap), (23.0, 11.0, 7.0, 12.0));
            assert_eq!(inspect.inspect_card_metrics(size).padding.value_px, 23.0);
            let toolbar = look.toolbar_theme().resolve(true, size, ToolbarVariant::Outline);
            assert_eq!((toolbar.padding_x, toolbar.padding_y, toolbar.gap), (10.0, 6.0, 12.0));
            assert_eq!(inspect.inspect_toolbar_metrics(size).gap.value_px, 12.0);
            let sidebar = crate::paint::sidebar_item_look(&look, false, InteractionState::default(), size);
            assert_eq!((sidebar.height, sidebar.padding_x, sidebar.icon_size, sidebar.gap), (34.0, 11.0, 19.0, 10.0));
            assert_eq!(inspect.inspect_sidebar_metrics(size).item_height.value_px, 34.0);
            let overlay = look.overlay_window_theme().resolve(size, OverlayWindowMode::Modal);
            assert_eq!(
                (overlay.padding, overlay.min_width, overlay.max_width, overlay.estimated_height),
                (19.0, 450.0, 570.0, 300.0)
            );
            assert_eq!(
                inspect.inspect_overlay_window_metrics(size, OverlayWindowMode::Modal).max_width.value_px,
                570.0
            );
        }
        let context = crate::paint::context_menu_look(&tokens, mode, InteractionState::default());
        assert_eq!((context.target_min_width, context.target_padding_x), (231.0, 12.0));
        let group = crate::paint::control_group_list_look(&tokens, true);
        assert_eq!((group.padding_x, group.padding_y, group.gap), (12.0, 7.0, 10.0));
        let table = crate::paint::table_look(&tokens, true, false, ControlSize::Md);
        assert_eq!((table.padding_x, table.padding_y), (4.0, 7.0));
        let badge = crate::badge_look(&look, crate::BadgeVariant::Default, crate::ShadcnSize::Md);
        assert_eq!((badge.min_height, badge.padding_x, badge.gap, badge.icon_size), (29.0, 13.0, 7.0, 17.0));
        for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
            let pager = crate::paint::pager_look(&look, true, style);
            assert_eq!((pager.button_size, pager.button_min_width, pager.group_gap), (39.0, 41.0, 19.0));
            assert_eq!(inspect.inspect_pager_metrics(style).button_size.value_px, 39.0);
        }
        let tooltip = crate::tooltip_theme(&look).resolve();
        assert_eq!(
            (tooltip.padding, tooltip.padding_y, tooltip.max_width, tooltip.text_size),
            (11.0, 6.0, 345.0, 15.0)
        );
    }
}

#[test]
fn selected_configuration_survives_color_copies_token_edits_and_live_replacement() {
    let look = custom_look();
    let copy = look
        .with_color_overrides(&HashMap::from([(
            "primary".into(),
            xgpui_luma::color::ColorValue::srgb(0.0, 0.0, 0.0, 1.0),
        )]))
        .unwrap();
    let progress = copy.progress_theme();
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 83.0);
    copy.apply_token_overrides(&HashMap::from([("radius".into(), "10px".into())])).unwrap();
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 83.0);
    let stored_tokens = copy.mode_tokens();
    let independent = crate::test_support::fallback_look();
    copy.replace_theme(&independent);
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 64.0);
    assert_eq!(crate::paint::progress_look(&stored_tokens, true, ControlSize::Md).size, 83.0);
    assert_eq!(look.progress_theme().resolve(true, ControlSize::Md).size, 83.0);
}

#[test]
fn migrated_literal_families_keep_their_bundled_size_baselines() {
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    let look = crate::test_support::fallback_look();
    for (size, diameter, track, thumb, badge) in [
        (ControlSize::Sm, 48.0, 4.0, 12.0, 24.0),
        (ControlSize::Md, 64.0, 6.0, 16.0, 32.0),
        (ControlSize::Lg, 80.0, 8.0, 20.0, 40.0),
    ] {
        let progress = look.progress_theme().resolve(true, size);
        assert_eq!((progress.size, progress.track_height, progress.thumb_size), (diameter, track, thumb));
        assert_eq!(look.stepper_theme().resolve(true, size).step_badge_size, badge);
        let scrollbar = crate::paint::scrollbar_look(
            &look.mode_tokens(),
            InteractionState::default(),
            ScrollbarOrientation::Horizontal,
            size,
            ScrollbarStyle::Ghost,
        );
        assert_eq!(scrollbar.length, 260.0);
    }
}

#[test]
fn sparse_common_geometry_inherits_sdk_baseline_and_preserves_explicit_zero() {
    let config = StylesheetConfig::parse(
        r#"
[progress.metrics.md]
size=91
stroke_width=7
track_height=8
thumb_size=19
[common.progress.geometry]
track_height=0
[common.progress.sizes.md]
size=0
"#,
    )
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let progress = look.progress_theme().resolve(true, ControlSize::Md);
    assert_eq!(
        (progress.size, progress.stroke_width, progress.track_height, progress.thumb_size),
        (0.0, 6.0, 0.0, 16.0)
    );
    #[cfg(feature = "inspect")]
    {
        let p = crate::inspect::ShadcnInspect::new(&look).inspect_progress_metrics_for_size(ControlSize::Md);
        assert_eq!(p.size.value_px, 0.0);
        assert!(format!("{:?}", p.size.source).contains("common.progress.sizes.md.size"));
        assert_eq!(p.thumb_size.value_px, 16.0);
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selector_unset_geometry_retains_sdk_provenance_and_values() {
    let look = crate::test_support::fallback_look();
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        let tokens = look.mode_tokens();
        let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
        let s = crate::inspect::ShadcnInspect::new(&look).inspect_selector_metrics(size);
        assert_eq!(s.trigger.padding_x.value_px, scale.padding_x);
        assert_eq!(s.trigger.radius.value_px, scale.radius);
        assert!(!format!("{:?}", s.trigger.padding_x.source).contains("button"));
        assert!(!format!("{:?}", s.trigger.height.source).contains("button"));
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selected_color_rules_follow_the_same_configuration_as_geometry() {
    let mut config = crate::embedded_stylesheet().clone();
    for rule in &mut config.progress.color_rules {
        rule.track_color = "secondary".into();
        rule.progress_color = "secondary-foreground".into();
    }
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        for enabled in [false, true] {
            let paint = look.progress_theme().resolve(enabled, ControlSize::Md);
            let inspect = crate::inspect::ShadcnInspect::new(&look).inspect_progress_color_palette(enabled);
            assert_eq!(paint.track_color, tokens.catalog.color("secondary").unwrap());
            assert_eq!(paint.progress_color, tokens.catalog.color("secondary-foreground").unwrap());
            assert_eq!(inspect.track_color.value, paint.track_color);
            assert_eq!(inspect.progress_color.value, paint.progress_color);
        }
    }
}

#[test]
fn selector_panel_and_trigger_apply_independent_common_overrides() {
    use xgpui_luma::controls::selector::SelectorTriggerStyle;
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.selector.geometry]
radius=9
menu_offset_y=0
[common.selector_list.geometry]
min_width=0
item_height=42
radius=7
[common.selector_list.sizes.sm]
item_height=0
"#])
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let tokens = look.mode_tokens();
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
        let resolved =
            look.selector_theme()
                .resolve_look(SelectorTriggerStyle::Outline, Default::default(), size, &scale, false);
        assert_eq!((resolved.trigger_radius, resolved.menu_offset_y), (9.0, 0.0));
        assert_eq!((resolved.items_panel.min_width, resolved.items_panel.radius), (0.0, 7.0));
        #[cfg(feature = "inspect")]
        {
            let inspected = crate::inspect::ShadcnInspect::new(&look).inspect_selector_metrics(size);
            assert_eq!(inspected.trigger.radius.value_px, resolved.trigger_radius);
            assert_eq!(inspected.items_panel.min_width.value_px, resolved.items_panel.min_width);
            assert_eq!(inspected.items_panel.radius.value_px, resolved.items_panel.radius);
            assert_eq!(inspected.items_panel.item_height.value_px, resolved.items_panel.item_height);
        }
        assert_eq!(resolved.items_panel.item_height, if size == ControlSize::Sm { 0.0 } else { 42.0 });
    }
}

#[test]
fn pager_and_resize_handle_overrides_reach_paint_and_inspection() {
    use xgpui_luma::controls::{pager::PagerStyle, resizable_panels::ResizeHandleSize};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.pager.geometry]
radius=13
disabled_opacity=0
[common.pager.sizes.minimal]
button_size=0
[common.resizable_panels.geometry]
grip_radius=0
[common.resizable_panels.sizes.md]
lane_width=17
hit_target=23
grip_main_axis=47
[common.slide_panel.geometry]
side_width=507
edge_height=431
edge_max_viewport_ratio="math(2 / 3)"
"#])
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let pager = look.pager_theme().resolve(true, PagerStyle::Minimal);
    assert_eq!((pager.radius, pager.disabled_opacity, pager.button_size), (13.0, 0.0, 0.0));
    let handle = look.resizable_panels_theme().resize_handle_metrics(ResizeHandleSize::Md);
    assert_eq!(
        (handle.lane_px, handle.hit_target_px, handle.grip_main_axis_px, handle.grip_radius),
        (17.0, 23.0, 47.0, 0.0)
    );
    let panel = look.slide_panel_state(
        xgpui_luma::controls::slide_panel::SlidePanelTopAnchor::WindowEdge,
        xgpui_luma::controls::slide_panel::SlidePanelSizeConfig::new(0.0, 0.0, 900.0),
    );
    assert_eq!(panel.geometry().side_width, Some(507.0));
    #[cfg(feature = "inspect")]
    {
        let inspector = crate::inspect::ShadcnInspect::new(&look);
        let inspected = inspector.inspect_resizable_panels_metrics(ResizeHandleSize::Md);
        assert_eq!(inspected.lane_px.value_px, handle.lane_px);
        assert_eq!(inspected.hit_target_px.value_px, handle.hit_target_px);
        assert_eq!(inspected.grip_main_axis_px.value_px, handle.grip_main_axis_px);
        let pager_metrics = inspector.inspect_pager_metrics(PagerStyle::Minimal);
        assert_eq!(pager_metrics.radius.value_px, pager.radius);
        assert_eq!(pager_metrics.button_size.value_px, pager.button_size);
    }
}

#[test]
fn panel_and_group_chrome_use_selected_common_stylesheet() {
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.control_group.geometry]
padding_x=0
radius=7
[common.selector_list.geometry]
border_width=0
disabled_opacity=0
popup_offset_y=0
window_margin=0
[common.selection_panel.geometry]
border_width=3
disabled_opacity=0.2
radius=11
"#])
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let shell = look.control_group_theme().resolve_list(true);
    assert_eq!((shell.padding_x, shell.radius), (0.0, 7.0));
    let list = look.selector_items_panel_look(crate::ShadcnSize::Md);
    assert_eq!(
        (list.border_width, list.disabled_opacity, list.popup_offset_y, list.window_margin),
        (0.0, 0.0, 0.0, 0.0)
    );
    let selection = look.selection_panel_look(crate::ShadcnSize::Md);
    assert_eq!((selection.border_width, selection.disabled_opacity, selection.radius), (3.0, 0.2, 11.0));
    #[cfg(feature = "inspect")]
    {
        let inspected = crate::inspect::ShadcnInspect::new(&look).inspect_control_group_metrics(ControlSize::Md);
        assert_eq!(inspected.radius.value_px, shell.radius);
        assert_eq!(inspected.padding_x.value_px, shell.padding_x);
    }
}

#[test]
fn resize_chrome_uses_common_stylesheet_including_zero_width() {
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.resizable_panels.geometry]
border_width=0
divider_width=3
disabled_opacity=0.2
"#])
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let theme = look.resizable_panels_theme();
    let enabled = theme.resolve(InteractionState::default());
    assert_eq!((enabled.border_width, enabled.divider_width), (0.0, 3.0));
    let disabled = theme.resolve(InteractionState { disabled: true, ..Default::default() });
    assert_eq!(disabled.disabled_opacity, 0.2);
    let slide = crate::controls::slide_panel::slide_panel_panels_look(&look);
    assert_eq!((slide.border_width, slide.divider_width), (0.0, 3.0));
}

#[test]
fn table_strokes_and_header_padding_use_selected_stylesheet() {
    use xgpui_luma::controls::table::{DefaultTableTheme, TableTheme};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.table.geometry]
border_width=0
header_divider_width=3
header_padding_bottom_scale="math(1 / 2)"
column_resize_hit_width=24
column_resize_line_width=0
column_resize_vertical_extension=7
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultTableTheme::new(tokens).resolve_look(true, false, ControlSize::Md);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = crate::controls::table::table_look(&look.mode_tokens(), true, false, ControlSize::Md);
    for resolved in [sdk, themed] {
        assert_eq!(
            (
                resolved.column_resize_hit_width,
                resolved.column_resize_line_width,
                resolved.column_resize_vertical_extension
            ),
            (24.0, 0.0, 7.0)
        );
        assert_eq!(
            (resolved.border_width, resolved.header_divider_width, resolved.header_padding_bottom_scale),
            (0.0, 3.0, 0.5)
        );
    }
}

#[test]
fn table_drag_preview_uses_selected_size_and_zero_strokes() {
    use xgpui_luma::controls::table::{DefaultTableTheme, TableTheme};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.table.geometry]
drag_offset=5
drag_width=180
drag_padding_x=7
drag_padding_y=3
drag_border_width=0
drag_marker_width=0
[common.table.sizes.lg]
drag_width=310
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultTableTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = look.table_theme();
    for theme in [&sdk as &dyn TableTheme, themed.as_ref()] {
        let md = theme.resolve_drag(ControlSize::Md).geometry;
        assert_eq!((md.drag_offset, md.drag_width, md.drag_padding_x, md.drag_padding_y), (5.0, 180.0, 7.0, 3.0));
        assert_eq!((md.drag_border_width, md.drag_marker_width), (0.0, 0.0));
        assert_eq!(theme.resolve_drag(ControlSize::Lg).geometry.drag_width, 310.0);
    }
}

#[test]
fn table_row_chrome_uses_selected_stylesheet_and_size() {
    use xgpui_luma::controls::table::{DefaultTableTheme, TableTheme};
    use xgpui_luma::theme::ListRowScale;
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.table.geometry]
column_padding_x=0
column_muted_opacity=0
row_disabled_opacity=0
row_radius=7
[common.table.sizes.lg]
column_padding_x=19
column_muted_opacity=0.4
row_disabled_opacity=0.3
row_radius=11
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultTableTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = look.table_theme();
    for theme in [&sdk as &dyn TableTheme, themed.as_ref()] {
        for (size, opacity, radius) in [(ControlSize::Md, 0.0, 7.0), (ControlSize::Lg, 0.3, 11.0)] {
            let scale = ListRowScale::compute(size, &theme.metrics(), 1.0);
            let row =
                theme.resolve_row_look(false, InteractionState { disabled: true, ..Default::default() }, size, &scale);
            assert_eq!((row.disabled_opacity, row.radius), (opacity, radius));
            let expected_padding = if size == ControlSize::Lg { 19.0 } else { 0.0 };
            let expected_muted = if size == ControlSize::Lg { 0.4 } else { 0.0 };
            assert_eq!(row.column_padding_x, expected_padding);
            assert_eq!(row.column_muted_opacity, expected_muted);
            assert_eq!(theme.resolve_look(true, false, size).column_padding_x, expected_padding);
            assert_eq!(row.min_height, scale.min_height);
        }
    }
}

#[test]
fn pager_popup_chrome_uses_shared_stylesheet_and_presentation_overrides() {
    use xgpui_luma::controls::pager::{DefaultPagerTheme, PagerStyle, PagerTheme};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.pager.geometry]
border_width=0
page_size_label_gap=17
popup_offset_y=9
popup_gap=0
popup_padding=7
popup_item_padding_y=6
popup_item_radius_inset=0
[common.pager.sizes.minimal-edge]
popup_padding=11
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultPagerTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = look.pager_theme();
    for theme in [&sdk as &dyn PagerTheme, themed.as_ref()] {
        let numeric = theme.resolve(true, PagerStyle::Numeric);
        assert_eq!(
            (numeric.border_width, numeric.page_size_label_gap, numeric.popup_offset_y, numeric.popup_gap),
            (0.0, 17.0, 9.0, 0.0)
        );
        assert_eq!(
            (numeric.popup_padding, numeric.popup_item_padding_y, numeric.popup_item_radius_inset),
            (7.0, 6.0, 0.0)
        );
        assert_eq!(theme.resolve(true, PagerStyle::MinimalEdge).popup_padding, 11.0);
    }
}

#[test]
fn context_menu_chrome_uses_shared_stylesheet_with_zero_overrides() {
    use xgpui_luma::controls::context_menu::{DefaultContextMenuTheme, ContextMenuTheme};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.context_menu.geometry]
border_width=0
disabled_opacity=0
window_margin=19
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultContextMenuTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = look.context_menu_theme();
    for theme in [&sdk as &dyn ContextMenuTheme, themed.as_ref()] {
        let resolved = theme.resolve(InteractionState { disabled: true, ..Default::default() });
        assert_eq!((resolved.border_width, resolved.disabled_opacity, resolved.window_margin), (0.0, 0.0, 19.0));
    }
}

#[test]
fn overlay_shell_chrome_uses_shared_stylesheet_and_size() {
    use xgpui_luma::controls::overlay_window::{DefaultOverlayWindowTheme, OverlayWindowTheme, OverlayWindowMode};
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.overlay_window.geometry]
border_width=0
radius=0
[common.overlay_window.sizes.lg]
border_width=3
radius=17
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk = DefaultOverlayWindowTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let themed = look.overlay_window_theme();
    for theme in [&sdk as &dyn OverlayWindowTheme, themed.as_ref()] {
        for mode in [OverlayWindowMode::Modal, OverlayWindowMode::Modeless] {
            let md = theme.resolve(ControlSize::Md, mode);
            let lg = theme.resolve(ControlSize::Lg, mode);
            assert_eq!((md.border_width, md.radius), (0.0, 0.0));
            assert_eq!((lg.border_width, lg.radius), (3.0, 17.0));
        }
    }
    #[cfg(feature = "inspect")]
    assert_eq!(
        crate::inspect::ShadcnInspect::new(&look)
            .inspect_overlay_window_metrics(ControlSize::Lg, OverlayWindowMode::Modal)
            .radius
            .value_px,
        17.0
    );
}

#[test]
fn remaining_control_chrome_inherits_shared_overrides() {
    use xgpui_luma::controls::{
        checkbox::{DefaultCheckboxTheme, CheckboxTheme},
        radio_button::{DefaultRadioButtonTheme, RadioButtonTheme},
        switch::{DefaultSwitchTheme, SwitchTheme},
        toolbar::{DefaultToolbarTheme, ToolbarTheme, ToolbarVariant},
        stepper::{DefaultStepperTheme, StepperTheme},
        split_view::{DefaultSplitViewTheme, SplitViewTheme},
        dock_splitter::{DefaultDockSplitterTheme, DockSplitterTheme},
    };
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.checkbox.geometry]
disabled_opacity=0
[common.radio.geometry]
disabled_opacity=0.3
[common.switch.geometry]
disabled_opacity=0.4
[common.toolbar.geometry]
border_width=0
separator_width=3
[common.dock_splitter.geometry]
thumb_width=9
thumb_rest_opacity=0
[common.stepper.geometry]
content_gap=0
glyph_scale=0.7
[common.split_view.geometry]
hitbox_width=32
cue_width=0
[common.accordion.geometry]
disabled_opacity=0
drag_handle_padding=9
[common.sidebar.geometry]
container_gap=0
subtitle_opacity=0.2
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk_checkbox = DefaultCheckboxTheme::new(tokens.clone());
    let sdk_radio = DefaultRadioButtonTheme::new(tokens.clone());
    let sdk_switch = DefaultSwitchTheme::new(tokens.clone());
    let sdk_toolbar = DefaultToolbarTheme::new(tokens.clone());
    let sdk_dock = DefaultDockSplitterTheme::new(tokens.clone());
    let sdk_stepper = DefaultStepperTheme::new(tokens.clone());
    let sdk_split = DefaultSplitViewTheme::new(tokens.clone());
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    for scale in [sdk_checkbox.scale(ControlSize::Md, 2.0), look.checkbox_theme().scale(ControlSize::Md, 2.0)] {
        assert_eq!(scale.disabled_opacity, 0.0);
    }
    for scale in [sdk_radio.scale(ControlSize::Md, 2.0), look.radio_button_theme().scale(ControlSize::Md, 2.0)] {
        assert_eq!(scale.disabled_opacity, 0.3);
    }
    for scale in [sdk_switch.scale(ControlSize::Md, 2.0), look.switch_theme().scale(ControlSize::Md, 2.0)] {
        assert_eq!(scale.disabled_opacity, 0.4);
    }
    for resolved in [
        sdk_toolbar.resolve(true, ControlSize::Md, ToolbarVariant::Outline),
        look.toolbar_theme().resolve(true, ControlSize::Md, ToolbarVariant::Outline),
    ] {
        assert_eq!((resolved.border_width, resolved.separator_width), (0.0, 3.0));
    }
    for resolved in [sdk_dock.resolve(true), look.dock_splitter_theme().resolve(true)] {
        assert_eq!((resolved.geometry.thumb_width, resolved.geometry.thumb_rest_opacity), (9.0, 0.0));
    }
    for resolved in [sdk_stepper.resolve(true, ControlSize::Md), look.stepper_theme().resolve(true, ControlSize::Md)] {
        assert_eq!((resolved.geometry.content_gap, resolved.geometry.glyph_scale), (0.0, 0.7));
    }
    for resolved in [sdk_split.resolve(false, true), look.split_view_theme().resolve(false, true)] {
        assert_eq!((resolved.geometry.hitbox_width, resolved.geometry.cue_width), (32.0, 0.0));
    }
}

#[test]
fn range_and_tooltip_decoration_use_selected_common_stylesheet() {
    use xgpui_luma::controls::{
        slider::{DefaultSliderTheme, SliderTheme},
        progress::{DefaultProgressTheme, ProgressTheme},
    };
    let mut config = crate::embedded_stylesheet().clone();
    config.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[r#"
[common.slider.geometry]
dial_size=160
track_radius=52
disabled_opacity=0.3
[common.progress.geometry]
indeterminate_fraction=0
shimmer_soft_opacity=0
[common.tooltip.geometry]
target_gap=17
window_margin=0
pointer_size=9
shortcut_opacity=0
"#])
    .unwrap();
    let tokens = config
        .common
        .theme_tokens(ThemeMode::Light, xgpui_luma::color::GamutMapping::CssLocalMinde)
        .unwrap();
    let sdk_slider = DefaultSliderTheme::new(tokens.clone());
    let sdk_progress = DefaultProgressTheme::new(tokens);
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    for resolved in [
        sdk_slider.resolve(ControlSize::Md, None, Default::default()),
        look.slider_theme().resolve(ControlSize::Md, None, Default::default()),
    ] {
        assert_eq!(
            (
                resolved.decoration.dial_size,
                resolved.decoration.track_radius,
                resolved.decoration.disabled_opacity
            ),
            (160.0, 52.0, 0.3)
        );
    }
    for resolved in [sdk_progress.resolve(true, ControlSize::Md), look.progress_theme().resolve(true, ControlSize::Md)]
    {
        assert_eq!((resolved.geometry.indeterminate_fraction, resolved.geometry.shimmer_soft_opacity), (0.0, 0.0));
    }
    let tooltip = crate::tooltip_theme(&look).resolve();
    assert_eq!(
        (
            tooltip.placement.gap,
            tooltip.placement.margin,
            tooltip.placement.pointer_size,
            tooltip.shortcut_opacity
        ),
        (17.0, 0.0, 9.0, 0.0)
    );
}
