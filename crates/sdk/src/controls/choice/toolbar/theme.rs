use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarVariant {
    /// Bordered, filled shell.
    #[default]
    Outline,
    /// Transparent / borderless shell for denser embedding.
    Ghost,
}

#[derive(Clone, Debug)]
pub struct ToolbarLook {
    /// Shell stroke thickness in pixels.
    pub border_width: f32,
    /// Vertical separator thickness in pixels.
    pub separator_width: f32,
    pub background: Hsla,
    pub border: Hsla,
    pub separator: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub separator_height: f32,
}

pub trait ToolbarTheme: Send + Sync {
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook;
}

pub struct DefaultToolbarTheme {
    tokens: ThemeTokens,
}

impl Default for DefaultToolbarTheme {
    fn default() -> Self {
        Self::new(ThemeTokens::default())
    }
}

impl DefaultToolbarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

pub fn default_toolbar_theme() -> Arc<dyn ToolbarTheme> {
    static THEME: OnceLock<Arc<dyn ToolbarTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(DefaultToolbarTheme::default())).clone()
}

impl ToolbarTheme for DefaultToolbarTheme {
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
        let palette = self.tokens.stylesheet.recipes.palette("toolbar", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let control = metrics.control;
        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };

        let (background, border) = match variant {
            ToolbarVariant::Outline => {
                let background = if enabled {
                    palette.surface.subtle.background
                } else {
                    palette.state.disabled.background
                };
                (background, palette.border.default)
            }
            ToolbarVariant::Ghost => (transparent, transparent),
        };

        let mut look = ToolbarLook {
            border_width: metrics.border_width.default,
            separator_width: metrics.border_width.default,
            background,
            border,
            separator: palette.border.default,
            radius: 6.0,
            padding_x: metrics.spacing.s2,
            padding_y: metrics.spacing.s1,
            gap: control.gap,
            separator_height: control.height,
        };
        crate::theme::stylesheet::apply_common_geometry!(self.tokens.stylesheet.toolbar, ToolbarGeometry, look,
            [padding_x => padding_x, padding_y => padding_y, gap => gap, border_width => border_width, separator_width => separator_width, radius => radius]);
        look
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    ToolbarStylesheet,
    ToolbarGeometryStylesheet,
    ToolbarGeometry,
    ResolvedToolbarGeometry,
    "toolbar",
    [padding_x, padding_y, gap, border_width, separator_width, radius]
);
