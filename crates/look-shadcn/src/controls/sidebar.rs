//! Navigation sidebar property mappings (shadcn / tweakcn sidebar tokens):
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Container bg      | `sidebar`                          |
//! | Container fg      | `sidebar-foreground`               |
//! | Container border  | `sidebar-border`                   |
//! | Section label     | `muted-foreground`                 |
//! | Item fg           | `sidebar-foreground`               |
//! | Item hover bg     | `accent`                           |
//! | Item hover fg     | `accent-foreground`                |
//! | Selected bg       | `sidebar-primary`                  |
//! | Selected fg       | `sidebar-primary-foreground`       |
//! | Focus ring        | `sidebar-ring`                     |

use crate::stylesheet::config::matches_optional_bool;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;

use xgpui_luma::controls::sidebar::SidebarItemLook;
use xgpui_luma::controls::sidebar::SidebarSectionLook;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::look::ShadcnLook;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_sidebar_branch_color_rule;
use crate::stylesheet::find_sidebar_container_color_rule;
use crate::stylesheet::find_sidebar_item_color_rule;
use crate::stylesheet::find_sidebar_section_color_rule;
use crate::stylesheet::resolve_sidebar_branch_color_rule;
use crate::stylesheet::resolve_sidebar_container_color_rule;
use crate::stylesheet::resolve_sidebar_item_color_rule;
use crate::stylesheet::resolve_sidebar_section_color_rule;
use crate::tokens::ShadcnTextSize;

#[derive(Clone, Debug)]
pub struct SidebarContainerColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

impl SidebarContainerColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_sidebar_container_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<SidebarContainerColorTable> {
    resolve_sidebar_container_colors_with_stylesheet(resolver, resolver.stylesheet())
}

pub fn resolve_sidebar_container_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<SidebarContainerColorTable> {
    let rule = find_sidebar_container_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar container color rule"))?;
    let colors = resolve_sidebar_container_color_rule(resolver, rule)?;
    Ok(SidebarContainerColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
    })
}

#[derive(Clone, Debug)]
pub struct SidebarSectionColorTable {
    pub label_color: ResolvedColor,
}

impl SidebarSectionColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_sidebar_section_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<SidebarSectionColorTable> {
    resolve_sidebar_section_colors_with_stylesheet(resolver, resolver.stylesheet())
}

pub fn resolve_sidebar_section_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<SidebarSectionColorTable> {
    let rule = find_sidebar_section_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar section color rule"))?;
    let colors = resolve_sidebar_section_color_rule(resolver, rule)?;
    Ok(SidebarSectionColorTable { label_color: colors.label_color })
}

#[derive(Clone, Debug)]
pub struct SidebarBranchColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl SidebarBranchColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_sidebar_branch_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarBranchColorTable> {
    resolve_sidebar_branch_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, layer)
}

pub fn resolve_sidebar_branch_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarBranchColorTable> {
    let rule = find_sidebar_branch_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar branch color rule"))?;
    let colors = resolve_sidebar_branch_color_rule(resolver, rule, layer)?;
    Ok(SidebarBranchColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

#[derive(Clone, Debug)]
pub struct SidebarItemColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl SidebarItemColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_sidebar_item_colors(
    resolver: &LookResolver<'_>,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarItemColorTable> {
    resolve_sidebar_item_colors_with_stylesheet(resolver, resolver.stylesheet(), selected, disabled, layer)
}

pub fn resolve_sidebar_item_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<SidebarItemColorTable> {
    let rule = find_sidebar_item_color_rule(stylesheet, selected, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching navigation sidebar item color rule"))?;
    let colors = resolve_sidebar_item_color_rule(resolver, rule, layer)?;
    Ok(SidebarItemColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        background: colors.background,
    })
}

pub fn sidebar_container_look(mode: &ShadcnModeTokens) -> SidebarContainerLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_container").with_stylesheet(mode.stylesheet());
    let colors =
        resolve_sidebar_container_colors(&resolver, true).unwrap_or_else(|_| SidebarContainerColorTable::fallback());
    SidebarContainerLook {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        radius: mode.metrics.radius.lg,
    }
}

pub fn sidebar_section_look(theme: &ShadcnLook) -> SidebarSectionLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_section").with_stylesheet(ctx.tokens.stylesheet());
    let colors =
        resolve_sidebar_section_colors(&resolver, true).unwrap_or_else(|_| SidebarSectionColorTable::fallback());
    SidebarSectionLook {
        label_color: colors.label_color.hsla(),
        typography: theme.typography_scale(ShadcnTextSize::Xs),
        height: crate::tables::metrics::resolve_sidebar_metrics(tokens.as_ref(), theme.mode(), ControlSize::Md)
            .section_height
            .value_px,
    }
}

