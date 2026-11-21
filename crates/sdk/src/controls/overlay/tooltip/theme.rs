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
impl From<&crate::theme::stylesheet::TooltipGeometry> for TooltipPlacementGeometry {
    fn from(value: &crate::theme::stylesheet::TooltipGeometry) -> Self {
        Self {
            gap: value.target_gap,
            margin: value.window_margin,
            pointer_size: value.pointer_size,
            pointer_inset: value.pointer_inset,
        }
    }
}
impl Default for TooltipPlacementGeometry {
    fn default() -> Self {
        (&crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .tooltip
            .resolve_geometry(Default::default()))
            .into()
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

impl TooltipLook {
    /// Shared compact geometry for look adapters. Colors and corner radius remain
    /// look-owned; adapters may override any resolved field before returning it.
    pub fn compact(background: Hsla, foreground: Hsla, radius: f32) -> Self {
        let geometry = crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .tooltip
            .resolve_geometry_for("compact", crate::theme::stylesheet::TooltipGeometry::default());
        Self {
            placement: (&geometry).into(),
            shortcut_gap: geometry.shortcut_gap,
            shortcut_font_size: geometry.shortcut_font_size,
            shortcut_opacity: geometry.shortcut_opacity,
            background,
            foreground,
            radius,
            padding: geometry.padding,
            padding_y: geometry.padding_y,
            max_width: geometry.max_width,
            text_size: geometry.font_size,
            line_height: geometry.line_height,
            shadow: Vec::new(),
        }
    }
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
    /// Capture complete tooltip styling from a resolved SDK/common token snapshot.
    /// `key` selects a look-defined tooltip size; `None` uses the general geometry.
    pub fn from_tokens(tokens: &ThemeTokens, key: Option<&str>) -> Self {
        let fallback = crate::theme::stylesheet::TooltipGeometry::default();
        let tooltip = &tokens.stylesheet.tooltip;
        let geometry = match key {
            Some(key) => tooltip.resolve_geometry_for(key, fallback),
            None => tooltip.resolve_geometry(fallback),
        };
        let palette = tokens.stylesheet.recipes.palette("tooltip", &tokens.palette);
        Self {
            placement: (&geometry).into(),
            shortcut_gap: geometry.shortcut_gap,
            shortcut_font_size: geometry.shortcut_font_size,
            shortcut_opacity: geometry.shortcut_opacity,
            background: palette.surface.floating.background,
            foreground: palette.surface.floating.foreground,
            padding: geometry.padding,
            padding_y: geometry.padding_y,
            radius: geometry.radius,
            max_width: geometry.max_width,
            text_size: geometry.font_size,
            line_height: geometry.line_height,
            shadow: tokens.elevation.menu.to_box_shadows(),
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

crate::theme::stylesheet::geometry::geometry_family!(
    TooltipStylesheet,
    TooltipGeometryStylesheet,
    TooltipGeometry,
    ResolvedTooltipGeometry,
    "tooltip",
    [
        target_gap,
        window_margin,
        pointer_size,
        pointer_inset,
        padding,
        padding_y,
        max_width,
        font_size,
        line_height,
        shortcut_gap,
        shortcut_font_size,
        radius
    ],
    unit[shortcut_opacity]
);

impl TooltipStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TooltipGeometryStylesheet {
                target_gap: Some(10.0),
                window_margin: Some(8.0),
                pointer_size: Some(6.0),
                pointer_inset: Some(12.0),
                shortcut_gap: Some(16.0),
                shortcut_font_size: Some(12.0),
                shortcut_opacity: Some(0.65),
                padding: Some(12.0),
                padding_y: Some(8.0),
                max_width: Some(260.0),
                radius: Some(6.0),
                font_size: Some(14.0),
                line_height: Some(20.0),
            },
            sizes: crate::theme::ControlSizes::from([(
                "compact".into(),
                TooltipGeometryStylesheet {
                    padding: Some(8.0),
                    padding_y: Some(4.0),
                    font_size: Some(12.0),
                    line_height: Some(16.0),
                    ..Default::default()
                },
            )]),
        }
    }
}
