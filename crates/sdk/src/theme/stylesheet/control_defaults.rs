use super::CommonStylesheet;
use crate::{
    color::GamutMapping,
    controls::{
        checkbox::{CheckboxScale, CheckboxTheme, DefaultCheckboxTheme},
        context_menu::{ContextMenuTheme, DefaultContextMenuTheme},
        floating_menu::{DefaultFloatingMenuTheme, FloatingMenuTheme},
        overlay_window::{DefaultOverlayWindowTheme, OverlayWindowMode, OverlayWindowTheme},
        radio_button::RadioScale,
        selection_panel::default_selection_panel_look,
        slider::{DefaultSliderTheme, SliderTheme},
        switch::SwitchScale,
        textfield::{DefaultTextFieldTheme, TextFieldTheme},
    },
    theme::{MetricTokens, StandardBoxScale, ThemeMode, snap_to_pixel},
};

#[test]
fn choice_baseline_keeps_stock_metrics_at_multiple_display_scales() {
    let metrics = MetricTokens::default();
    let (indicator_scale, radius, inset, width, height, thumb) = (0.5, 4.0, 3.0, 40.0, 18.0, 14.0);
    {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let checkbox = CheckboxScale::compute(&metrics, scale);
            let radio = RadioScale::compute(&metrics, scale);
            let switch = SwitchScale::compute(&metrics, scale);
            let indicator = snap_to_pixel(metrics.control_height() * indicator_scale, scale);
            assert_eq!(checkbox.indicator_size, indicator);
            assert_eq!(checkbox.indicator_radius, snap_to_pixel(radius, scale));
            assert_eq!(
                checkbox.glyph_size,
                snap_to_pixel((indicator - snap_to_pixel(inset, scale) * 2.0).max(0.0), scale)
            );
            assert_eq!(radio.indicator_size, indicator);
            assert_eq!(radio.dot_size, snap_to_pixel(metrics.control_height() * 0.24, scale));
            assert_eq!(switch.track_width, snap_to_pixel(width, scale));
            assert_eq!(switch.track_height, snap_to_pixel(height, scale));
            assert_eq!(switch.thumb_size, snap_to_pixel(thumb, scale));
        }
    }
}

#[test]
fn sparse_sheet_prepares_a_complete_sdk_theme_and_preserves_zero() {
    let sheet = CommonStylesheet::parse("[common.checkbox.geometry]\nindicator_size=0\nglyph_size=0").unwrap();
    let mut tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    tokens.metrics.control.control_height = 54.0;
    let theme = DefaultCheckboxTheme::new(tokens.clone());
    let scale = theme.scale(1.0);
    assert_eq!((scale.indicator_size, scale.glyph_size, scale.height), (0.0, 0.0, 54.0));
    assert_eq!(tokens.stylesheet.progress.geometry.size, Some(64.0));
    assert_eq!(RadioScale::compute(&tokens.metrics, 1.0).indicator_size, 27.0);
}

#[test]
fn captured_input_geometry_overrides_shared_metrics_without_affecting_palette() {
    let sheet =
        CommonStylesheet::parse("[common.textfield.geometry]\nmin_height=47\npadding_x=0\nfont_size=19").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let scale = StandardBoxScale::compute(&tokens.metrics, 1.0);
    let expected_background = tokens.palette.form.input.background;
    let theme = DefaultTextFieldTheme::new(tokens);
    let look = theme.resolve_look(Default::default(), true, &scale);
    assert_eq!((look.min_height, look.padding_x, look.typography.size), (47.0, 0.0, 19.0));
    assert_eq!(look.background, expected_background);
    assert_eq!(look.padding_y, scale.padding_y);
}

#[test]
fn menu_and_selection_panel_share_a_baseline_with_independent_final_overrides() {
    let sheet = CommonStylesheet::parse("[common.floating_menu.geometry]\nmin_width=207\nitem_height_scale=2\nitem_padding_scale=0\ndisabled_opacity=0\nseparator_thickness=0\n[common.selection_panel.geometry]\nmin_width=0\n[common.context_menu.geometry]\nmin_width=99").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let menu = DefaultFloatingMenuTheme::new(tokens.clone()).resolve();
    assert_eq!((menu.min_width, menu.item_height, menu.item_padding_x), (207.0, 72.0, 0.0));
    assert_eq!((menu.disabled_opacity, menu.separator_thickness), (0.0, 0.0));
    let panel = default_selection_panel_look(&tokens);
    assert_eq!((panel.min_width, panel.item_height), (0.0, 72.0));
    assert_eq!(panel.item_typography.size, tokens.typography.text.label.size);
    let context = DefaultContextMenuTheme::new(tokens).resolve(Default::default());
    assert_eq!((context.target_min_width, context.floating_menu.min_width), (99.0, 207.0));
}

