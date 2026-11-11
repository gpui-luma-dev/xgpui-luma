use crate::theme::stylesheet::{CheckboxMotionStylesheet, CommonStylesheet, PaletteRole, TransitionSpec};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::scales::{label_baseline_shift, snap_to_pixel};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxScale {
    /// Whole-control opacity when disabled.
    pub disabled_opacity: f32,
    pub control_radius: f32,
    pub control_padding_x: f32,
    pub control_padding_y: f32,
    pub indicator_size: f32,
    pub indicator_radius: f32,
    pub height: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
    pub glyph_size: f32,
}

impl CheckboxScale {
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
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().checkbox.authored_geometry(key);
        let geometry = stylesheet.checkbox.authored_geometry(key).with_fallback(&baseline);
        let indicator_size = snap_to_pixel(
            geometry
                .indicator_size
                .unwrap_or_else(|| metrics.control_height(size) * geometry.indicator_scale.unwrap_or_default()),
            scale_factor,
        );
        let glyph_inset = snap_to_pixel(geometry.glyph_inset.unwrap_or_default(), scale_factor);

        Self {
            disabled_opacity: geometry.disabled_opacity.unwrap_or_default(),
            control_radius: metrics.radius(size),
            control_padding_x: snap_to_pixel(geometry.control_padding_x.unwrap_or_default(), scale_factor),
            control_padding_y: snap_to_pixel(geometry.control_padding_y.unwrap_or_default(), scale_factor),
            indicator_size,
            height: snap_to_pixel(metrics.control_height(size), scale_factor),
            gap: snap_to_pixel(geometry.gap.unwrap_or_else(|| metrics.gap(size)), scale_factor),
            label_baseline_shift: label_baseline_shift(size, stylesheet),
            indicator_radius: snap_to_pixel(geometry.indicator_radius.unwrap_or_default(), scale_factor),
            glyph_size: snap_to_pixel(
                geometry.glyph_size.unwrap_or_else(|| (indicator_size - glyph_inset * 2.0).max(0.0)),
                scale_factor,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckboxPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub control_background: Option<Hsla>,
    /// Body-border paint; `None` is transparent and retains the configured border width.
    pub control_border: Option<Hsla>,
    pub indicator_background: Hsla,
    pub indicator_border: Hsla,
    pub checkmark_color: Hsla,
    pub label_color: Hsla,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
    pub indicator_shadow: Option<Vec<BoxShadow>>,
}

pub type CheckboxLook = CheckboxPalette;

pub trait CheckboxTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette;
    fn metrics(&self) -> MetricTokens;

    /// Geometry for the indicator / label row. Defaults to [`CheckboxScale::compute`].
    fn scale(&self, size: ControlSize, scale_factor: f32) -> CheckboxScale {
        CheckboxScale::compute(size, &self.metrics(), scale_factor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultCheckboxTheme {
    tokens: ThemeTokens,
}

pub fn default_checkbox_theme() -> Arc<dyn CheckboxTheme> {
    static THEME: OnceLock<Arc<dyn CheckboxTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultCheckboxTheme::default())).clone()
}

impl DefaultCheckboxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl CheckboxTheme for DefaultCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette {
        let palette = self.tokens.stylesheet.recipes.palette("checkbox", &self.tokens.palette);
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let selected = palette.state.selected;

        let indicator_background = match (checked, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.state.pressed.background,
            (true, InteractionLayer::Hovered) => selected.background,
            (true, InteractionLayer::Default) => selected.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        CheckboxPalette {
            focus_border: palette.focus.ring,
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border: if checked && !state.disabled {
                indicator_background
            } else {
                palette.form.input.border
            },
            checkmark_color: if state.disabled {
                palette.state.disabled.foreground
            } else if checked {
                selected.foreground
            } else {
                palette.app.foreground
            },
            label_color,
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

    fn scale(&self, size: ControlSize, scale_factor: f32) -> CheckboxScale {
        CheckboxScale::compute_with_stylesheet(size, &self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    CheckboxStylesheet,
    CheckboxGeometryStylesheet,
    CheckboxGeometry,
    ResolvedCheckboxGeometry,
    "checkbox",
    [
        indicator_size,
        indicator_scale,
        indicator_radius,
        glyph_size,
        glyph_inset,
        control_padding_x,
        control_padding_y,
        gap
    ],
    unit[disabled_opacity]
);

impl CheckboxStylesheet {
    /// Construct the SDK checkbox geometry without parsing bundled data.
    pub fn native_default() -> Self {
        let mut checkbox = Self::default();
        checkbox
            .disabled_opacity(0.56)
            .control_padding_x(0.0)
            .control_padding_y(0.0)
            .size("sm", |m| {
                m.indicator(0.45, 2.0, 2.0);
            })
            .size("md", |m| {
                m.indicator(0.5, 4.0, 3.0);
            })
            .size("lg", |m| {
                m.indicator(0.55, 6.0, 4.0);
            });
        checkbox
    }

    /// Apply authored checkbox fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }

    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    pub fn indicator_size(&mut self, size: f32) -> &mut Self {
        self.geometry.indicator_size = Some(size);
        self
    }

    pub fn indicator_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.indicator_scale = Some(scale);
        self
    }

    pub fn indicator_radius(&mut self, radius: f32) -> &mut Self {
        self.geometry.indicator_radius = Some(radius);
        self
    }

    pub fn glyph_size(&mut self, size: f32) -> &mut Self {
        self.geometry.glyph_size = Some(size);
        self
    }

    pub fn glyph_inset(&mut self, inset: f32) -> &mut Self {
        self.geometry.glyph_inset = Some(inset);
        self
    }

    pub fn control_padding_x(&mut self, padding: f32) -> &mut Self {
        self.geometry.control_padding_x = Some(padding);
        self
    }

    pub fn control_padding_y(&mut self, padding: f32) -> &mut Self {
        self.geometry.control_padding_y = Some(padding);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.geometry.gap = Some(gap);
        self
    }

    pub fn size(
        &mut self,
        key: impl Into<String>,
        configure: impl FnOnce(&mut CheckboxGeometryStylesheet),
    ) -> &mut Self {
        let key = key.into();
        let entry = self.sizes.get_or_default(key.as_str());
        configure(entry);
        self
    }
}

impl CheckboxGeometryStylesheet {
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    pub fn indicator_size(&mut self, size: f32) -> &mut Self {
        self.indicator_size = Some(size);
        self
    }

    pub fn indicator_scale(&mut self, scale: f32) -> &mut Self {
        self.indicator_scale = Some(scale);
        self
    }

    pub fn indicator_radius(&mut self, radius: f32) -> &mut Self {
        self.indicator_radius = Some(radius);
        self
    }

    pub fn glyph_size(&mut self, size: f32) -> &mut Self {
        self.glyph_size = Some(size);
        self
    }

    pub fn glyph_inset(&mut self, inset: f32) -> &mut Self {
        self.glyph_inset = Some(inset);
        self
    }

    pub fn control_padding_x(&mut self, padding: f32) -> &mut Self {
        self.control_padding_x = Some(padding);
        self
    }

    pub fn control_padding_y(&mut self, padding: f32) -> &mut Self {
        self.control_padding_y = Some(padding);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.gap = Some(gap);
        self
    }

    pub fn indicator(&mut self, scale: f32, radius: f32, glyph_inset: f32) -> &mut Self {
        self.indicator_scale(scale);
        self.indicator_radius(radius);
        self.glyph_inset(glyph_inset);
        self
    }
}

impl CheckboxMotionStylesheet {
    /// Construct the SDK checkbox motion without parsing bundled data.
    pub fn native_default() -> Self {
        Self { state: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct SDK checkbox palette references without parsing bundled data.
pub fn native_sdk_checkbox_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    [
        AppForeground,
        FocusRing,
        FormInputBackground,
        FormInputBorder,
        StateDisabledBackground,
        StateDisabledForeground,
        StateHoverBackground,
        StatePressedBackground,
        StateSelectedBackground,
        StateSelectedForeground,
    ]
    .into_iter()
    .map(|role| (role, role))
    .collect()
}

/// Install the native SDK checkbox baseline.
pub fn define_sdk_checkbox(sheet: &mut CommonStylesheet) {
    sheet.checkbox = CheckboxStylesheet::native_default();
    sheet.motion.checkbox = CheckboxMotionStylesheet::native_default();
    sheet.recipes.0.insert("checkbox".into(), native_sdk_checkbox_recipes());
}
