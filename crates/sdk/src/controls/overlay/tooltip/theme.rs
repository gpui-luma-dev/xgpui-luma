use std::sync::Arc;
use gpui::{BoxShadow, Hsla, SharedString};
use crate::theme::ThemeTokens;

/// Shared tooltip placement and pointer dimensions, in logical pixels.
#[derive(Clone, Copy)]
pub struct TooltipPlacementGeometry {
    pub gap: f32,
    pub margin: f32,
    pub pointer_size: f32,
    pub pointer_inset: f32,
}
impl Default for TooltipPlacementGeometry {
    /// The SDK's placement, from its `tooltip.lss`.
    fn default() -> Self {
        super::styled::styled_tooltip_look(&crate::style::StyleBinding::sdk("tooltip")).placement
    }
}

/// Resolved tooltip style. Looks own all ordinary presentation defaults.
#[derive(Clone)]
pub struct TooltipLook {
    pub placement: TooltipPlacementGeometry,
    /// Shortcut spacing, typography and de-emphasis from the shared stylesheet.
    pub shortcut_gap: f32,
    pub shortcut_font_size: f32,
    pub shortcut_opacity: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}

pub trait TooltipTheme: Send + Sync {
    fn resolve(&self) -> TooltipLook;
}

impl TooltipTheme for TooltipLook {
    fn resolve(&self) -> TooltipLook {
        self.clone()
    }
}

impl TooltipLook {
    /// A tooltip with the SDK's `tooltip.lss` geometry and `tokens`' palette and menu shadow.
    pub fn from_tokens(tokens: &ThemeTokens) -> Self {
        let geometry = super::styled::styled_tooltip_look(&crate::style::StyleBinding::sdk("tooltip"));
        let palette = tokens.stylesheet.recipes.palette("tooltip", &tokens.palette);
        Self {
            background: palette.surface.floating.background,
            foreground: palette.surface.floating.foreground,
            shadow: tokens.elevation.menu.to_box_shadows(),
            ..geometry
        }
    }
}

/// The SDK's tooltip theme: `tooltip.lss` alone.
pub fn default_tooltip_theme() -> Arc<dyn TooltipTheme> {
    Arc::new(super::styled::StyledTooltipTheme::sdk())
}

/// Presentation-only input, resolved from the explicitly bound draft look on every frame.
#[derive(Clone)]
pub struct TooltipRenderModel {
    pub placement: TooltipPlacementGeometry,
    pub shortcut_gap: f32,
    pub shortcut_font_size: f32,
    pub shortcut_opacity: f32,
    pub text: SharedString,
    pub shortcut: Option<SharedString>,
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}
