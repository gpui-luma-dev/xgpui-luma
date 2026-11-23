use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::controls::ShadcnButtonStyle;
use crate::tokens::{ShadcnTextRole, ShadcnTextSize};

use super::config::{ButtonColorRule, SidebarContainerColorRule, StylesheetConfig, TypographyRule};
use super::selector::{AsSelectorState, ButtonSelectorState};

pub fn find_button_color_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> Option<&ButtonColorRule> {
    let state = ButtonSelectorState { style, layer, mode: theme_mode, selected };
    let map = state.to_selector_map();
    stylesheet.button.find_color_rule(
        map.get("style").expect("style key"),
        map.get("layer").expect("layer key"),
        map.get("mode").expect("mode key"),
        selected,
    )
}

pub fn find_button_elevation_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<&crate::stylesheet::config::ButtonElevationRule> {
    stylesheet.button.elevation_rule_for_style(crate::stylesheet::selector::button_style_key(style))
}

pub fn find_sidebar_metrics(stylesheet: &StylesheetConfig) -> &crate::stylesheet::config::SidebarMetricsRule {
    &stylesheet.sidebar.metrics
}

pub fn find_sidebar_container_color_rule(stylesheet: &StylesheetConfig) -> Option<&SidebarContainerColorRule> {
    stylesheet.sidebar.container.color_rule()
}

pub fn find_typography_scale_rule(stylesheet: &StylesheetConfig, size: ShadcnTextSize) -> Option<&TypographyRule> {
    stylesheet.typography.scale_rule(size.as_str())
}

pub fn find_typography_semantic_rule(stylesheet: &StylesheetConfig, role: ShadcnTextRole) -> Option<&TypographyRule> {
    stylesheet.typography.semantic_rule(role.as_str())
}
