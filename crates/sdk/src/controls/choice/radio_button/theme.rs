use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::scales::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioScale {
    /// Whole-control opacity when disabled.
    pub disabled_opacity: f32,
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub height: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
    pub dot_size: f32,
}

impl RadioScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self::compute_with_stylesheet(
            size,
            metrics,
            scale_factor,
            crate::theme::stylesheet::CommonStylesheet::sdk_default(),
        )
    }

    /// Resolve shared appearance over the vanilla SDK baseline, then snap once.
    pub fn compute_with_stylesheet(
        size: ControlSize,
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let key = size.stylesheet_key();
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().radio.authored_geometry(key);
        let geometry = stylesheet.radio.authored_geometry(key).with_fallback(&baseline);
        let indicator_size = snap_to_pixel(
            geometry
                .indicator_size
                .unwrap_or_else(|| metrics.control_height(size) * geometry.indicator_scale.unwrap_or_default()),
            scale_factor,
        );

        Self {
            disabled_opacity: geometry.disabled_opacity.unwrap_or_default(),
            control_radius: metrics.radius(size),
            control_padding_x: snap_to_pixel(geometry.control_padding_x.unwrap_or_default(), scale_factor),
            control_padding_y: snap_to_pixel(geometry.control_padding_y.unwrap_or_default(), scale_factor),
            indicator_size,
            height: snap_to_pixel(metrics.control_height(size), scale_factor),
            gap: snap_to_pixel(geometry.gap.unwrap_or_else(|| metrics.gap(size)), scale_factor),
            label_baseline_shift: label_baseline_shift(size, stylesheet),
            dot_size: snap_to_pixel(
                geometry
                    .dot_size
                    .unwrap_or_else(|| metrics.control_height(size) * geometry.dot_scale.unwrap_or_default()),
                scale_factor,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RadioButtonPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub control_background: Option<Hsla>,
    /// Body-border paint; `None` is transparent and retains the configured border width.
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub dot_color: Hsla,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
    pub indicator_shadow: Option<Vec<BoxShadow>>,
}

pub type RadioButtonLook = RadioButtonPalette;

pub trait RadioButtonTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> RadioButtonPalette;
    fn metrics(&self) -> MetricTokens;

    /// Geometry for the indicator / label row. Defaults to [`RadioScale::compute`].
    fn scale(&self, size: ControlSize, scale_factor: f32) -> RadioScale {
        RadioScale::compute(size, &self.metrics(), scale_factor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultRadioButtonTheme {
    tokens: ThemeTokens,
}

pub fn default_radio_button_theme() -> Arc<dyn RadioButtonTheme> {
    static THEME: OnceLock<Arc<dyn RadioButtonTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultRadioButtonTheme::default())).clone()
}

impl DefaultRadioButtonTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl RadioButtonTheme for DefaultRadioButtonTheme {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> RadioButtonPalette {
        let palette = self.tokens.stylesheet.recipes.palette("radio_button", &self.tokens.palette);
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let selected = palette.state.selected;

        let indicator_background = match layer {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.form.input.background,
        };

        let selected_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => selected.background,
            InteractionLayer::Default => selected.background,
        };

        RadioButtonPalette {
            focus_border: palette.focus.ring,
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if checked && !state.disabled {
                selected_color
            } else {
                palette.form.input.border
            },
            dot_color: if state.disabled {
                palette.state.disabled.foreground
            } else if checked {
                selected.foreground
            } else {
                palette.app.foreground
            },
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            label_typography: {
                let mut label_typography = typography.text.label;
                crate::controls::textfield::apply_control_size_typography(&mut label_typography, typography, size);
                label_typography
            },
            label_font_family: typography.font.sans.family.clone().into(),
            indicator_shadow: None,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> RadioScale {
        RadioScale::compute_with_stylesheet(size, &self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    RadioStylesheet,
    RadioGeometryStylesheet,
    RadioGeometry,
    ResolvedRadioGeometry,
    "radio",
    [indicator_size, indicator_scale, dot_size, dot_scale, control_padding_x, control_padding_y, gap],
    unit[disabled_opacity]
);

impl RadioStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: RadioGeometryStylesheet {
                disabled_opacity: Some(0.56),
                control_padding_x: Some(0.0),
                control_padding_y: Some(0.0),
                ..Default::default()
            },
            sizes: [("sm", 0.45), ("md", 0.5), ("lg", 0.55)]
                .into_iter()
                .map(|(key, scale)| {
                    (
                        key.into(),
                        RadioGeometryStylesheet {
                            indicator_scale: Some(scale),
                            dot_scale: Some(0.24),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}
