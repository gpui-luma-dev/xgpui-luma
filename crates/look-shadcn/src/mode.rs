use anyhow::Result;
use std::{sync::Arc, collections::HashMap};

use xgpui_luma::theme::{InteractionLayer, LumaTypography, MetricTokens, ThemeMode};

use crate::catalog::{CssTokenMap, metrics_from_catalog, typography_from_catalog};
use crate::palette::ShadcnPalette;
use crate::state_color::StateColorTable;
use crate::tokens::ShadcnToken;

#[derive(Clone, Debug)]
pub struct ShadcnModeTokens {
    /// Shared positive status color projected once with the selected stylesheet.
    pub data_positive: gpui::Hsla,
    pub(crate) theme_mode: ThemeMode,
    pub catalog: CssTokenMap,
    pub palette: ShadcnPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    state_colors: StateColorTable,
    shadow_previews: HashMap<String, Vec<gpui::BoxShadow>>,
    stylesheet: Arc<crate::stylesheet::StylesheetConfig>,
}

impl ShadcnModeTokens {
    pub fn from_catalog(catalog: CssTokenMap, theme_mode: ThemeMode) -> Result<Self> {
        Self::from_catalog_with_stylesheet(
            catalog,
            theme_mode,
            Arc::new(crate::stylesheet::embedded_stylesheet().clone()),
        )
    }

    pub(crate) fn from_catalog_with_stylesheet(
        catalog: CssTokenMap,
        theme_mode: ThemeMode,
        stylesheet: Arc<crate::stylesheet::StylesheetConfig>,
    ) -> Result<Self> {
        let palette = ShadcnPalette::from_catalog(&catalog, theme_mode)?;
        let state_colors = StateColorTable::from_catalog(&catalog, &palette, theme_mode);
        let mut shadow_previews = HashMap::new();
        for token in crate::shadow::SHADOW_LADDER_TOKENS {
            if catalog.get(token).is_some() {
                let previews = crate::shadow::parse_shadow_token(&catalog, token)?;
                shadow_previews.insert(token.to_string(), previews);
            }
        }
        let data_positive = xgpui_luma::color::gpui_bridge::to_hsla(
            stylesheet.common.tokens.source(theme_mode).palette.data.positive,
            xgpui_luma::color::GamutMapping::CssLocalMinde,
        )?;
        Ok(Self {
            data_positive,
            theme_mode,
            metrics: metrics_from_catalog(&catalog, MetricTokens::default()),
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
            state_colors,
            shadow_previews,
            stylesheet,
            palette,
            catalog,
        })
    }

    /// Standard shadow previews are prepared with this immutable theme snapshot.
    pub(crate) fn shadow_preview(&self, token: &str) -> Option<&[gpui::BoxShadow]> {
        self.shadow_previews.get(token).map(Vec::as_slice)
    }

    /// Configuration captured with these palette/metric tokens.
    pub(crate) fn stylesheet(&self) -> &crate::stylesheet::StylesheetConfig {
        &self.stylesheet
    }

    /// Retained source/derived color before the current GPUI sRGB projection.
    pub fn resolve_source_color_state(
        &self,
        token: ShadcnToken,
        layer: InteractionLayer,
    ) -> xgpui_luma::color::ColorValue {
        self.state_colors.source(token, layer)
    }

    /// Resolves a semantic token color for an interaction layer (O(1) lookup).
    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> gpui::Hsla {
        self.state_colors.get(token, layer)
    }
}
