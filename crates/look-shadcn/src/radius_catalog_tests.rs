//! The theme's CSS `--radius` must reach every SDK control Shadcn themes, now that
//! the SDK has no radius vocabulary and its own defaults are fixed pixels.

use xgpui_luma::controls::overlay_window::OverlayWindowMode;
use xgpui_luma::controls::toolbar::ToolbarVariant;

use crate::{ShadcnLook, ShadcnSize};

/// The fallback theme with its `--radius` (0.375rem in both modes) replaced.
fn look_with_radius(radius: &str) -> ShadcnLook {
    assert!(crate::FALLBACK_CSS.contains("--radius: 0.375rem;"), "fallback CSS must declare --radius: 0.375rem");
    let css = crate::FALLBACK_CSS.replace("--radius: 0.375rem;", &format!("--radius: {radius};"));
    ShadcnLook::from_css_str(&css).unwrap()
}

#[derive(Debug, PartialEq)]
struct Radii {
    floating_menu: f32,
    floating_menu_item: f32,
    selector_panel: f32,
    tooltip: f32,
    toolbar: f32,
    overlay_window: f32,
}

fn radii(look: &ShadcnLook) -> Radii {
    let menu = xgpui_luma::controls::floating_menu::styled_floating_menu_look(&look.floating_menu_theme().style());
    Radii {
        floating_menu: menu.radius,
        floating_menu_item: menu.item_radius,
        selector_panel: {
            use xgpui_luma::style::PartRead;
            look.selector_items_panel_style(ShadcnSize::Md).resolve(&[], 1.0).root().number("radius")
        },
        tooltip: crate::tooltip::tooltip_theme(look).resolve().radius,
        toolbar: look.toolbar_theme().resolve(true, ToolbarVariant::Outline).radius,
        overlay_window: look.overlay_window_theme().resolve(OverlayWindowMode::Modal).radius,
    }
}

#[test]
fn css_radius_reaches_sdk_controls() {
    let base = radii(&look_with_radius("0.375rem"));
    let large = radii(&look_with_radius("1rem"));

    // Every chrome radius follows the theme, not a fixed SDK default.
    assert!(large.floating_menu > base.floating_menu, "floating menu: {base:?} vs {large:?}");
    assert!(large.floating_menu_item > base.floating_menu_item, "menu items: {base:?} vs {large:?}");
    assert!(large.selector_panel > base.selector_panel, "selector panel: {base:?} vs {large:?}");
    assert!(large.tooltip > base.tooltip, "tooltip: {base:?} vs {large:?}");
    assert!(large.toolbar > base.toolbar, "toolbar: {base:?} vs {large:?}");
    assert!(large.overlay_window > base.overlay_window, "overlay window: {base:?} vs {large:?}");

    // `--radius` is Shadcn's large step; the toolbar uses medium (`--radius-md`).
    assert_eq!(large.toolbar, 14.0);
    // `ShadcnRadius` and `ShadcnRadii` share one table: both "medium" steps agree.
    assert_eq!(large.tooltip, large.toolbar);
    assert_eq!(base.tooltip, base.toolbar);
}
