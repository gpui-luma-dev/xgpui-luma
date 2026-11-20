use crate::theme::provenance::{MetricSource, ResolvedMetric};
use crate::theme::stylesheet::{CommonStylesheet, PaletteRole, SliderMotionStylesheet, TransitionSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSizes, InteractionLayer, InteractionState, MetricTokens, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SliderLook {
    pub thumb_border_width: f32,
    /// Captured radial layout and disabled-track presentation.
    pub decoration: crate::theme::stylesheet::SliderDecoration,
    pub track_background: Hsla,
    pub fill_background: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

pub trait SliderTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> SliderLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSliderTheme {
    tokens: ThemeTokens,
}

pub fn default_slider_theme() -> Arc<dyn SliderTheme> {
    static THEME: OnceLock<Arc<dyn SliderTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSliderTheme::sdk())).clone()
}

impl DefaultSliderTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SliderTheme for DefaultSliderTheme {
    fn resolve(&self, state: InteractionState) -> SliderLook {
        let palette = self.tokens.stylesheet.recipes.palette("slider", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let elevation = &self.tokens.elevation;
        let selected = palette.state.selected;
        let fill_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => selected.background,
            InteractionLayer::Default => selected.background,
        };

        let fallback = SliderGeometry::sdk_default(metrics, &self.tokens.stylesheet);
        let geometry = self.tokens.stylesheet.slider.resolve_geometry(&fallback);
        SliderLook {
            thumb_border_width: self.tokens.metrics.border_width,
            decoration: self.tokens.stylesheet.slider.decoration(),
            track_background: if state.disabled {
                palette.state.disabled.background
            } else {
                palette.surface.subtle.background
            },
            fill_background,
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else {
                selected.background
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            width: geometry.width,
            height: geometry.height,
            track_height: geometry.track_height,
            thumb_size: geometry.thumb_size,
            radius: crate::theme::FULL_RADIUS_PX,
        }
    }
}

/// Default inset of the slider height below the control height. Looks that derive
/// it from their own spacing set `height` in slider geometry.
const SLIDER_HEIGHT_INSET_PX: f32 = 4.0;

fn slider_height(metrics: &MetricTokens, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> f32 {
    let recipe = stylesheet.slider.authored_geometry();
    let floor = recipe
        .height_floor_scale
        .or(crate::theme::stylesheet::CommonStylesheet::sdk_default().slider.geometry.height_floor_scale)
        .unwrap_or_default();
    recipe
        .height
        .unwrap_or_else(|| (metrics.control_height() - SLIDER_HEIGHT_INSET_PX).max(metrics.control_height() * floor))
}

/// Minimum slider track height, also subtracted from the control gap.
const SLIDER_TRACK_MIN_PX: f32 = 2.0;

fn slider_track_height(metrics: &MetricTokens) -> f32 {
    (metrics.gap() - SLIDER_TRACK_MIN_PX).max(SLIDER_TRACK_MIN_PX)
}

fn slider_thumb_size_with_stylesheet(
    metrics: &MetricTokens,
    stylesheet: &crate::theme::stylesheet::CommonStylesheet,
) -> f32 {
    let recipe = stylesheet.slider.authored_geometry();
    let factor = recipe
        .thumb_scale
        .or(crate::theme::stylesheet::CommonStylesheet::sdk_default().slider.geometry.thumb_scale)
        .unwrap_or_default();
    recipe.thumb_size.unwrap_or_else(|| slider_height(metrics, stylesheet) * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_theme_retains_p3_source_and_supplies_mapped_control_colors() {
        use crate::color::{ColorValue, GamutMapping, gpui_bridge};
        let mut source = crate::theme::LumaThemeMode::light();
        let p3 = ColorValue::display_p3(1.0, 0.0, 0.0, 0.37);
        source.palette.state.selected.background = p3;
        let tokens = ThemeTokens::from_source(source, GamutMapping::CssLocalMinde).unwrap();
        let theme = DefaultSliderTheme::new(tokens);
        let painted = theme.resolve(InteractionState::default());
        assert_eq!(painted.fill_background, gpui_bridge::to_hsla(p3, GamutMapping::CssLocalMinde).unwrap());
        assert_eq!(theme.tokens.source().palette.state.selected.background, p3);
        let disabled = theme.resolve(InteractionState { disabled: true, ..Default::default() });
        assert_ne!(disabled.fill_background, painted.fill_background);
        assert_eq!(theme.tokens.source().palette.state.selected.background, p3);
    }

    #[test]
    fn default_slider_geometry_tracks_custom_metric_tokens() {
        let mut tokens = ThemeTokens::default();
        tokens.metrics.control.control_height = 60.0;
        tokens.metrics.control.gap = 14.0;
        let theme = DefaultSliderTheme::new(tokens.clone());
        let look = theme.resolve(InteractionState::default());

        assert_eq!(look.height, 56.0);
        assert_eq!(look.track_height, 12.0);
        assert_eq!(look.thumb_size, 28.0);
    }
}

/// Shared slider dimensions; palettes, shadows and radius recipes remain look-owned.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SliderStylesheet {
    pub geometry: SliderGeometryStylesheet,
    /// Look-defined size keys; SDK sliders resolve the general geometry.
    pub sizes: ControlSizes<SliderGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SliderGeometryStylesheet {
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub dial_size: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_radius: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_unit")]
    pub disabled_opacity: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub width: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub height: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub track_height: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub thumb_size: Option<f32>,
    /// Minimum target height relative to the shared control height.
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub height_floor_scale: Option<f32>,
    /// Default thumb diameter relative to the selected target height.
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub thumb_scale: Option<f32>,
}

/// Concrete layout values shared by rendering and inspection.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderGeometry {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

/// Inspector descriptions are separate from the geometry used for painting.
#[derive(Clone, Debug)]
pub struct SliderGeometrySources {
    pub width: MetricSource,
    pub height: MetricSource,
    pub track_height: MetricSource,
    pub thumb_size: MetricSource,
}

impl SliderGeometry {
    pub fn sdk_default(metrics: &MetricTokens, stylesheet: &CommonStylesheet) -> Self {
        Self {
            width: 0.0,
            height: slider_height(metrics, stylesheet),
            track_height: slider_track_height(metrics),
            thumb_size: slider_thumb_size_with_stylesheet(metrics, stylesheet),
        }
    }
}

/// Nonlinear slider presentation metrics, in logical pixels and unit opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderDecoration {
    pub dial_size: f32,
    pub track_radius: f32,
    pub disabled_opacity: f32,
}

impl SliderStylesheet {
    /// Resolve radial paint dimensions and disabled opacity.
    pub fn decoration(&self) -> SliderDecoration {
        let recipe = self.authored_geometry();
        let baseline = CommonStylesheet::sdk_default().slider.authored_geometry();
        SliderDecoration {
            dial_size: recipe.dial_size.or(baseline.dial_size).unwrap_or_default(),
            track_radius: recipe.track_radius.or(baseline.track_radius).unwrap_or_default(),
            disabled_opacity: recipe.disabled_opacity.or(baseline.disabled_opacity).unwrap_or_default(),
        }
    }

    /// General authored inputs used by the SDK slider, including recipe coefficients.
    pub fn authored_geometry(&self) -> SliderGeometryStylesheet {
        self.geometry.clone()
    }

    /// Authored inputs for a look-defined size key, over the general fields.
    pub fn authored_geometry_for(&self, key: &str) -> SliderGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SliderGeometryStylesheet {
            dial_size: field!(dial_size),
            track_radius: field!(track_radius),
            disabled_opacity: field!(disabled_opacity),
            width: field!(width),
            height: field!(height),
            track_height: field!(track_height),
            thumb_size: field!(thumb_size),
            height_floor_scale: field!(height_floor_scale),
            thumb_scale: field!(thumb_scale),
        }
    }

    /// Resolve concrete general dimensions without constructing inspector metadata.
    pub fn resolve_geometry(&self, fallback: &SliderGeometry) -> SliderGeometry {
        Self::concrete_geometry(self.authored_geometry(), fallback)
    }

    /// Resolve concrete dimensions for a look-defined size key.
    pub fn resolve_geometry_for(&self, key: &str, fallback: &SliderGeometry) -> SliderGeometry {
        Self::concrete_geometry(self.authored_geometry_for(key), fallback)
    }

    fn concrete_geometry(authored: SliderGeometryStylesheet, fallback: &SliderGeometry) -> SliderGeometry {
        SliderGeometry {
            width: authored.width.unwrap_or(fallback.width),
            height: authored.height.unwrap_or(fallback.height),
            track_height: authored.track_height.unwrap_or(fallback.track_height),
            thumb_size: authored.thumb_size.unwrap_or(fallback.thumb_size),
        }
    }

    /// Resolve general, then SDK dimensions. Slider dimensions retain logical
    /// pixels without introducing display snapping.
    pub fn inspect_geometry(&self, fallback: &SliderGeometry) -> (SliderGeometry, SliderGeometrySources) {
        self.inspect_geometry_key(None, fallback)
    }

    /// Resolve a look-defined size key, then general, then SDK dimensions.
    pub fn inspect_geometry_for(
        &self,
        key: &str,
        fallback: &SliderGeometry,
    ) -> (SliderGeometry, SliderGeometrySources) {
        self.inspect_geometry_key(Some(key), fallback)
    }

    fn inspect_geometry_key(
        &self,
        key: Option<&str>,
        fallback: &SliderGeometry,
    ) -> (SliderGeometry, SliderGeometrySources) {
        let specific = key.and_then(|key| self.sizes.get(key)).cloned().unwrap_or_default();
        let key = key.unwrap_or_default();
        let resolve = |field: &str, specific: Option<f32>, general: Option<f32>, fallback: f32| {
            if let Some(value) = specific {
                ResolvedMetric::authored(value, format!("common.slider.sizes.{key}.{field}"))
            } else if let Some(value) = general {
                ResolvedMetric::authored(value, format!("common.slider.geometry.{field}"))
            } else {
                ResolvedMetric::constant(fallback, format!("slider {field} fallback"))
            }
        };
        let width = resolve("width", specific.width, self.geometry.width, fallback.width);
        let height = resolve("height", specific.height, self.geometry.height, fallback.height);
        let track_height =
            resolve("track_height", specific.track_height, self.geometry.track_height, fallback.track_height);
        let thumb = resolve("thumb_size", specific.thumb_size, self.geometry.thumb_size, fallback.thumb_size);
        (
            SliderGeometry {
                width: width.value_px,
                height: height.value_px,
                track_height: track_height.value_px,
                thumb_size: thumb.value_px,
            },
            SliderGeometrySources {
                width: width.source,
                height: height.source,
                track_height: track_height.source,
                thumb_size: thumb.source,
            },
        )
    }

    /// Configure general slider width in logical pixels.
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.geometry.width = Some(width);
        self
    }

