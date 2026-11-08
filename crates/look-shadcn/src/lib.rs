#[cfg(feature = "inspect")]
pub mod inspect;

mod action;
mod look_context;
pub mod catalog;
mod color;
mod context;
mod controls;
mod elements;
mod ext;
mod focus;
mod look;
mod mode;
mod palette;
mod provenance;
mod resolve;
mod shadow;
mod size;
mod state_color;
pub mod stylesheet;
mod tokens;
mod usage;
mod tooltip;
pub use tooltip::tooltip_theme;

#[cfg(test)]
mod test_support;

pub mod paint;
pub mod prelude;
pub mod tables;

pub use catalog::{CssTokenCatalog, CssTokenMap, parse_css_catalog};
pub use context::{sync_color_control_theme, with_look};
/// Look-agnostic provenance types (shared with other looks via the SDK).
pub use xgpui_luma::theme::provenance::{
    ColorSource as LookColorSource, MetricSource as LookMetricSource, ResolvedColor as LookResolvedColor,
    ResolvedMetric as LookResolvedMetric, ResolvedTypography as LookResolvedTypography,
    TypographySource as LookTypographySource,
};
pub use ext::{LumaTypographyExt, ShadcnElementExt};
pub use look::ShadcnLook;

/// Bundled fallback theme CSS, shared by the fallback look and demo apps.
///
/// Parse with [`ShadcnLook::from_css_str`] to create an independently mutable look.
pub const FALLBACK_CSS: &str = include_str!("../assets/fallback.css");

pub use shadow::{SHADOW_LADDER_TOKENS, ShadowTokenParts, shadow_ladder_overrides};
pub use size::ShadcnSize;
pub use mode::ShadcnModeTokens;
pub use palette::{ShadcnActionRole, ShadcnPalette};
pub use tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnStyle, ShadcnTextRole, ShadcnTextSize, ShadcnToken};
pub use look_context::LookContext;
pub use stylesheet::{
    ColorRuleMetadataSection, StylesheetConfig, all_color_rule_metadata, embedded_color_rule_metadata,
    embedded_stylesheet,
};
pub use elements::{Badge, BadgeLook, BadgeColorTable, BadgeIconPlacement, BadgeVariant, badge_look, resolve_badge_colors};
pub use controls::{
    Frame, Accordion, Autocomplete, Button, ButtonRadiusPreset, Card, Checkbox, ComboBox, ContextMenu, IconGroup,
    Table, ListBoxBuilder, MenuChoiceGroup, Pager, PopupMenu, Progress, Radio, RadioGroup, Scrollbar, SearchSelector,
    SelectionPanel, Selector, ShadcnButtonStyle, ShadcnCard, Sidebar, Slider, SplitButton, Stepper, Switch, Tabs,
    TextArea, TextField, Toggle, ToggleLayout, Toolbar, TreeView, ShadcnTextFieldStyle, ShadcnToolbarItemExt,
    ToolbarTextFieldItemBuilder, slide_panel_background, slide_panel_panels_look,
};
pub use provenance::{
    ColorSource, LookResolver, ResolvedSourceColor, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography,
    TypographySource,
};
pub use usage::all_shadcn_theme_usages;

#[cfg(test)]
mod button_stylesheet_tests;

#[cfg(test)]
mod choice_stylesheet_tests;

#[cfg(test)]
mod remaining_stylesheet_tests;

#[cfg(all(test, feature = "test-support"))]
mod event_callback_tests;

#[cfg(all(test, feature = "test-support"))]
xgpui_luma::test_look_motion_contract!(crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap(), crate);

#[cfg(all(test, feature = "test-support"))]
xgpui_luma::test_stylesheet_motion_contract!(
    |source: &str| {
        let stylesheet = crate::stylesheet::StylesheetConfig::parse(source).unwrap();
        crate::ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, stylesheet).unwrap()
    },
    crate
);
