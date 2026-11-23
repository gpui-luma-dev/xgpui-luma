//! Cross-family regression coverage for selected configuration and common geometry.
use crate::ShadcnLook;

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
"#])
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let list = look.selector_items_panel_look(crate::ShadcnSize::Md);
    assert_eq!(
        (list.border_width, list.disabled_opacity, list.popup_offset_y, list.window_margin),
        (0.0, 0.0, 0.0, 0.0)
    );
    let selection = look.selection_panel_look(crate::ShadcnSize::Md);
    // The selection panel follows the items panel; it has no geometry of its own.
    assert_eq!((selection.border_width, selection.disabled_opacity), (0.0, 0.0));
}

/// The drag preview's geometry is the table's `.lss` (#287).
#[test]
fn table_drag_preview_geometry_comes_from_the_table_lss() {
    use xgpui_luma::style::lss::Value;
    let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
    let theme = look.style_theme();
    let resolved = theme.resolve(&theme.key_named("table", &[], &[]).expect("key"));
    let number = |name| resolved.root().value(name).and_then(Value::as_number).expect(name);
    let drag = look.table_theme().resolve_drag().geometry;
    assert_eq!(
        (drag.drag_offset, drag.drag_width, drag.drag_padding_x, drag.drag_padding_y),
        (number("drag_offset"), number("drag_width"), number("drag_padding_x"), number("drag_padding_y"))
    );
    assert_eq!(
        (drag.drag_border_width, drag.drag_marker_width),
        (number("drag_border_width"), number("drag_marker_width"))
    );
}