#[test]
fn slider_thumb_uses_its_own_recipe() {
    let sheet = CommonStylesheet::parse("[common.slider.geometry]\nthumb_scale=0.25").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let theme = DefaultSliderTheme::new(tokens);
    let look = theme.resolve(Default::default());
    assert_eq!((look.width, look.height, look.thumb_size), (260.0, 32.0, 8.0));
}

#[test]
fn overlay_windows_use_shared_shadow_data_and_explicit_backdrop_opacity() {
    let sheet = CommonStylesheet::parse("[common.overlay_window.geometry]\nbackdrop_opacity=0").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Dark, GamutMapping::CssLocalMinde).unwrap();
    let expected_shadow = tokens.elevation.overlay_window.to_box_shadows();
    let theme = DefaultOverlayWindowTheme::new(tokens);
    let modal = theme.resolve(OverlayWindowMode::Modal);
    let floating = theme.resolve(OverlayWindowMode::Modeless);
    assert_eq!((modal.min_width, modal.max_width, modal.estimated_height), (400.0, 520.0, 228.0));
    assert_eq!(modal.backdrop_background.a, 0.0);
    assert_eq!(floating.shadow, expected_shadow);
    for family in ["floating_menu", "overlay_window", "resizable_panels"] {
        let field = if family == "overlay_window" {
            "backdrop_opacity"
        } else {
            "disabled_opacity"
        };
        for value in ["-1", "1.1", "nan", "inf"] {
            assert!(CommonStylesheet::parse(&format!("[common.{family}.geometry]\n{field}={value}")).is_err());
        }
    }
}

#[test]
fn explicit_false_replaces_an_inset_shadow_and_reaches_rendering() {
    let layer = "color = 'hsla(0, 0%, 0%, 0.2)', offset_x=0, offset_y=1, blur=2, spread=0";
    let first = format!("[common.tokens.elevation.light.menu]\nlayers=[{{ {layer}, inset=true }}]");
    let last = format!("[common.tokens.elevation.light.menu]\nlayers=[{{ {layer}, inset=false }}]");
    let initial = CommonStylesheet::resolve_layers(&[&first]).unwrap();
    assert!(initial.tokens.elevation.light.menu.layers[0].inset);
    let resolved = CommonStylesheet::resolve_layers(&[&first, &last]).unwrap();
    assert!(!resolved.tokens.elevation.light.menu.layers[0].inset);
    let tokens = resolved.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    assert!(!tokens.elevation.menu.to_box_shadows()[0].inset);
}

#[test]
fn tree_and_accordion_themes_resolve_captured_common_geometry() {
    use crate::controls::{
        accordion::{AccordionTheme, DefaultAccordionTheme},
        tree_view::{DefaultTreeViewTheme, TreeViewTheme},
    };
    let sheet = CommonStylesheet::parse("[common.tree_view.geometry]\nrow_height_scale=0.5\nindentation_width=0\nchevron_size=0\n[common.accordion.geometry]\ntrigger_height=0\ncontent_padding_scale=2\nicon_size=0").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let tree = DefaultTreeViewTheme::new(tokens.clone()).resolve_scale(1.0);
    assert_eq!((tree.row_height, tree.indentation_width, tree.chevron_size), (18.0, 0.0, 0.0));
    assert_eq!(tree.icon_size, tokens.metrics.icon_size());
    let accordion = DefaultAccordionTheme::new(tokens).resolve_scale(1.0);
    assert_eq!((accordion.trigger_height, accordion.content_padding_y, accordion.icon_size), (0.0, 16.0, 0.0));
    assert_eq!(accordion.chevron_size, 14.0);
}

