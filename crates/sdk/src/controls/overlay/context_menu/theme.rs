use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuLook {
    /// Target stroke thickness in pixels.
    pub border_width: f32,
    /// Whole-target opacity when disabled.
    pub disabled_opacity: f32,
    /// Minimum popup distance from window edges in pixels.
    pub window_margin: f32,
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub floating_menu: FloatingMenuLook,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultContextMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledContextMenuTheme::sdk())).clone()
}

impl DefaultContextMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ContextMenuTheme for DefaultContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        let palette = self.tokens.stylesheet.recipes.palette("context_menu", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;

        let target_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.app.background,
        };
        let target_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().context_menu.authored_geometry();
        let recipe = self.tokens.stylesheet.context_menu.authored_geometry().with_fallback(&baseline);
        let mut look = ContextMenuLook {
            border_width: recipe.border_width.unwrap_or_default(),
            disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
            window_margin: recipe.window_margin.unwrap_or_default(),
            target_background,
            target_foreground,
            target_border: palette.border.default,
            target_typography: typography.text.label,
            target_radius: metrics.radius(),
            target_padding_x: metrics.padding_x(),
            target_padding_y: metrics.padding_y(),
            target_min_width: 0.0,
            floating_menu: default_floating_menu_look(&self.tokens),
        };
        crate::theme::stylesheet::apply_common_geometry!(self.tokens.stylesheet.context_menu, ContextMenuGeometry, look,
            [min_width => target_min_width, padding_x => target_padding_x, padding_y => target_padding_y,
             font_size => target_typography.size, line_height => target_typography.line_height], defaults = Default::default());
        look
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    ContextMenuStylesheet,
    ContextMenuGeometryStylesheet,
    ContextMenuGeometry,
    ResolvedContextMenuGeometry,
    "context_menu",
    [min_width, padding_x, padding_y, font_size, line_height, border_width, window_margin],
    unit[disabled_opacity]
);

impl ContextMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ContextMenuGeometryStylesheet {
                border_width: Some(1.0),
                window_margin: Some(8.0),
                disabled_opacity: Some(0.56),
                min_width: Some(200.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