fn base_item_look(_theme: &ShadcnLook, ctx: &LookContext, size: ControlSize) -> SidebarItemLook {
    let state = ctx.state;
    let typography = ctx.typography();
    let resolved_metrics = crate::tables::metrics::resolve_sidebar_metrics(ctx.tokens, ctx.theme_mode, size);
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_item_base").with_stylesheet(ctx.tokens.stylesheet());
    let colors = resolve_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarBranchColorTable::fallback());

    SidebarItemLook {
        background: None,
        focus_border: resolve_sidebar_focus_border(ctx.catalog(), ctx.theme_mode, state).map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
        icon_color: colors.icon_color.hsla(),
        typography: typography.text.label,
        radius: resolved_metrics.item_radius.value_px,
        height: resolved_metrics.item_height.value_px,
        padding_x: resolved_metrics.item_padding_x.value_px,
        gap: resolved_metrics.item_gap.value_px,
        icon_size: resolved_metrics.item_icon_size.value_px,
    }
}

pub fn sidebar_branch_look(theme: &ShadcnLook, state: InteractionState, size: ControlSize) -> SidebarItemLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), state);
    let mut look = base_item_look(theme, &ctx, size);
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_branch").with_stylesheet(ctx.tokens.stylesheet());
    let colors = resolve_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarBranchColorTable::fallback());

    look.background = colors.background.map(|color| color.hsla());
    look.foreground = colors.foreground.hsla();
    look.icon_color = colors.icon_color.hsla();

    look
}

