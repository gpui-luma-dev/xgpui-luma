use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, hsla};

use super::PopupMenuTriggerStyle;
use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

#[derive(Clone, Debug)]
pub struct PopupMenuPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub trigger_focus_border: gpui::Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Debug)]
pub struct PopupMenuLook {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub trigger_focus_border: gpui::Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_offset_y: f32,
    pub floating_menu: FloatingMenuLook,
}

impl PopupMenuLook {
    /// Resolve common trigger geometry before explicit consumer overrides.
    /// Authored lengths snap at the supplied display scale; fonts retain logical units.
    pub fn with_stylesheet(
        mut self,
        stylesheet: &crate::theme::stylesheet::PopupMenuStylesheet,
        key: &str,
        scale_factor: f32,
    ) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().popup_menu.authored_geometry(key);
        let recipe = stylesheet.authored_geometry(key).with_fallback(&baseline);
        self.menu_offset_y = recipe.gap.unwrap_or(self.trigger_gap) * recipe.menu_offset_scale.unwrap_or_default();
        crate::theme::stylesheet::apply_common_geometry!(stylesheet, key, PopupMenuGeometry, self,
            [height => trigger_height, padding_x => trigger_padding_x, padding_y => trigger_padding_y,
             gap => trigger_gap, icon_size => trigger_icon_size, radius => trigger_radius,
             font_size => trigger_typography.size, line_height => trigger_typography.line_height,
             menu_offset_y => menu_offset_y], defaults = Default::default());
        self.trigger_height = crate::theme::snap_to_pixel(self.trigger_height, scale_factor);
        self.trigger_padding_x = crate::theme::snap_to_pixel(self.trigger_padding_x, scale_factor);
        self.trigger_padding_y = crate::theme::snap_to_pixel(self.trigger_padding_y, scale_factor);
        self.trigger_gap = crate::theme::snap_to_pixel(self.trigger_gap, scale_factor);
        self.trigger_icon_size = crate::theme::snap_to_pixel(self.trigger_icon_size, scale_factor);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PopupMenuTriggerMetrics {
    pub size: ControlSize,
    pub menu_size: ControlSize,
    pub without_elevation: bool,
    pub icon_only: bool,
    pub trigger_radius_override: Option<f32>,
}

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> PopupMenuLook {
        let scale = StandardBoxScale::compute(metrics.size, &self.metrics(), scale_factor);
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale);
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPopupMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPopupMenuTheme::default())).clone()
}

impl DefaultPopupMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }

    fn styled_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
    ) -> PopupMenuLook {
        let scale = StandardBoxScale::compute(metrics.size, &self.tokens.metrics, scale_factor);
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale).with_stylesheet(
            &self.tokens.stylesheet.popup_menu,
            metrics.size.stylesheet_key(),
            scale_factor,
        );
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

