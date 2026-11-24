use xgpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::selector::interaction_layer_key;

pub use crate::controls::typography::{TypographyStylesheet, TypographyRule};
pub use crate::controls::button::{ButtonStylesheet, ButtonMetricsRule};
pub use crate::controls::toggle::ToggleStylesheet;
pub use crate::controls::sidebar::{SidebarStylesheet, SidebarMetricsRule, SidebarContainerColorRule};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct StylesheetConfig {
    #[serde(default)]
    pub common: xgpui_luma::theme::stylesheet::CommonStylesheet,
    #[serde(default)]
    pub typography: TypographyStylesheet,
    #[serde(default)]
    pub button: ButtonStylesheet,
    #[serde(default)]
    pub toggle: ToggleStylesheet,
    #[serde(default, alias = "navigation_sidebar")]
    pub sidebar: SidebarStylesheet,
}

impl StylesheetConfig {
    /// Construct bundled control definitions directly in Rust.
    pub fn native_default() -> Self {
        Self {
            common: super::native_shadcn_common_stylesheet(),
            typography: TypographyStylesheet::native_default(),
            button: ButtonStylesheet::native_default(),
            toggle: ToggleStylesheet::native_default(),
            sidebar: SidebarStylesheet::native_default(),
        }
    }

    /// Parses stylesheet TOML without imposing an input-size limit.
    /// Applications own file access and any limits required for untrusted input.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        let mut stylesheet: Self = toml::from_str(source).map_err(|err| anyhow::anyhow!("parse stylesheet: {err}"))?;
        stylesheet.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[source])?;
        Ok(stylesheet)
    }
}

pub(crate) fn matches_optional_bool(selector: Option<bool>, value: bool) -> bool {
    selector.is_none_or(|expected| expected == value)
}

pub(crate) fn matches_optional_layer(selector: Option<&str>, layer: InteractionLayer) -> bool {
    selector.is_none_or(|expected| expected == interaction_layer_key(layer))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TYPOGRAPHY_SNIPPET: &str = r#"
[typography.semantic.h1]
size = 44.0
line_height = 52.0
weight = 700.0

[typography.scale.xs]
size = 11.0
line_height = 16.0
weight = 500.0

[typography.scale."2xl"]
size = 20.0
line_height = 28.0
weight = 600.0
"#;

    const BUTTON_SNIPPET: &str = r#"
[button.metrics.md]
height = "metrics.control.md"
padding_horizontal = 16.0
font_size = 14.0
icon_size = 16.0
corner_radius = "radius"
"#;

    #[test]
    fn parses_typography_rules() {
        let config = StylesheetConfig::parse(TYPOGRAPHY_SNIPPET).expect("parse typography snippet");
        assert_eq!(config.typography.semantic_rule("h1").expect("h1").size, 44.0);
        assert_eq!(config.typography.scale_rule("xs").expect("xs").line_height, 16.0);
        assert_eq!(config.typography.scale_rule("2xl").expect("2xl").weight, 600.0);
    }

    #[test]
    fn parses_button_metrics() {
        let config = StylesheetConfig::parse(BUTTON_SNIPPET).expect("parse snippet");
        assert!(config.button.metrics.contains_key("md"));
    }

    /// Button and toggle colors and shadows come from `.lss`: rules for them are an error, not ignored.
    #[test]
    fn color_and_elevation_rules_are_rejected() {
        for source in [
            "[[button.color_rules]]\nbackground = \"primary\"",
            "[[button.elevation_rules]]\nstyle = \"outline\"\nshadow = \"shadow-xs\"",
            "[[toggle.elevation_rules]]\nshadow = \"shadow-sm\"",
        ] {
            let error = StylesheetConfig::parse(source).expect_err(source);
            assert!(error.to_string().contains("rules"), "{source}: {error}");
        }
    }
}
