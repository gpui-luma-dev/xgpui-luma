mod metrics;
mod parse;

use std::collections::BTreeMap;

use anyhow::{Context as _, Result, anyhow};
use gpui::Hsla;

pub use parse::{CssTokenCatalog, parse_css_catalog};

use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CssTokenMap {
    pub tokens: BTreeMap<String, String>,
}

impl CssTokenMap {
    pub fn from_map(tokens: BTreeMap<String, String>) -> Self {
        Self { tokens }
    }

    pub fn get(&self, token: &str) -> Option<&str> {
        self.tokens.get(token).map(String::as_str)
    }

    /// Parse the original source space and components without gamut mapping.
    pub fn source_color(&self, token: &str) -> Result<ColorValue> {
        let raw = self.get(token).ok_or_else(|| anyhow!("missing css token `--{token}`"))?;
        ColorValue::parse_css(raw).with_context(|| format!("parse css color for `--{token}`"))
    }

    /// First present token; malformed values are errors rather than fallback requests.
    pub fn source_color_first(&self, tokens: &[&str]) -> Result<ColorValue> {
        self.optional_source_color(tokens)?
            .ok_or_else(|| anyhow!("missing css token (tried: {})", tokens.join(", ")))
    }

    pub fn optional_source_color(&self, tokens: &[&str]) -> Result<Option<ColorValue>> {
        for token in tokens {
            if self.get(token).is_some() {
                return self.source_color(token).map(Some);
            }
        }
        Ok(None)
    }

    /// Current GPUI sRGB preview; retain `source_color` for storage and derivations.
    pub fn color(&self, token: &str) -> Result<Hsla> {
        gpui_bridge::to_hsla(self.source_color(token)?, GamutMapping::CssLocalMinde)
    }

    pub fn color_first(&self, tokens: &[&str]) -> Result<Hsla> {
        gpui_bridge::to_hsla(self.source_color_first(tokens)?, GamutMapping::CssLocalMinde)
    }

    pub fn optional_color(&self, tokens: &[&str]) -> Result<Option<Hsla>> {
        self.optional_source_color(tokens)?
            .map(|source| gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde))
            .transpose()
    }
}

impl CssTokenCatalog {
    pub fn light_map(&self) -> CssTokenMap {
        CssTokenMap::from_map(self.light.clone())
    }

    pub fn dark_map(&self) -> CssTokenMap {
        CssTokenMap::from_map(self.dark.clone())
    }
}

pub use metrics::{SpacingField, spacing_multiplier};
pub(crate) use metrics::{metrics_from_catalog, typography_from_catalog};

pub(crate) use metrics::parse_length_px;
