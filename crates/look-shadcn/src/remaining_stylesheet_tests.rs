//! Cross-family regression coverage for selected configuration and common geometry.
use crate::ShadcnLook;
use crate::size::ShadcnSize;

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