pub fn sidebar_item_look(
    theme: &ShadcnLook,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> SidebarItemLook {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), state);
    let mut look = base_item_look(theme, &ctx, size);
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "sidebar_item").with_stylesheet(ctx.tokens.stylesheet());
    let colors = resolve_sidebar_item_colors(&resolver, selected, state.disabled, state.layer())
        .unwrap_or_else(|_| SidebarItemColorTable::fallback());

    look.background = colors.background.map(|color| color.hsla());
    look.foreground = colors.foreground.hsla();
    look.icon_color = colors.icon_color.hsla();

    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::look::ShadcnLook;
    use crate::mode::ShadcnModeTokens;
    use super::{sidebar_container_look, sidebar_item_look, sidebar_section_look};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("sidebar".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("sidebar-primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("sidebar-primary-foreground".into(), "oklch(1 0 0)".into()),
            ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("sidebar-accent-foreground".into(), "oklch(1 0 0)".into()),
            ("sidebar-border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("sidebar-ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn sidebar_uses_sidebar_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = ShadcnLook::from_css_str(
            ":root { --background: oklch(0.9735 0.0261 90.0953); --foreground: oklch(0.3092 0.0518 219.6516); --card: oklch(0.9306 0.0260 92.4020); --card-foreground: oklch(0.3092 0.0518 219.6516); --primary: oklch(0.5924 0.2025 355.8943); --primary-foreground: oklch(1 0 0); --secondary: oklch(0.6437 0.1019 187.3840); --secondary-foreground: oklch(1 0 0); --accent: oklch(0.5808 0.1732 39.5003); --accent-foreground: oklch(1 0 0); --muted: oklch(0.6979 0.0159 196.7940); --muted-foreground: oklch(0.3092 0.0518 219.6516); --border: oklch(0.6537 0.0197 205.2618); --input: oklch(0.6537 0.0197 205.2618); --ring: oklch(0.5924 0.2025 355.8943); --sidebar: oklch(0.9306 0.0260 92.4020); --sidebar-foreground: oklch(0.3092 0.0518 219.6516); --sidebar-primary: oklch(0.5924 0.2025 355.8943); --sidebar-primary-foreground: oklch(1 0 0); --sidebar-accent: oklch(0.5808 0.1732 39.5003); --sidebar-accent-foreground: oklch(1 0 0); --sidebar-border: oklch(0.6537 0.0197 205.2618); --sidebar-ring: oklch(0.5924 0.2025 355.8943); } .dark { --background: oklch(0.9735 0.0261 90.0953); --foreground: oklch(0.3092 0.0518 219.6516); --card: oklch(0.9306 0.0260 92.4020); --card-foreground: oklch(0.3092 0.0518 219.6516); --primary: oklch(0.5924 0.2025 355.8943); --primary-foreground: oklch(1 0 0); --secondary: oklch(0.6437 0.1019 187.3840); --secondary-foreground: oklch(1 0 0); --accent: oklch(0.5808 0.1732 39.5003); --accent-foreground: oklch(1 0 0); --muted: oklch(0.6979 0.0159 196.7940); --muted-foreground: oklch(0.3092 0.0518 219.6516); --border: oklch(0.6537 0.0197 205.2618); --input: oklch(0.6537 0.0197 205.2618); --ring: oklch(0.5924 0.2025 355.8943); --sidebar: oklch(0.9306 0.0260 92.4020); --sidebar-foreground: oklch(0.3092 0.0518 219.6516); --sidebar-primary: oklch(0.5924 0.2025 355.8943); --sidebar-primary-foreground: oklch(1 0 0); --sidebar-accent: oklch(0.5808 0.1732 39.5003); --sidebar-accent-foreground: oklch(1 0 0); --sidebar-border: oklch(0.6537 0.0197 205.2618); --sidebar-ring: oklch(0.5924 0.2025 355.8943); }"
        ).expect("look");
        let container = sidebar_container_look(&mode);
        let section = sidebar_section_look(&look);
        let selected = sidebar_item_look(&look, true, InteractionState::default(), ControlSize::Md);

        assert_eq!(container.background, catalog.color("sidebar").expect("sidebar"));
        assert_eq!(section.label_color, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_eq!(selected.background, Some(catalog.color("sidebar-primary").expect("sidebar-primary")));
        assert_eq!(
            selected.foreground,
            catalog.color("sidebar-primary-foreground").expect("sidebar-primary-foreground")
        );
    }

    #[test]
    fn focused_sidebar_item_does_not_panic_when_ring_token_is_missing() {
        let mut mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("ring");
        let ctx = crate::look_context::LookContext::new(
            &mode,
            ThemeMode::Light,
            InteractionState { focused: true, ..InteractionState::default() },
        );
        let look = super::base_item_look(&ShadcnLook::built_in(), &ctx, ControlSize::Md);

        assert_eq!(look.focus_border, Some(crate::provenance::ResolvedColor::fallback_foreground().hsla()));
    }
}

/// Sidebar focus border with the same disabled-state and missing-token behavior as paint.
pub fn resolve_sidebar_focus_border(
    catalog: &crate::catalog::CssTokenMap,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> Option<ResolvedColor> {
    (state.focused && !state.disabled).then(|| {
        LookResolver::new(catalog, theme_mode, "sidebar_focus")
            .resolve_decl("ring")
            .unwrap_or_else(|_| ResolvedColor::fallback_foreground())
    })
}

/// Theme colors used by standalone sidebar frames, separate from navigation items.
#[derive(Clone, Copy, Debug)]
pub struct SidebarContainerLook {
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    pub radius: f32,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SidebarStylesheet {
    #[serde(default)]
    pub metrics: SidebarMetricsRule,
    #[serde(default)]
    pub container: SidebarContainerStylesheet,
    #[serde(default)]
    pub section: SidebarSectionStylesheet,
    #[serde(default)]
    pub branch: SidebarBranchStylesheet,
    #[serde(default)]
    pub item: SidebarItemStylesheet,
}

impl SidebarStylesheet {
    /// Native bundled sidebar metrics and ordered rules.
    pub fn native_default() -> Self {
        Self {
            metrics: SidebarMetricsRule::default(),
            container: SidebarContainerStylesheet {
                color_rules: vec![SidebarContainerColorRule {
                    background: "first(sidebar,card,background)".into(),
                    foreground: "first(sidebar-foreground,foreground)".into(),
                    border: "first(sidebar-border,border)".into(),
                }],
            },
            section: SidebarSectionStylesheet {
                color_rules: vec![SidebarSectionColorRule { label_color: "muted-foreground".into() }],
            },
            branch: SidebarBranchStylesheet {
                color_rules: [
                    (true, None, "muted-foreground", None),
                    (false, Some("default"), "first(sidebar-foreground,foreground)", None),
                    (false, Some("disabled"), "first(sidebar-foreground,foreground)", None),
                    (false, Some("hover"), "accent-foreground", Some("accent")),
                    (false, Some("pressed"), "accent-foreground", Some("accent")),
                ]
                .into_iter()
                .map(|(disabled, layer, color, background)| SidebarBranchColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    foreground: color.into(),
                    icon_color: color.into(),
                    background: background.map(Into::into),
                })
                .collect(),
            },
            item: SidebarItemStylesheet {
                color_rules: [
                    (true, true, None, "muted-foreground", None),
                    (false, true, None, "muted-foreground", None),
                    (
                        true,
                        false,
                        Some("default"),
                        "first(sidebar-primary-foreground,primary-foreground)",
                        Some("first(sidebar-primary,primary)"),
                    ),
                    (
                        true,
                        false,
                        Some("hover"),
                        "first(sidebar-primary-foreground,primary-foreground)",
                        Some("first_layer(sidebar-primary,primary)"),
                    ),
                    (
                        true,
                        false,
                        Some("pressed"),
                        "first(sidebar-primary-foreground,primary-foreground)",
                        Some("first_layer(sidebar-primary,primary)"),
                    ),
                    (
                        true,
                        false,
                        Some("disabled"),
                        "first(sidebar-primary-foreground,primary-foreground)",
                        Some("first(sidebar-primary,primary)"),
                    ),
                    (false, false, Some("default"), "first(sidebar-foreground,foreground)", None),
                    (false, false, Some("disabled"), "first(sidebar-foreground,foreground)", None),
                    (false, false, Some("hover"), "accent-foreground", Some("accent")),
                    (false, false, Some("pressed"), "accent-foreground", Some("accent")),
                ]
                .into_iter()
                .map(|(selected, disabled, layer, color, background)| SidebarItemColorRule {
                    selected: Some(selected),
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    foreground: color.into(),
                    icon_color: color.into(),
                    background: background.map(Into::into),
                })
                .collect(),
            },
        }
    }
}

/// Layout metrics for [`xgpui_luma::controls::sidebar::SidebarMetricScale`].
///
/// Values accept bare numbers, `px`, or `rem` (1rem = 16px).
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SidebarMetricsRule {
    #[serde(default = "sidebar_metric_width_expanded")]
    pub width_expanded: String,
    #[serde(default = "sidebar_metric_width_icon_rail")]
    pub width_icon_rail: String,
    #[serde(default = "sidebar_metric_width_mobile")]
    pub width_mobile: String,
    #[serde(default = "sidebar_metric_item_height")]
    pub item_height: String,
    #[serde(default = "sidebar_metric_icon_size")]
    pub icon_size: String,
    #[serde(default = "sidebar_metric_rail_hit_width")]
    pub rail_hit_width: String,
    #[serde(default = "sidebar_metric_popover_offset")]
    pub popover_offset: String,
}

impl Default for SidebarMetricsRule {
    fn default() -> Self {
        Self {
            width_expanded: sidebar_metric_width_expanded(),
            width_icon_rail: sidebar_metric_width_icon_rail(),
            width_mobile: sidebar_metric_width_mobile(),
            item_height: sidebar_metric_item_height(),
            icon_size: sidebar_metric_icon_size(),
            rail_hit_width: sidebar_metric_rail_hit_width(),
            popover_offset: sidebar_metric_popover_offset(),
        }
    }
}

fn sidebar_metric_width_expanded() -> String {
    "16rem".into()
}
fn sidebar_metric_width_icon_rail() -> String {
    "3rem".into()
}
fn sidebar_metric_width_mobile() -> String {
    "18rem".into()
}
fn sidebar_metric_item_height() -> String {
    "2rem".into()
}
fn sidebar_metric_icon_size() -> String {
    "1rem".into()
}
fn sidebar_metric_rail_hit_width() -> String {
    "0.375rem".into()
}
fn sidebar_metric_popover_offset() -> String {
    "0.5rem".into()
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SidebarContainerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarContainerColorRule>,
}

impl SidebarContainerStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarContainerColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SidebarContainerColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SidebarSectionStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarSectionColorRule>,
}

impl SidebarSectionStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarSectionColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SidebarSectionColorRule {
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SidebarBranchStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarBranchColorRule>,
}

impl SidebarBranchStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&SidebarBranchColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SidebarBranchColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SidebarItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarItemColorRule>,
}

impl SidebarItemStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&SidebarItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled && matches_optional_bool(rule.selected, selected);
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.selected, selected) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SidebarItemColorRule {
    pub selected: Option<bool>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}

