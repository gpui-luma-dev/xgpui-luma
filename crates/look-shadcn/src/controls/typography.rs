//! Shared ControlSize → font-size mapping from common geometry with legacy metric fallback.

use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{resolve_button_metrics_rule};

/// Preserve the supplied typography ratio while applying configured button sizes.
pub fn apply_button_metrics_typography(typography: &mut LumaTextStyle, mode: &ShadcnModeTokens, size: ControlSize) {
    if let Some(rule) = mode.stylesheet().button.metrics_for_size(size) {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }
    apply_common_button_typography(typography, mode.stylesheet(), size);
}

pub(crate) fn apply_common_button_typography(
    typography: &mut LumaTextStyle,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
) {
    let (font, line, _, _) = crate::tables::typography::common_typography_overrides(stylesheet, size, false);
    if let Some(font) = font {
        let ratio = if typography.size > 0.0 {
            typography.line_height / typography.size
        } else {
            1.0
        };
        typography.size = font;
        typography.line_height = font * ratio;
    }
    if let Some(line) = line {
        typography.line_height = line;
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TypographyStylesheet {
    #[serde(default)]
    pub semantic: HashMap<String, TypographyRule>,
    #[serde(default)]
    pub scale: HashMap<String, TypographyRule>,
}

impl TypographyStylesheet {
    /// Native bundled typography rules.
    pub fn native_default() -> Self {
        Self {
            semantic: [
                ("h1", 32.0, 40.0, 700.0),
                ("h2", 24.0, 32.0, 600.0),
                ("h3", 18.0, 26.0, 600.0),
                ("h4", 16.0, 24.0, 600.0),
                ("p", 14.0, 20.0, 400.0),
            ]
            .into_iter()
            .map(|(key, size, line_height, weight)| (key.into(), TypographyRule { size, line_height, weight }))
            .collect(),
            scale: [
                ("xs", 12.0, 16.0, 500.0),
                ("sm", 13.0, 18.0, 400.0),
                ("base", 14.0, 20.0, 400.0),
                ("lg", 16.0, 24.0, 500.0),
                ("xl", 18.0, 26.0, 600.0),
                ("2xl", 20.0, 28.0, 600.0),
            ]
            .into_iter()
            .map(|(key, size, line_height, weight)| (key.into(), TypographyRule { size, line_height, weight }))
            .collect(),
        }
    }

    pub fn semantic_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.semantic.get(key)
    }

    pub fn scale_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.scale.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct TypographyRule {
    pub size: f32,
    pub line_height: f32,
    pub weight: f32,
}
