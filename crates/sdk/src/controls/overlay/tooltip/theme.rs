#![allow(deprecated)]

use std::sync::Arc;
use gpui::{BoxShadow, Hsla, SharedString};
use crate::style::{PartRead, StyleBinding};
use crate::theme::ThemeTokens;

use super::styled::resolve_tooltip;

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
        Self::from_resolved(&resolve_tooltip(&StyleBinding::sdk("tooltip")))
    }
}

impl TooltipPlacementGeometry {
    /// The placement `resolved` (a tooltip's resolved style, see [`resolve_tooltip`]) describes (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let root = resolved.root();
        Self {
            gap: root.number("target_gap"),
            margin: root.number("window_margin"),
            pointer_size: root.number("pointer_size"),
            pointer_inset: root.number("pointer_inset"),
        }
    }
}

/// Resolved tooltip style. Looks own all ordinary presentation defaults.
#[deprecated(note = "the tooltip draws from `TooltipTheme::style()`; resolve it with `resolve_tooltip` (#433)")]
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
    /// The tooltip's style (#433): the binding the tooltip resolves (see [`resolve_tooltip`]) and draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("tooltip")
    }

    /// The tooltip's look, built from [`Self::style`] for code that still reads a [`TooltipLook`].
    #[deprecated(note = "the tooltip draws from `style()`; resolve it with `resolve_tooltip` (#433)")]
    fn resolve(&self) -> TooltipLook {
        TooltipLook::from_resolved(&resolve_tooltip(&self.style()))
    }
}

impl TooltipLook {
    /// The look `resolved` (a tooltip's resolved style, see [`resolve_tooltip`]) describes: the compatibility path
    /// for code that still reads a `TooltipLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let root = resolved.root();
        let typography = root.typography(14.0, 20.0);
        Self {
            placement: TooltipPlacementGeometry::from_resolved(resolved),
            shortcut_gap: root.number("shortcut_gap"),
            shortcut_font_size: root.number("shortcut_font_size"),
            shortcut_opacity: root.number("shortcut_opacity"),
            background: root.color("background"),
            foreground: root.color("foreground"),
            padding: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            radius: root.number("radius"),
            max_width: root.number("max_width"),
            text_size: typography.size,
            line_height: typography.line_height,
            shadow: root.box_shadow(),
        }
    }

    /// A tooltip with the SDK's `tooltip.lss` geometry and `tokens`' palette and menu shadow.
    pub fn from_tokens(tokens: &ThemeTokens) -> Self {
        let geometry = Self::from_resolved(&resolve_tooltip(&StyleBinding::sdk("tooltip")));
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

impl TooltipRenderModel {
    /// The bubble for `text` (and `shortcut`) drawn from `resolved`, a tooltip's resolved style (#433; see
    /// [`resolve_tooltip`]).
    pub fn from_resolved(
        resolved: &xgpui_luma_style::Resolved,
        text: SharedString,
        shortcut: Option<SharedString>,
    ) -> Self {
        let root = resolved.root();
        let typography = root.typography(14.0, 20.0);
        Self {
            placement: TooltipPlacementGeometry::from_resolved(resolved),
            shortcut_gap: root.number("shortcut_gap"),
            shortcut_font_size: root.number("shortcut_font_size"),
            shortcut_opacity: root.number("shortcut_opacity"),
            text,
            shortcut,
            background: root.color("background"),
            foreground: root.color("foreground"),
            padding: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            radius: root.number("radius"),
            max_width: root.number("max_width"),
            text_size: typography.size,
            line_height: typography.line_height,
            shadow: root.box_shadow(),
        }
    }
}
