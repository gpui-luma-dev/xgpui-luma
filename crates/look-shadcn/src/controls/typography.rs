//! Shared ShadcnSize → font-size mapping from common geometry with legacy metric fallback.

use serde::Deserialize;
use std::collections::HashMap;

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