#[test]
fn math_values_resolve_through_tokens_layers_and_geometry_validation() {
    let sheet = CommonStylesheet::resolve_layers(&[
        "[common.switch.geometry]\ntrack_width_scale = 'math(32 / 28)'\n[common.tokens.metrics.control]\ncontrol_height = 'math(28 * 2)'\n[common.overlay_window.geometry]\nbackdrop_opacity = 'math(1 / 4)'",
        "[common.switch.geometry]\nthumb_scale = 'math(0 / 28)'",
    ]).unwrap();
    assert_eq!(sheet.tokens.metrics.control.control_height, 56.0);
    assert_eq!(sheet.overlay_window.geometry.backdrop_opacity, Some(0.25));
    let scale = SwitchScale::compute_with_stylesheet(&sheet.tokens.metrics, 1.0, &sheet);
    assert_eq!(scale.track_width, 64.0);
    assert_eq!(scale.thumb_size, 0.0);
    // Direct serde ingress is also used by look-owned stylesheet documents.
    let direct: CommonStylesheet = toml::from_str("[switch.geometry]\ngap = 'math(2 * 3)'").unwrap();
    assert_eq!(direct.switch.geometry.gap, Some(6.0));
    for source in [
        "[common.switch.geometry]\ntrack_width_scale = 'math(-1 / 2)'",
        "[common.overlay_window.geometry]\nbackdrop_opacity = 'math(3 / 2)'",
        "[common.tokens.metrics.control]\ncontrol_height = 'math(-28 * 2)'",
        "[common.switch.geometry]\nunknown = 'math(1 + 2)'",
    ] {
        assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
    }
}

