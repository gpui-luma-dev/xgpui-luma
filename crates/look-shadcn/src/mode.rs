use anyhow::Result;
use std::{sync::Arc, collections::HashMap};

use xgpui_luma::theme::{InteractionLayer, LumaTypography, MetricTokens, ThemeMode};

use crate::catalog::{CssTokenMap, metrics_from_theme, radii_from_catalog, typography_from_catalog};
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
            match theme_mode {
                ThemeMode::Light => xgpui_luma::theme::LumaThemeMode::light(),
                ThemeMode::Dark => xgpui_luma::theme::LumaThemeMode::dark(),
            }
            .palette
            .data
            .positive,
            xgpui_luma::color::GamutMapping::CssLocalMinde,
        )?;
        let radii = radii_from_catalog(&catalog);
        // Metrics come from the look's `.lss`, bound to this catalog below. The theme's native
        // tokens read only the catalog, so these placeholders don't reach it.
        let placeholder = MetricTokens::default();
        let mut tokens = Self {
            data_positive,
            theme_mode,
            metrics: placeholder,
            control_sizes: crate::ShadcnControlMetrics {
                sm: placeholder.control,
                md: placeholder.control,
                lg: placeholder.control,
            },
            radii,
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
            state_colors,
            shadow_previews,
            stylesheet,
            palette,
            catalog,
            style_theme: StyleThemeSlot::default(),
        };
        let (metrics, control_sizes) = metrics_from_theme(&crate::style::theme_for_mode(&tokens))?;
        tokens.metrics = metrics;
        tokens.control_sizes = control_sizes;
        Ok(tokens)
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

    /// The common stylesheet SDK controls read (motion, icons and the other `[common]` families).
    /// It has no size-keyed values, so every Shadcn size shares it.
    pub fn common(&self) -> &xgpui_luma::theme::stylesheet::CommonStylesheet {
        &self.stylesheet.common
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

#[cfg(test)]
mod metric_tests {
    use xgpui_luma::theme::{MetricTokens, ThemeMode};

    use crate::{ShadcnLook, ShadcnSize};

    const RETRO_ARCADE: &str = include_str!("../tests/fixtures/tweakcn/retro-arcade.css");

    /// The control metrics native controls read are the look's `.lss` tokens for each size, in each
    /// mode: one source for `.lss` and native controls.
    #[test]
    fn control_metrics_are_the_look_lss_tokens_for_each_size() {
        let look = ShadcnLook::from_css_str(RETRO_ARCADE).expect("catalog");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let (tokens, theme) = (look.mode_tokens(), look.style_theme());
            for size in ShadcnSize::ALL {
                let lss = MetricTokens::from_theme(&theme, &[("size", size.key())]).expect("metrics");
                assert_eq!(tokens.control_sizes.get(size), lss.control, "{mode:?} {size:?}");
            }
            assert_eq!(tokens.metrics.control, tokens.control_sizes.md, "the shared scale is md");
        }
    }

    /// The metrics follow the catalog's `--spacing`.
    #[test]
    fn control_metrics_follow_the_catalog_spacing() {
        let look = ShadcnLook::from_css_str(RETRO_ARCADE).expect("catalog");
        let wider = RETRO_ARCADE.replace("--spacing: 0.25rem", "--spacing: 0.5rem");
        assert_ne!(wider, RETRO_ARCADE, "the fixture sets --spacing");
        let wider = ShadcnLook::from_css_str(&wider).expect("catalog");
        for size in ShadcnSize::ALL {
            let (base, wide) =
                (look.mode_tokens().control_sizes.get(size), wider.mode_tokens().control_sizes.get(size));
            assert!(wide.gap > base.gap && wide.padding_x > base.padding_x, "{size:?}");
        }
    }
}
