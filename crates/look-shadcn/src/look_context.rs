use xgpui_luma::theme::{InteractionLayer, InteractionState, LumaTypography, MetricTokens, ThemeMode};

use super::catalog::CssTokenMap;
use super::mode::ShadcnModeTokens;
use super::palette::ShadcnPalette;
use super::tokens::ShadcnToken;

/// Bundles styling dependencies and interactive state for control look resolution.
pub struct LookContext<'a> {
    pub tokens: &'a ShadcnModeTokens,
    pub theme_mode: ThemeMode,
    pub state: InteractionState,
}

impl<'a> LookContext<'a> {
    pub fn new(tokens: &'a ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> Self {
        Self { tokens, theme_mode, state }
    }

    pub fn catalog(&self) -> &CssTokenMap {
        &self.tokens.catalog
    }

    pub fn palette(&self) -> &ShadcnPalette {
        &self.tokens.palette
    }

    pub fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }

    pub fn typography(&self) -> &LumaTypography {
        &self.tokens.typography
    }

    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> gpui::Hsla {
        self.tokens.resolve_color_state(token, layer)
    }
}