#[test]
fn table_and_selector_factories_resolve_snapshots_and_preserve_zero() {
    use crate::controls::{
        table::{DefaultTableTheme, TableTheme},
        selector::{DefaultSelectorTheme, SelectorTheme, SelectorTriggerStyle},
        selector_list::default_selector_items_panel_look,
    };
    use crate::theme::{InteractionState, ListRowScale};
    let sheet = CommonStylesheet::resolve_layers(&[r#"
[common.table.geometry]
padding_x = 0
padding_y_scale = "math(3 / 4)"
row_hover_opacity = 0
row_radius = 7
[common.selector.geometry]
height = 53
padding_x = 0
radius = 9
font_size = 19
menu_offset_scale = "math(3 / 2)"
[common.floating_menu.geometry]
min_width = 250
item_height = 35
[common.selector_list.geometry]
min_width = 0
item_height = 43
font_size = 18
radius = 6
"#])
    .unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let table = DefaultTableTheme::new(tokens.clone());
    let surface = table.resolve_look(true, false);
    assert_eq!(surface.padding_x, 0.0);
    assert_eq!(surface.padding_y, tokens.metrics.padding_y() * 0.75);
    assert_eq!(table.resolve_row(false, InteractionState { hovered: true, ..Default::default() }).background.a, 0.0);
    let row = table.resolve_row_look(false, Default::default(), &ListRowScale::compute(&tokens.metrics, 1.0));
    assert_eq!(row.radius, 7.0);
    let selector = DefaultSelectorTheme::new(tokens.clone());
    let scale = StandardBoxScale::compute(&tokens.metrics, 1.0);
    let look = selector.resolve_look(SelectorTriggerStyle::Outline, Default::default(), &scale, false);
    assert_eq!(
        (look.trigger_height, look.trigger_padding_x, look.trigger_radius, look.trigger_typography.size),
        (53.0, 0.0, 9.0, 19.0)
    );
    assert_eq!(look.menu_offset_y, scale.gap * 1.5);
    let panel = default_selector_items_panel_look(&tokens);
    assert_eq!((panel.min_width, panel.item_typography.size, panel.radius), (0.0, 18.0, 6.0));
    assert_eq!(panel.item_height, 43.0);
    let explicit =
        CommonStylesheet::resolve_layers(&["[common.selector.geometry]\nmenu_offset_y=0\nmenu_offset_scale=3"])
            .unwrap();
    let selector =
        DefaultSelectorTheme::new(explicit.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap());
    let scale = StandardBoxScale::compute(&tokens.metrics, 1.0);
    assert_eq!(
        selector
            .resolve_look(SelectorTriggerStyle::Outline, Default::default(), &scale, false)
            .menu_offset_y,
        0.0
    );
    for value in ["-1", "1.1", "nan"] {
        assert!(CommonStylesheet::parse(&format!("[common.table.geometry]\nrow_hover_opacity={value}")).is_err());
    }
}

#[test]
fn selector_panel_baseline_retains_stock_metrics_and_inherits_menu_overrides() {
    use crate::controls::selector_list::default_selector_items_panel_look;
    let tokens = crate::theme::ThemeTokens::default();
    let panel = default_selector_items_panel_look(&tokens);
    assert_eq!(panel.min_width, 180.0);
    assert_eq!(panel.padding, tokens.metrics.padding_y() * 0.5);
    assert_eq!(panel.item_height, tokens.metrics.control_height() * 0.9);
    assert_eq!(panel.item_padding_x, tokens.metrics.padding_x() * 0.75);
    let sheet =
        CommonStylesheet::resolve_layers(&["[common.floating_menu.geometry]\nmin_width=273\nitem_height=31"]).unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Dark, GamutMapping::CssLocalMinde).unwrap();
    let panel = default_selector_items_panel_look(&tokens);
    assert_eq!((panel.min_width, panel.item_height), (273.0, 31.0));
}

#[test]
fn pager_recipes_apply_style_overrides_and_validate_opacity() {
    use crate::controls::pager::{DefaultPagerTheme, PagerStyle, PagerTheme};
    let sheet = CommonStylesheet::resolve_layers(&[r#"
[common.pager.geometry]
padding_x_inset_scale=0
padding_y_gap_scale="math(3 / 2)"
button_extra_width_scale=2
radius=9
disabled_opacity=0
font_size=17
line_height=24
[common.pager.sizes.minimal]
button_min_width=0
"#])
    .unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let theme = DefaultPagerTheme::new(tokens.clone());
    for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
        let look = theme.resolve(true, style);
        assert_eq!(look.padding_x, tokens.metrics.padding_x());
        assert_eq!(look.padding_y, tokens.metrics.padding_y() + tokens.metrics.gap() * 1.5);
        assert_eq!(
            (look.radius, look.disabled_opacity, look.typography.size, look.typography.line_height),
            (9.0, 0.0, 17.0, 24.0)
        );
        assert_eq!(
            look.button_min_width,
            if style == PagerStyle::Minimal {
                0.0
            } else {
                tokens.metrics.control_height() + tokens.metrics.spacing.s1 * 2.0
            }
        );
    }
    for value in ["-1", "1.1", "nan"] {
        assert!(CommonStylesheet::parse(&format!("[common.pager.geometry]\ndisabled_opacity={value}")).is_err());
        assert!(
            CommonStylesheet::parse(&format!("[common.slide_panel.geometry]\nedge_max_viewport_ratio={value}"))
                .is_err()
        );
    }
}

#[test]
fn slide_panel_and_resize_presets_capture_stylesheet_without_overriding_explicit_sizes() {
    use crate::controls::{
        slide_panel::{SlidePanelState, SlidePanelTopAnchor, SlidePanelSizeConfig},
        resizable_panels::{ResizeHandleSize, DefaultResizablePanelsTheme, ResizablePanelsTheme},
    };
    let mut sheet = CommonStylesheet::resolve_layers(&[r#"
[common.slide_panel.geometry]
side_width=490
edge_height=530
edge_max_viewport_ratio="math(3 / 5)"
[common.resizable_panels.geometry]
grip_radius=0
[common.resizable_panels.sizes.sm]
lane_width=13
hit_target=21
grip_main_axis=45
"#])
    .unwrap();
    let state = SlidePanelState::new(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 0.0, 900.0))
        .stylesheet(&sheet);
    let explicit = SlidePanelState::new(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(270.0, 0.0, 900.0))
        .stylesheet(&sheet);
    sheet.slide_panel.geometry.side_width = Some(700.0);
    assert_eq!(state.main_axis_size(state.geometry().side_width.unwrap()), 490.0);
    assert_eq!(explicit.main_axis_size(explicit.geometry().side_width.unwrap()), 270.0);
    assert_eq!((state.geometry().edge_height, state.geometry().edge_max_viewport_ratio), (Some(530.0), Some(0.6)));
    let handle = state.resize_handle_metrics();
    assert_eq!(
        (handle.lane_px, handle.hit_target_px, handle.grip_main_axis_px, handle.grip_radius),
        (13.0, 21.0, 45.0, 0.0)
    );
    let theme =
        DefaultResizablePanelsTheme::new(sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap());
    assert_eq!(theme.resize_handle_metrics(ResizeHandleSize::Sm), handle);
    let baseline = SlidePanelState::new(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 0.0, 900.0));
    assert_eq!(
        (
            baseline.geometry().side_width,
            baseline.geometry().edge_height,
            baseline.geometry().edge_max_viewport_ratio
        ),
        (Some(360.0), Some(420.0), Some(0.72))
    );
}

#[test]
fn sdk_toolbar_factory_uses_common_geometry_for_both_variants() {
    use crate::controls::toolbar::{DefaultToolbarTheme, ToolbarTheme, ToolbarVariant};
    let sheet =
        CommonStylesheet::resolve_layers(&["[common.toolbar.geometry]\npadding_x=0\npadding_y=7\ngap=13"]).unwrap();
    let theme = DefaultToolbarTheme::new(sheet.theme_tokens(ThemeMode::Dark, GamutMapping::CssLocalMinde).unwrap());
    for variant in [ToolbarVariant::Outline, ToolbarVariant::Ghost] {
        let look = theme.resolve(true, variant);
        assert_eq!((look.padding_x, look.padding_y, look.gap), (0.0, 7.0, 13.0));
    }
}

#[test]
fn selector_panel_chrome_inherits_tokens_and_placement_before_independent_overrides() {
    use crate::controls::{selector_list::default_selector_items_panel_look, selection_panel::default_selection_panel_look};
    let sheet = CommonStylesheet::resolve_layers(&[r#"
[common.tokens.metrics.border_width]
default=3
[common.popover.geometry]
window_margin=13
[common.popover.sizes.panel]
offset_y=9
[common.floating_menu.geometry]
disabled_opacity=0.3
[common.selection_panel.geometry]
border_width=0
disabled_opacity=0
radius=9
"#])
    .unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let look = default_selector_items_panel_look(&tokens);
    assert_eq!(
        (look.border_width, look.disabled_opacity, look.popup_offset_y, look.window_margin),
        (3.0, 0.3, 9.0, 13.0)
    );
    let selection = default_selection_panel_look(&tokens);
    assert_eq!((selection.border_width, selection.disabled_opacity, selection.radius), (0.0, 0.0, 9.0));
    for family in ["selector_list", "selection_panel"] {
        for value in ["-1", "1.1", "nan"] {
            assert!(CommonStylesheet::parse(&format!("[common.{family}.geometry]\ndisabled_opacity={value}")).is_err());
        }
    }
}

#[test]
fn group_shell_factory_and_final_geometry_preserve_explicit_zero() {
    use crate::controls::control_group::{ControlGroupTheme, DefaultControlGroupTheme};
    let sheet =
        CommonStylesheet::resolve_layers(&["[common.control_group.geometry]\npadding_x=0\nradius=0\ngap=11\nvertical_gap=0\nborder_width=0\ndisabled_opacity=0"]).unwrap();
    let theme =
        DefaultControlGroupTheme::new(sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap());
    let shell = theme.resolve_list(true);
    assert_eq!((shell.padding_x, shell.radius, shell.gap), (0.0, 0.0, 11.0));
    assert_eq!((shell.vertical_gap, shell.border_width, shell.disabled_opacity), (0.0, 0.0, 0.0));
    // Sparse final overrides preserve a supplied theme's other dimensions.
    let final_sheet = CommonStylesheet::parse("[common.control_group.geometry]\ngap=0").unwrap();
    let final_shell = shell.with_stylesheet(&final_sheet.control_group);
    assert_eq!(
        (final_shell.padding_x, final_shell.padding_y, final_shell.radius, final_shell.gap),
        (shell.padding_x, shell.padding_y, shell.radius, 0.0)
    );
}

#[test]
fn shared_helper_chrome_and_drag_elevation_use_selected_snapshots() {
    use crate::controls::{
        textarea::{DefaultTextAreaTheme, TextAreaTheme},
        tooltip::TooltipLook,
        tree_view::{DefaultTreeViewTheme, TreeViewTheme},
    };
    let sheet = CommonStylesheet::resolve_layers(&["[common.shared.geometry]
compact_textfield_height=31
composition_underline_width=0
textarea_resize_icon_size=17
label_baseline=0
[common.tokens.elevation.light.drag_preview]
layers=[]
[common.tooltip.geometry]
window_margin=0"])
    .unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let scale = StandardBoxScale::compute(&tokens.metrics, 1.0);
    let field = DefaultTextFieldTheme::new(tokens.clone()).resolve_look(Default::default(), true, &scale);
    assert_eq!(field.helper_geometry.compact_textfield_height, 31.0);
    assert_eq!(field.helper_geometry.composition_underline_width, 0.0);
    let area = DefaultTextAreaTheme::new(tokens.clone()).resolve_look(Default::default(), true, &scale);
    assert_eq!(area.helper_geometry.textarea_resize_icon_size, 17.0);
    assert_eq!(CheckboxScale::compute_with_stylesheet(&tokens.metrics, 2.0, &sheet).label_baseline_shift, 0.0);
    assert_eq!(TooltipLook::from_tokens(&tokens, None).placement.margin, 0.0);
    assert!(DefaultTreeViewTheme::new(tokens).drag_preview_shadow().is_empty());
}
