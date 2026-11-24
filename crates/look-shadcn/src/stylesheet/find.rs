use crate::tokens::{ShadcnTextRole, ShadcnTextSize};

use super::config::{SidebarContainerColorRule, StylesheetConfig, TypographyRule};

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
