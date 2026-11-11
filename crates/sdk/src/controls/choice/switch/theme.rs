use crate::theme::provenance::ResolvedMetric;
use crate::theme::stylesheet::{CommonStylesheet, PaletteRole, SwitchMotionStylesheet, TransitionSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::scales::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchScale {
    /// Whole-control opacity when disabled.
    pub disabled_opacity: f32,
    pub track_width: f32,
    pub track_height: f32,
    pub track_padding: f32,
    pub thumb_size: f32,
    pub track_radius: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
}

impl SwitchScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self::compute_with_stylesheet(
            size,
            metrics,
            scale_factor,
            crate::theme::stylesheet::CommonStylesheet::sdk_default(),
        )
    }

    /// Resolve a snapshot's shared dimensions without changing control behavior.
    pub fn compute_with_stylesheet(
        size: ControlSize,
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let key = size.stylesheet_key();
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().switch.authored_geometry(key);
        let recipe = stylesheet.switch.authored_geometry(key);
        // Authored scales are dimension / control height, preserving proportional
        // sizing when metric tokens change. Explicit dimensions take precedence below.
        let height = metrics.control_height(size);
        let fallback = Self {
            disabled_opacity: recipe.disabled_opacity.or(baseline.disabled_opacity).unwrap_or_default(),
            track_width: snap_to_pixel(
                height * recipe.track_width_scale.or(baseline.track_width_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_height: snap_to_pixel(
                height * recipe.track_height_scale.or(baseline.track_height_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_padding: snap_to_pixel(baseline.track_padding.unwrap_or_default(), scale_factor),
            thumb_size: snap_to_pixel(
                height * recipe.thumb_scale.or(baseline.thumb_scale).unwrap_or_default(),
                scale_factor,
            ),
            track_radius: metrics.radius.pill,
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: label_baseline_shift(size, stylesheet),
        };
        stylesheet.switch.resolve_geometry(size, fallback, scale_factor)
    }
}

#[derive(Clone, Debug)]
pub struct SwitchPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
}

pub type SwitchLook = SwitchPalette;

pub trait SwitchTheme: Send + Sync {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette;
    fn metrics(&self) -> MetricTokens;
    /// Look-owned geometry for `size`. Defaults to [`SwitchScale::compute`].
    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        SwitchScale::compute(size, &self.metrics(), scale_factor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSwitchTheme {
    tokens: ThemeTokens,
}

pub fn default_switch_theme() -> Arc<dyn SwitchTheme> {
    static THEME: OnceLock<Arc<dyn SwitchTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSwitchTheme::default())).clone()
}

impl DefaultSwitchTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SwitchTheme for DefaultSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette {
        let palette = self.tokens.stylesheet.recipes.palette("switch", &self.tokens.palette);
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let selected = palette.state.selected;

        let track_background = if state.disabled {
            palette.state.disabled.background
        } else if on {
            selected.background
        } else {
            palette.form.input.background
        };

        let track_border = if on && !state.disabled {
            track_background
        } else {
            palette.border.default
        };

        let (thumb_background, thumb_border) = if state.disabled {
            (palette.state.disabled.foreground, palette.state.disabled.background)
        } else if on {
            let thumb = selected.foreground;
            (thumb, thumb)
        } else {
            (palette.app.background, palette.border.default)
        };

        SwitchPalette {
            focus_border: palette.focus.ring,
            track_background,
            track_border,
            thumb_background,
            thumb_border,
            thumb_shadow: elevation.thumb.to_box_shadows(),
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
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        SwitchScale::compute_with_stylesheet(size, &self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}

/// Shared switch dimensions. Size fields override general geometry field by field.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchStylesheet {
    pub geometry: SwitchGeometryStylesheet,
    /// Semantic size keys belong to the look (for example Radix "1"/"2"/"3").
    pub sizes: crate::theme::ControlSizes<SwitchGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchGeometryStylesheet {
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_unit")]
    pub disabled_opacity: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_width: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_height: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_padding: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub thumb_size: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub gap: Option<f32>,
    /// Track width divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_width_scale: Option<f32>,
    /// Track height divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_height_scale: Option<f32>,
    /// Thumb size divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub thumb_scale: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSwitchGeometry {
    pub disabled_opacity: ResolvedMetric,
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
}

crate::theme::stylesheet::geometry::inherit_geometry!(
    SwitchStylesheet,
    sizes,
    [
        track_width,
        track_height,
        track_padding,
        thumb_size,
        gap,
        track_width_scale,
        track_height_scale,
        thumb_scale,
        disabled_opacity
    ]
);

impl SwitchStylesheet {
    /// Construct the SDK switch geometry without parsing bundled data.
    pub fn native_default() -> Self {
        let mut switch = Self::default();
        switch
            .disabled_opacity(0.56)
            .track_padding(2.0)
            .size("sm", |m| {
                m.scales(32.0 / 28.0, 13.0 / 28.0, 9.0 / 28.0);
            })
            .size("md", |m| {
                m.scales(40.0 / 36.0, 18.0 / 36.0, 14.0 / 36.0);
            })
            .size("lg", |m| {
                m.scales(48.0 / 44.0, 24.0 / 44.0, 20.0 / 44.0);
            });
        switch
    }

    /// Configure general switch disabled opacity (0.0 to 1.0).
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    /// Configure general switch track width in logical pixels.
    pub fn track_width(&mut self, width: f32) -> &mut Self {
        self.geometry.track_width = Some(width);
        self
    }

    /// Configure general switch track height in logical pixels.
    pub fn track_height(&mut self, height: f32) -> &mut Self {
        self.geometry.track_height = Some(height);
        self
    }

    /// Configure general switch track padding in logical pixels.
    pub fn track_padding(&mut self, padding: f32) -> &mut Self {
        self.geometry.track_padding = Some(padding);
        self
    }

    /// Configure general switch thumb size in logical pixels.
    pub fn thumb_size(&mut self, thumb: f32) -> &mut Self {
        self.geometry.thumb_size = Some(thumb);
        self
    }

    /// Configure general switch label gap in logical pixels.
    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.geometry.gap = Some(gap);
        self
    }

    /// Configure general switch track width scale relative to control height.
    pub fn track_width_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.track_width_scale = Some(scale);
        self
    }

    /// Configure general switch track height scale relative to control height.
    pub fn track_height_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.track_height_scale = Some(scale);
        self
    }

    /// Configure general switch thumb scale relative to control height.
    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.thumb_scale = Some(scale);
        self
    }

    /// Configure or override a size key without manual map manipulation.
    pub fn size(&mut self, key: impl Into<String>, configure: impl FnOnce(&mut SwitchGeometryStylesheet)) -> &mut Self {
        let key = key.into();
        let entry = self.sizes.get_or_default(key.as_str());
        configure(entry);
        self
    }

    /// Apply authored switch fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }

    /// Resolve optional dimensions and proportional recipes over general fields.
    pub fn authored_geometry(&self, key: &str) -> SwitchGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SwitchGeometryStylesheet {
            disabled_opacity: field!(disabled_opacity),
            track_width: field!(track_width),
            track_height: field!(track_height),
            track_padding: field!(track_padding),
            thumb_size: field!(thumb_size),
            gap: field!(gap),
            track_width_scale: field!(track_width_scale),
            track_height_scale: field!(track_height_scale),
            thumb_scale: field!(thumb_scale),
        }
    }

    /// Resolve concrete dimensions without allocating inspector source strings.
    pub fn resolve_geometry<'a>(
        &self,
        key: impl Into<crate::theme::SizeKey<'a>>,
        fallback: SwitchScale,
        scale_factor: f32,
    ) -> SwitchScale {
        let authored = self.authored_geometry(key.into().as_str());
        let pixel = |value: Option<f32>, baseline| value.map(|v| snap_to_pixel(v, scale_factor)).unwrap_or(baseline);
        SwitchScale {
            disabled_opacity: authored.disabled_opacity.unwrap_or(fallback.disabled_opacity),
            track_width: pixel(authored.track_width, fallback.track_width),
            track_height: pixel(authored.track_height, fallback.track_height),
            track_padding: pixel(authored.track_padding, fallback.track_padding),
            thumb_size: pixel(authored.thumb_size, fallback.thumb_size),
            gap: pixel(authored.gap, fallback.gap),
            ..fallback
        }
    }

    /// Resolve per-size, general, then supplied SDK/legacy fallback geometry.
    /// Authored values are snapped once at the requested display scale factor.
    pub fn inspect_geometry(
        &self,
        key: &str,
        fallback: crate::controls::switch::SwitchScale,
        scale_factor: f32,
    ) -> ResolvedSwitchGeometry {
        use crate::theme::snap_to_pixel;
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        let resolve = |field: &str, specific: Option<f32>, general: Option<f32>, fallback: f32| {
            if let Some(value) = specific {
                ResolvedMetric::authored(
                    snap_to_pixel(value, scale_factor),
                    format!("common.switch.sizes.{key}.{field}"),
                )
            } else if let Some(value) = general {
                ResolvedMetric::authored(snap_to_pixel(value, scale_factor), format!("common.switch.geometry.{field}"))
            } else {
                ResolvedMetric::constant(fallback, format!("switch {field} fallback"))
            }
        };
        let disabled_opacity = if let Some(value) = specific.disabled_opacity {
            ResolvedMetric::authored(value, format!("common.switch.sizes.{key}.disabled_opacity"))
        } else if let Some(value) = self.geometry.disabled_opacity {
            ResolvedMetric::authored(value, "common.switch.geometry.disabled_opacity")
        } else {
            ResolvedMetric::constant(fallback.disabled_opacity, "switch disabled opacity fallback")
        };
        ResolvedSwitchGeometry {
            disabled_opacity,
            track_width: resolve("track_width", specific.track_width, self.geometry.track_width, fallback.track_width),
            track_height: resolve(
                "track_height",
                specific.track_height,
                self.geometry.track_height,
                fallback.track_height,
            ),
            track_padding: resolve(
                "track_padding",
                specific.track_padding,
                self.geometry.track_padding,
                fallback.track_padding,
            ),
            thumb_size: resolve("thumb_size", specific.thumb_size, self.geometry.thumb_size, fallback.thumb_size),
            gap: resolve("gap", specific.gap, self.geometry.gap, fallback.gap),
        }
    }
}

