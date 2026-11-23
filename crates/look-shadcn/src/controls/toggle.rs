use crate::controls::button::{ButtonMetricsRule, ButtonMetricTokensRule};
use serde::Deserialize;
use std::collections::HashMap;

use crate::size::ShadcnSize;

/// Inline padded toggle (shadcn default). Square geometry requires explicit `.round(true)` on the builder.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleLayout {
    #[default]
    Inline,
    Square,
}

impl ToggleLayout {
    pub fn from_round(round: bool) -> Self {
        if round { Self::Square } else { Self::Inline }
    }
}

// Mirrors the public semantic resolver with an additional selected stylesheet.

// Mirrors the public semantic resolver with an additional selected stylesheet.

/// Native common geometry owned by this control.
pub(crate) fn native_toggle_stylesheet() -> xgpui_luma::theme::stylesheet::ToggleStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ToggleStylesheet {
        sizes: [("sm", 10.0, 12.0, 14.0, 36.0), ("md", 12.0, 14.0, 16.0, 40.0), ("lg", 20.0, 16.0, 18.0, 44.0)]
            .into_iter()
            .map(|(key, padding, font, icon, height)| {
                (
                    key.into(),
                    ToggleGeometryStylesheet {
                        padding_x: Some(padding),
                        font_size: Some(font),
                        icon_size: Some(icon),
                        height: Some(height),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ToggleStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
}

impl ToggleStylesheet {
    /// Native bundled toggle token references.
    pub fn native_default() -> Self {
        Self {
            tokens: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), ButtonMetricTokensRule { height: None, corner_radius: Some("radius".into()) }))
                .collect(),
            ..Default::default()
        }
    }

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ShadcnSize::Sm => "sm",
            ShadcnSize::Md => "md",
            ShadcnSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}
