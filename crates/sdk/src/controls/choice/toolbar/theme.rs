use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use xgpui_luma_style::{ResolveCx, State, StateSet, Style};

use super::style::{ResolvedToolbarSeparatorPart, ResolvedToolbarStyle, ToolbarStyle};
use crate::theme::ThemeTokens;
use crate::theme::style_tokens::ThemeTokenSource;

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

    /// Everything the template reads (issue #282). The default adapts
    /// [`resolve`](Self::resolve), so themes written before `ToolbarStyle` keep working.
    fn resolved_style(&self, enabled: bool, variant: ToolbarVariant) -> ResolvedToolbarStyle {
        resolved_from_look(&self.resolve(enabled, variant), variant)
    }
}

/// Build a resolved style from the older look. Only `Outline` draws a border.
pub fn resolved_from_look(look: &ToolbarLook, variant: ToolbarVariant) -> ResolvedToolbarStyle {
    ResolvedToolbarStyle {
        background: look.background,
        border: (variant == ToolbarVariant::Outline).then_some(look.border),
        border_width: look.border_width,
        radius: look.radius,
        padding_x: look.padding_x,
        padding_y: look.padding_y,
        gap: look.gap,
        separator: ResolvedToolbarSeparatorPart {
            color: look.separator,
            width: look.separator_width,
            height: look.separator_height,
        },
    }
}

pub struct DefaultToolbarTheme {
    tokens: ThemeTokens,
    outline: ToolbarStyle,
    ghost: ToolbarStyle,
}

impl Default for DefaultToolbarTheme {
    fn default() -> Self {
        Self::new(ThemeTokens::default())
    }
}

impl DefaultToolbarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        let outline = ToolbarStyle::from_stylesheet(&tokens.stylesheet);
        let ghost = outline.ghost();
        Self { tokens, outline, ghost }
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
            border_width: metrics.border_width,
            separator_width: metrics.border_width,
            background,
            border,
            separator: palette.border.default,
            radius: 6.0,
            padding_x: 6.0,
            padding_y: 4.0,
            gap: control.gap,
            separator_height: control.height,
        };
        crate::theme::stylesheet::apply_common_geometry!(self.tokens.stylesheet.toolbar, ToolbarGeometry, look,
            [padding_x => padding_x, padding_y => padding_y, gap => gap, border_width => border_width, separator_width => separator_width, radius => radius]);
        look
    }

    fn resolved_style(&self, enabled: bool, variant: ToolbarVariant) -> ResolvedToolbarStyle {
        let style = match variant {
            ToolbarVariant::Outline => &self.outline,
            ToolbarVariant::Ghost => &self.ghost,
        };
        let at = if enabled {
            StateSet::NONE
        } else {
            StateSet::from(State::Disabled)
        };
        let tokens = ThemeTokenSource::for_family(&self.tokens, "toolbar");
        style.resolve(&ResolveCx::new(&tokens, 1.0), at)
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
