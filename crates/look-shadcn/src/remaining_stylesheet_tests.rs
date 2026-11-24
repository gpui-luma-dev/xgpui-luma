//! Cross-family regression coverage for selected configuration and common geometry.
use crate::ShadcnLook;

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
