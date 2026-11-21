//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border-hover`     |
//! | Disabled          | `muted-foreground` |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use serde::Deserialize;

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct SplitViewColorTable {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

impl SplitViewColorTable {
    pub fn fallback() -> Self {
        Self { separator: ResolvedColor::transparent(), separator_hover: ResolvedColor::fallback_foreground() }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SplitViewStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SplitViewColorRule>,
}

impl SplitViewStylesheet {
    /// Native bundled separator colors.
    pub fn native_default() -> Self {
        Self {
            color_rules: [(true, "border", "border-hover"), (false, "muted-foreground", "muted-foreground")]
                .into_iter()
                .map(|(enabled, separator, hover)| SplitViewColorRule {
                    enabled: Some(enabled),
                    separator: separator.into(),
                    separator_hover: hover.into(),
                })
                .collect(),
        }
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&SplitViewColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SplitViewColorRule {
    pub enabled: Option<bool>,
    pub separator: String,
    pub separator_hover: String,
}

impl EnabledColorRule for SplitViewColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}