/// Native common geometry owned by this control.
pub(crate) fn native_sidebar_stylesheet() -> xgpui_luma::theme::stylesheet::SidebarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    SidebarStylesheet {
        geometry: SidebarGeometryStylesheet {
            section_height: Some(20.0),
            item_height: Some(30.0),
            item_padding_x: Some(8.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct SidebarMetricTable {
    pub section_height: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
}

pub fn resolve_sidebar_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> SidebarMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, radius_metric, spacing_control_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let size_metrics = metrics.for_size(size);

    let mut table = SidebarMetricTable {
        section_height: derived_metric("navigation sidebar section height", 20.0),
        item_height: derived_metric("navigation sidebar item height", 30.0),
        item_padding_x: derived_metric("navigation sidebar item padding x", 8.0),
        item_gap: spacing_control_metric(catalog, size, crate::catalog::SpacingField::Gap, size_metrics.gap),
        item_radius: radius_metric(catalog, size, metrics.radius(size)),
        item_icon_size: derived_metric("navigation sidebar icon size", 16.0),
    };
    let geometry = mode.stylesheet().common.sidebar.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::SidebarGeometry {
            section_height: table.section_height.value_px,
            item_height: table.item_height.value_px,
            item_padding_x: table.item_padding_x.value_px,
            item_gap: table.item_gap.value_px,
            item_icon_size: table.item_icon_size.value_px,
            ..Default::default()
        },
    );
    table.section_height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.section_height, table.section_height);
    table.item_height = crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_height, table.item_height);
    table.item_padding_x =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_padding_x, table.item_padding_x);
    table.item_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_gap, table.item_gap);
    table.item_icon_size =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_icon_size, table.item_icon_size);
    table
}
