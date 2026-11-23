use super::CommonStylesheet;
use crate::{color::GamutMapping, theme::ThemeMode};

#[test]
fn sparse_sheet_prepares_a_complete_sdk_theme_and_preserves_zero() {
    let sheet = CommonStylesheet::parse("[common.progress.geometry]\nstroke_width=0").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    assert_eq!(tokens.stylesheet.progress.geometry.size, Some(64.0));
    assert_eq!(tokens.stylesheet.progress.geometry.stroke_width, Some(0.0));
}

#[test]
fn opacity_fields_reject_values_outside_zero_to_one() {
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
fn math_values_resolve_through_layers_and_geometry_validation() {
    let sheet = CommonStylesheet::resolve_layers(&[
        "[common.slider.geometry]\nwidth = 'math(32 * 2)'\n[common.overlay_window.geometry]\nbackdrop_opacity = 'math(1 / 4)'",
        "[common.slider.geometry]\nthumb_size = 'math(0 / 28)'",
    ]).unwrap();
    assert_eq!(sheet.overlay_window.geometry.backdrop_opacity, Some(0.25));
    assert_eq!(sheet.slider.geometry.width, Some(64.0));
    assert_eq!(sheet.slider.geometry.thumb_size, Some(0.0));
    // Direct serde ingress is also used by look-owned stylesheet documents.
    let direct: CommonStylesheet = toml::from_str("[slider.geometry]\nheight = 'math(2 * 3)'").unwrap();
    assert_eq!(direct.slider.geometry.height, Some(6.0));
    for source in [
        "[common.slider.geometry]\nwidth = 'math(-1 / 2)'",
        "[common.overlay_window.geometry]\nbackdrop_opacity = 'math(3 / 2)'",
        "[common.slider.geometry]\nunknown = 'math(1 + 2)'",
    ] {
        assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
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
}

#[test]
fn selector_panel_chrome_inherits_tokens_and_placement_before_independent_overrides() {
    use crate::controls::{selector_list::default_selector_items_panel_look, selection_panel::default_selection_panel_look};
    let sheet = CommonStylesheet::resolve_layers(&[r#"
[common.popover.geometry]
window_margin=13
[common.popover.sizes.panel]
offset_y=9
[common.selection_panel.geometry]
border_width=0
disabled_opacity=0
radius=9
"#])
    .unwrap();
    let mut tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    tokens.metrics.border_width = 3.0;
    let look = default_selector_items_panel_look(&tokens);
    // The menu's colors and rows come from `.lss`; the border width from the tokens and placement from
    // the sheet.
    assert_eq!((look.border_width, look.popup_offset_y, look.window_margin), (3.0, 9.0, 13.0));
    let selection = default_selection_panel_look(&tokens);
    assert_eq!((selection.border_width, selection.disabled_opacity, selection.radius), (0.0, 0.0, 9.0));
    for family in ["selector_list", "selection_panel"] {
        for value in ["-1", "1.1", "nan"] {
            assert!(CommonStylesheet::parse(&format!("[common.{family}.geometry]\ndisabled_opacity={value}")).is_err());
        }
    }
}

#[test]
fn token_tables_are_not_stylesheet_fields() {
    // Token values live in the SDK's `.lss` token files, not in the common stylesheet.
    assert!(CommonStylesheet::parse("[common.tokens.metrics]\nborder_width = 3").is_err());
}
