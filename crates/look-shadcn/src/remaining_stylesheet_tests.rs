//! Cross-family regression coverage for selected configuration and common geometry.
use xgpui_luma::theme::{InteractionState, StandardBoxScale};
use crate::ShadcnLook;
use crate::size::ShadcnSize;

#[test]
fn migrated_literal_families_keep_their_bundled_size_baselines() {
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    let look = crate::test_support::fallback_look();
    for (size, diameter, track, thumb, badge) in [
        (ShadcnSize::Sm, 48.0, 4.0, 12.0, 24.0),
        (ShadcnSize::Md, 64.0, 6.0, 16.0, 32.0),
        (ShadcnSize::Lg, 80.0, 8.0, 20.0, 40.0),
    ] {
        let progress = look.with_size(size).progress_theme().resolve(true);
        assert_eq!((progress.size, progress.track_height, progress.thumb_size), (diameter, track, thumb));
        assert_eq!(look.with_size(size).stepper_theme().resolve(true).step_badge_size, badge);
        let scrollbar = crate::controls::scrollbar::scrollbar_look(
            &look.mode_tokens(),
            InteractionState::default(),
            ScrollbarOrientation::Horizontal,
            size,
            ScrollbarStyle::Ghost,
        );
        assert_eq!(scrollbar.length, 260.0);
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selector_unset_geometry_retains_sdk_provenance_and_values() {
    let look = crate::test_support::fallback_look();
    for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
        let tokens = look.mode_tokens();
        let scale = StandardBoxScale::compute(&tokens.metrics_for(size), 1.0);
        let s = crate::inspect::ShadcnInspect::new(&look).inspect_selector_metrics(size);
        assert_eq!(s.trigger.padding_x.value_px, scale.padding_x);
        assert_eq!(s.trigger.radius.value_px, scale.radius);
        assert!(!format!("{:?}", s.trigger.padding_x.source).contains("button"));
        assert!(!format!("{:?}", s.trigger.height.source).contains("button"));
    }
}

#[test]
fn selector_panel_applies_common_overrides() {
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
    for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
        let scale = StandardBoxScale::compute(&tokens.metrics_for(size), 1.0);
        let resolved = look.with_size(size).selector_theme().resolve_look(
            SelectorTriggerStyle::Outline,
            Default::default(),
            &scale,
            false,
        );
        assert_eq!((resolved.items_panel.min_width, resolved.items_panel.radius), (0.0, 7.0));
        #[cfg(feature = "inspect")]
        {
            let inspected = crate::inspect::ShadcnInspect::new(&look).inspect_selector_metrics(size);
            assert_eq!(inspected.items_panel.min_width.value_px, resolved.items_panel.min_width);
            assert_eq!(inspected.items_panel.radius.value_px, resolved.items_panel.radius);
            assert_eq!(inspected.items_panel.item_height.value_px, resolved.items_panel.item_height);
        }
        assert_eq!(resolved.items_panel.item_height, if size == ShadcnSize::Sm { 0.0 } else { 42.0 });
    }
}

#[test]
fn pager_and_resize_handle_overrides_reach_paint_and_inspection() {
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
    let handle = look.resizable_panels_theme().resize_handle_metrics();
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
        let inspected = inspector.inspect_resizable_panels_metrics(crate::ShadcnSize::Md);
        assert_eq!(inspected.lane_px.value_px, handle.lane_px);
        assert_eq!(inspected.hit_target_px.value_px, handle.hit_target_px);
        assert_eq!(inspected.grip_main_axis_px.value_px, handle.grip_main_axis_px);
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
    let list = look.selector_items_panel_look(crate::ShadcnSize::Md);
    assert_eq!(
        (list.border_width, list.disabled_opacity, list.popup_offset_y, list.window_margin),
        (0.0, 0.0, 0.0, 0.0)
    );
    let selection = look.selection_panel_look(crate::ShadcnSize::Md);
    assert_eq!((selection.border_width, selection.disabled_opacity, selection.radius), (3.0, 0.2, 11.0));
}

#[test]
fn table_drag_preview_uses_selected_size_and_zero_strokes() {
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
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let md = look.table_theme().resolve_drag().geometry;
    assert_eq!((md.drag_offset, md.drag_width, md.drag_padding_x, md.drag_padding_y), (5.0, 180.0, 7.0, 3.0));
    assert_eq!((md.drag_border_width, md.drag_marker_width), (0.0, 0.0));
    assert_eq!(look.with_size(ShadcnSize::Lg).table_theme().resolve_drag().geometry.drag_width, 310.0);
}