    /// Configure general slider height in logical pixels.
    pub fn height(&mut self, height: f32) -> &mut Self {
        self.geometry.height = Some(height);
        self
    }

    /// Configure general slider track height in logical pixels.
    pub fn track_height(&mut self, track_height: f32) -> &mut Self {
        self.geometry.track_height = Some(track_height);
        self
    }

    /// Configure general slider thumb size in logical pixels.
    pub fn thumb_size(&mut self, thumb_size: f32) -> &mut Self {
        self.geometry.thumb_size = Some(thumb_size);
        self
    }

    /// Configure general slider height floor scale relative to control height.
    pub fn height_floor_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.height_floor_scale = Some(scale);
        self
    }

    /// Configure general slider thumb scale relative to resolved height.
    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.thumb_scale = Some(scale);
        self
    }

    /// Configure general slider radial dial size in logical pixels.
    pub fn dial_size(&mut self, dial_size: f32) -> &mut Self {
        self.geometry.dial_size = Some(dial_size);
        self
    }

    /// Configure general slider radial track radius in logical pixels.
    pub fn track_radius(&mut self, track_radius: f32) -> &mut Self {
        self.geometry.track_radius = Some(track_radius);
        self
    }

    /// Configure general slider disabled opacity (0.0 to 1.0).
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    /// Configure or override a look-defined size key without manual map manipulation.
    pub fn size(&mut self, key: &str, configure: impl FnOnce(&mut SliderGeometryStylesheet)) -> &mut Self {
        let entry = self.sizes.get_or_default(key);
        configure(entry);
        self
    }

    /// Construct the native SDK slider stylesheet in pure Rust without preloading TOML.
    pub fn native_default() -> Self {
        let mut slider = Self::default();
        slider
            .width(260.0)
            .height_floor_scale(0.5)
            .thumb_scale(0.5)
            .dial_size(200.0)
            .track_radius(72.0)
            .disabled_opacity(0.56);
        slider
    }

    /// Apply authored slider fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }
}