impl SwitchGeometryStylesheet {
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    pub fn track_width(&mut self, width: f32) -> &mut Self {
        self.track_width = Some(width);
        self
    }

    pub fn track_height(&mut self, height: f32) -> &mut Self {
        self.track_height = Some(height);
        self
    }

    pub fn track_padding(&mut self, padding: f32) -> &mut Self {
        self.track_padding = Some(padding);
        self
    }

    pub fn thumb_size(&mut self, thumb: f32) -> &mut Self {
        self.thumb_size = Some(thumb);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.gap = Some(gap);
        self
    }

    pub fn track_width_scale(&mut self, scale: f32) -> &mut Self {
        self.track_width_scale = Some(scale);
        self
    }

    pub fn track_height_scale(&mut self, scale: f32) -> &mut Self {
        self.track_height_scale = Some(scale);
        self
    }

    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.thumb_scale = Some(scale);
        self
    }

    /// Convenience setter for track width, track height, and thumb size in logical pixels.
    pub fn dimensions(&mut self, track_width: f32, track_height: f32, thumb_size: f32) -> &mut Self {
        self.track_width(track_width);
        self.track_height(track_height);
        self.thumb_size(thumb_size);
        self
    }

    /// Convenience setter for track width scale, track height scale, and thumb scale.
    pub fn scales(&mut self, track_width_scale: f32, track_height_scale: f32, thumb_scale: f32) -> &mut Self {
        self.track_width_scale(track_width_scale);
        self.track_height_scale(track_height_scale);
        self.thumb_scale(thumb_scale);
        self
    }
}