impl PopupMenuTheme for DefaultPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let palette = self.tokens.stylesheet.recipes.palette("popup_menu", &self.tokens.palette);
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let (trigger_background, trigger_foreground, trigger_border, trigger_shadow) = match trigger_style {
            PopupMenuTriggerStyle::Primary | PopupMenuTriggerStyle::Secondary => {
                let background = match layer {
                    InteractionLayer::Disabled => palette.state.disabled.background,
                    InteractionLayer::Pressed => palette.state.pressed.background,
                    InteractionLayer::Hovered => palette.state.hover.background,
                    InteractionLayer::Default => palette.surface.subtle.background,
                };
                (
                    background,
                    if state.disabled {
                        palette.state.disabled.foreground
                    } else {
                        palette.app.foreground
                    },
                    Some(palette.border.default),
                    Some(self.tokens.elevation.control.to_box_shadows()),
                )
            }
            PopupMenuTriggerStyle::Outline => {
                let background = match layer {
                    InteractionLayer::Disabled => palette.state.disabled.background,
                    InteractionLayer::Pressed => palette.state.pressed.background,
                    InteractionLayer::Hovered => palette.state.hover.background,
                    InteractionLayer::Default => palette.app.background,
                };
                (
                    background,
                    if state.disabled {
                        palette.state.disabled.foreground
                    } else {
                        palette.app.foreground
                    },
                    Some(palette.border.default),
                    Some(self.tokens.elevation.control.to_box_shadows()),
                )
            }
            PopupMenuTriggerStyle::Ghost => {
                let background = match layer {
                    InteractionLayer::Disabled => palette.state.disabled.background,
                    InteractionLayer::Pressed | InteractionLayer::Hovered => palette.state.hover.background,
                    InteractionLayer::Default => hsla(0.0, 0.0, 0.0, 0.0),
                };
                (
                    background,
                    if state.disabled {
                        palette.state.disabled.foreground
                    } else {
                        palette.app.foreground
                    },
                    None,
                    None,
                )
            }
        };

        let trigger_shadow = if metrics.without_elevation {
            None
        } else {
            trigger_shadow
        };

        PopupMenuPalette {
            trigger_focus_border: palette.focus.ring,
            trigger_background,
            trigger_foreground,
            trigger_border,
            trigger_shadow,
            trigger_typography: typography.text.label,
            floating_menu: default_floating_menu_look(&self.tokens, metrics.menu_size),
        }
    }

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> PopupMenuLook {
        self.styled_look(trigger_style, metrics, state, scale_factor)
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub fn compose_popup_menu_look(palette: &PopupMenuPalette, scale: &StandardBoxScale) -> PopupMenuLook {
    PopupMenuLook {
        trigger_focus_border: palette.trigger_focus_border,
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.icon_size,
        menu_offset_y: scale.gap
            * crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .popup_menu
                .geometry
                .menu_offset_scale
                .unwrap_or_default(),
        floating_menu: palette.floating_menu.clone(),
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use crate::{
        color::GamutMapping,
        theme::{ThemeMode, stylesheet::CommonStylesheet},
    };

    #[test]
    fn sdk_popup_geometry_uses_snapshot_and_explicit_radius_wins() {
        let sheet = CommonStylesheet::resolve_layers(&[r#"
[common.popup_menu.geometry]
height=51
padding_x=0
padding_y=3
gap=7
icon_size=22
radius=11
font_size=19
line_height=27
menu_offset_scale="math(3 / 2)"
[common.popup_menu.sizes.sm]
menu_offset_y=0
"#])
        .unwrap();
        let theme =
            DefaultPopupMenuTheme::new(sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap());
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let metrics = PopupMenuTriggerMetrics { size, trigger_radius_override: Some(0.0), ..Default::default() };
            let look = theme.styled_look(PopupMenuTriggerStyle::Outline, metrics, Default::default(), 1.0);
            assert_eq!(
                (look.trigger_height, look.trigger_padding_x, look.trigger_icon_size, look.trigger_radius),
                (51.0, 0.0, 22.0, 0.0)
            );
            assert_eq!((look.trigger_typography.size, look.trigger_typography.line_height), (19.0, 27.0));
            // The proportional offset uses the resolved gap; an explicit offset wins.
            assert_eq!(look.menu_offset_y, if size == ControlSize::Sm { 0.0 } else { 7.0 * 1.5 });
        }
    }

    #[test]
    fn default_popup_metrics_retain_fractional_offsets_at_display_scales() {
        let theme = DefaultPopupMenuTheme::default();
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            for factor in [1.0, 1.25, 1.5, 2.0] {
                let scale = StandardBoxScale::compute(size, &theme.metrics(), factor);
                let look = theme.styled_look(
                    PopupMenuTriggerStyle::Outline,
                    PopupMenuTriggerMetrics { size, ..Default::default() },
                    Default::default(),
                    factor,
                );
                assert_eq!(
                    (look.trigger_height, look.trigger_padding_x, look.trigger_icon_size),
                    (scale.height, scale.padding_x, scale.icon_size)
                );
                assert_eq!(look.menu_offset_y, scale.gap * 0.5);
            }
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    PopupMenuStylesheet,
    PopupMenuGeometryStylesheet,
    PopupMenuGeometry,
    ResolvedPopupMenuGeometry,
    "popup_menu",
    [
        height,
        padding_x,
        padding_y,
        gap,
        font_size,
        line_height,
        icon_size,
        radius,
        menu_offset_y,
        menu_offset_scale
    ]
);

impl PopupMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PopupMenuGeometryStylesheet { menu_offset_scale: Some(1.0 / 2.0), ..Default::default() },
            ..Default::default()
        }
    }
}