crate::theme::stylesheet::geometry::inherit_geometry!(
    SliderStylesheet,
    sizes,
    [
        width,
        height,
        track_height,
        thumb_size,
        height_floor_scale,
        thumb_scale,
        dial_size,
        track_radius,
        disabled_opacity
    ]
);

impl SliderGeometryStylesheet {
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn height(&mut self, height: f32) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn track_height(&mut self, track_height: f32) -> &mut Self {
        self.track_height = Some(track_height);
        self
    }

    pub fn thumb_size(&mut self, thumb_size: f32) -> &mut Self {
        self.thumb_size = Some(thumb_size);
        self
    }

    pub fn height_floor_scale(&mut self, scale: f32) -> &mut Self {
        self.height_floor_scale = Some(scale);
        self
    }

    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.thumb_scale = Some(scale);
        self
    }

    pub fn dial_size(&mut self, dial_size: f32) -> &mut Self {
        self.dial_size = Some(dial_size);
        self
    }

    pub fn track_radius(&mut self, track_radius: f32) -> &mut Self {
        self.track_radius = Some(track_radius);
        self
    }

    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    /// Convenience setter for height, track height, and thumb size in logical pixels.
    pub fn dimensions(&mut self, height: f32, track_height: f32, thumb_size: f32) -> &mut Self {
        self.height(height);
        self.track_height(track_height);
        self.thumb_size(thumb_size);
        self
    }
}

impl SliderMotionStylesheet {
    /// Construct native SDK slider motion specs in pure Rust without preloading TOML.
    pub fn native_default() -> Self {
        Self { thumb: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct native SDK slider palette recipe mappings in pure Rust without preloading TOML.
pub fn native_sdk_slider_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    HashMap::from([
        (StateDisabledBackground, StateDisabledBackground),
        (StateDisabledForeground, StateDisabledForeground),
        (StatePressedBackground, StatePressedBackground),
        (StateSelectedBackground, StateSelectedBackground),
        (StateSelectedForeground, StateSelectedForeground),
        (SurfacePanelBackground, SurfacePanelBackground),
        (SurfaceSubtleBackground, SurfaceSubtleBackground),
    ])
}

/// Native SDK slider defaults.
pub fn define_sdk_slider(sheet: &mut CommonStylesheet) {
    sheet.slider = SliderStylesheet::native_default();
    sheet.motion.slider = SliderMotionStylesheet::native_default();
    sheet.recipes.0.insert("slider".into(), native_sdk_slider_recipes());
}
