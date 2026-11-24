use super::CommonStylesheet;
use crate::{color::GamutMapping, theme::ThemeMode};

#[test]
fn sparse_sheet_prepares_a_complete_sdk_theme_and_preserves_zero() {
    let sheet = CommonStylesheet::parse("[common.row_limits.pager]\nmin=0").unwrap();
    let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    let baseline = CommonStylesheet::sdk_default().row_limits.0["pager"].max;
    assert!(baseline.is_some(), "the SDK baseline sets the pager's row maximum");
    let pager = tokens.stylesheet.row_limits.0["pager"];
    assert_eq!(pager.max, baseline, "unset fields come from the baseline");
    assert_eq!(pager.min, Some(0), "an authored zero is kept");
}

#[test]
fn math_values_resolve_through_layers_and_geometry_validation() {
    let sheet = CommonStylesheet::resolve_layers(&[
        "[common.row_limits.pager]\nmax = 'math(32 * 2)'",
        "[common.row_limits.combobox]\nmin = 'math(0 / 28)'",
    ])
    .unwrap();
    assert_eq!(sheet.row_limits.0["pager"].max, Some(64));
    assert_eq!(sheet.row_limits.0["combobox"].min, Some(0));
    // Direct serde ingress is also used by look-owned stylesheet documents.
    let direct: CommonStylesheet = toml::from_str("[row_limits.pager]\nmax = 'math(2 * 3)'").unwrap();
    assert_eq!(direct.row_limits.0["pager"].max, Some(6));
    for source in [
        "[common.row_limits.pager]\nmax = 'math(-1 / 2)'",
        "[common.row_limits.pager]\nmax = 'math(3 / 2)'",
        "[common.row_limits.pager]\nunknown = 'math(1 + 2)'",
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
fn selector_panel_chrome_inherits_tokens_and_popover_placement() {
    use crate::controls::{selector_list::default_selector_items_panel_look, selection_panel::default_selection_panel_look};
    let mut tokens =
        CommonStylesheet::sdk_default().theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
    tokens.metrics.border_width = 3.0;
    let look = default_selector_items_panel_look(&tokens);
    // The menu's colors and rows come from `.lss`, the border width from the tokens, and the placement from
    // the SDK's `popover.lss` panel offset and margin.
    let sdk = |name| crate::style::sdk_number("popover", name);
    assert_eq!(
        (look.border_width, look.popup_offset_y, look.window_margin),
        (3.0, sdk("panel_offset_y"), sdk("window_margin"))
    );
    let selection = default_selection_panel_look(&tokens);
    // The selection panel is the items panel: no geometry of its own.
    assert_eq!(
        (selection.border_width, selection.disabled_opacity, selection.radius),
        (look.border_width, look.disabled_opacity, look.radius)
    );
}

#[test]
fn token_tables_are_not_stylesheet_fields() {
    // Token values live in the SDK's `.lss` token files, not in the common stylesheet.
    assert!(CommonStylesheet::parse("[common.tokens.metrics]\nborder_width = 3").is_err());
}

/// Families no control reads any more are not stylesheet sections: a sheet that sets them is an
/// error naming the section, not silently ignored (#287).
#[test]
fn removed_families_are_rejected() {
    for family in [
        "pager",
        "context_menu",
        "dock_splitter",
        "toolbar",
        "textfield",
        "textarea",
        "selector",
        "progress",
        "overlay_window",
        "popup_menu",
        "accordion",
        "tree_view",
        "avatar",
        "badge",
        "card",
        "callout",
        "segmented",
        "floating_menu",
        "split_view",
        "stepper",
        "scrollbar",
        "icon_group",
        "toggle",
        "selection_panel",
        "table",
        "control_group",
        "radio_group",
        "selector_list",
        "resizable_panels",
        "tabs",
        "button",
        "shared",
        "slider",
        "tooltip",
        "slide_panel",
        "popover",
        "sidebar",
        "autocomplete",
        "combobox",
        "search_selector",
        "shell",
    ] {
        let error = CommonStylesheet::parse(&format!("[common.{family}.geometry]\nheight = 10")).expect_err(family);
        assert!(error.to_string().contains(family), "{family}: {error}");
    }
}