impl ResolvedSwitchGeometry {
    /// Preserve the look-owned radius and baseline shift while supplying common inputs.
    pub fn apply_to(&self, mut scale: crate::controls::switch::SwitchScale) -> crate::controls::switch::SwitchScale {
        scale.disabled_opacity = self.disabled_opacity.value_px;
        scale.track_width = self.track_width.value_px;
        scale.track_height = self.track_height.value_px;
        scale.track_padding = self.track_padding.value_px;
        scale.thumb_size = self.thumb_size.value_px;
        scale.gap = self.gap.value_px;
        scale
    }
}

impl SwitchMotionStylesheet {
    /// Construct the SDK switch motion without parsing bundled data.
    pub fn native_default() -> Self {
        Self { state: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct SDK switch palette references without parsing bundled data.
pub fn native_sdk_switch_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    [
        AppBackground,
        AppForeground,
        BorderDefault,
        FocusRing,
        FormInputBackground,
        StateDisabledBackground,
        StateDisabledForeground,
        StateSelectedBackground,
        StateSelectedForeground,
    ]
    .into_iter()
    .map(|role| (role, role))
    .collect()
}

/// Install the native SDK switch baseline.
pub fn define_sdk_switch(sheet: &mut CommonStylesheet) {
    sheet.switch = SwitchStylesheet::native_default();
    sheet.motion.switch = SwitchMotionStylesheet::native_default();
    sheet.recipes.0.insert("switch".into(), native_sdk_switch_recipes());
}
