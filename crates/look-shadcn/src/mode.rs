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
    /// Shared metrics; `control` holds the Shadcn `default` (md) size.
    pub metrics: MetricTokens,
    /// Shadcn control metrics for each [`crate::ShadcnSize`].
    pub control_sizes: crate::ShadcnControlMetrics,
    /// Shadcn's named corner radii in pixels.
    pub radii: crate::ShadcnRadii,
    pub typography: LumaTypography,
    state_colors: StateColorTable,
    shadow_previews: HashMap<String, Vec<gpui::BoxShadow>>,
    stylesheet: Arc<crate::stylesheet::StylesheetConfig>,
    /// Common stylesheet with each Shadcn size merged into general geometry, in
    /// [`crate::ShadcnSize::ALL`] order. SDK controls only read these pixel values.
    sized_common: Arc<[xgpui_luma::theme::stylesheet::CommonStylesheet; 3]>,
    /// This mode's catalog bound to the `.lss` stylesheet, made on first use.
    style_theme: StyleThemeSlot,
}

/// A lazily bound `.lss` theme. A clone starts empty, so a changed copy of the tokens binds afresh.
#[derive(Default)]
struct StyleThemeSlot(std::sync::OnceLock<xgpui_luma::style::lss::Theme>);

impl Clone for StyleThemeSlot {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl std::fmt::Debug for StyleThemeSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StyleThemeSlot")
    }
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
        let (metrics, control_sizes, radii) = metrics_from_catalog(&catalog, MetricTokens::default());
        let sized_common = Arc::new(crate::ShadcnSize::ALL.map(|size| stylesheet.common.with_size_key(size.key())));
        Ok(Self {
            data_positive,
            theme_mode,
            metrics,
            control_sizes,
            radii,
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
            state_colors,
            shadow_previews,
            stylesheet,
            sized_common,
            palette,
            catalog,
            style_theme: StyleThemeSlot::default(),
        })
    }

    /// This mode's catalog bound to the look's `.lss` stylesheet, for values read without the look
    /// (menus built from mode tokens).
    pub fn style_theme(&self) -> xgpui_luma::style::lss::Theme {
        self.style_theme.0.get_or_init(|| crate::style::theme_for_mode(self)).clone()
    }

    /// Shared metrics whose `control` scale is the given Shadcn size, for SDK geometry.
    pub fn metrics_for(&self, size: crate::ShadcnSize) -> MetricTokens {
        let mut metrics = self.metrics;
        metrics.control = self.control_sizes.get(size);
        metrics
    }

    /// Common stylesheet with `size` merged into general geometry, for SDK controls.
    pub fn common_for(&self, size: crate::ShadcnSize) -> &xgpui_luma::theme::stylesheet::CommonStylesheet {
        let index = crate::ShadcnSize::ALL.iter().position(|candidate| *candidate == size).unwrap_or(1);
        &self.sized_common[index]
    }

    /// Common stylesheet for `size`, reusing the prepared copy when `stylesheet` is this
    /// snapshot's own configuration.
    pub(crate) fn sized_common<'a>(
        &'a self,
        stylesheet: &'a crate::stylesheet::StylesheetConfig,
        size: crate::ShadcnSize,
    ) -> std::borrow::Cow<'a, xgpui_luma::theme::stylesheet::CommonStylesheet> {
        if std::ptr::eq(stylesheet, self.stylesheet.as_ref()) {
            std::borrow::Cow::Borrowed(self.common_for(size))
        } else {
            std::borrow::Cow::Owned(stylesheet.common.with_size_key(size.key()))
        }
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
