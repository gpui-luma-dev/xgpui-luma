use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens};
use crate::controls::textarea::TextAreaState;
use crate::controls::textfield::apply_control_size_typography;

#[derive(Clone, Debug)]
pub struct TextAreaPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
}

#[derive(Clone, Debug)]
pub struct TextAreaLook {
    /// Helper chrome resolved through the selected common stylesheet.
    pub helper_geometry: crate::theme::stylesheet::SharedGeometry,
    /// Painted caret width and extra height in pixels.
    pub caret_width: f32,
    pub caret_height_extra: f32,
    /// Whole-input opacity when disabled.
    pub disabled_opacity: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    /// Independent focus paint; transparency does not remove focus space.
    pub focus_border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub border_width: f32,
}

pub trait TextAreaTheme: Send + Sync {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        state: TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextAreaLook {
        let _ = size;
        compose_textarea_look(&self.resolve(state, enabled), scale, self.metrics().border_width.default)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextAreaTheme {
    tokens: ThemeTokens,
}

pub fn default_textarea_theme() -> Arc<dyn TextAreaTheme> {
    static THEME: OnceLock<Arc<dyn TextAreaTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextAreaTheme::default())).clone()
}

impl DefaultTextAreaTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextAreaTheme for DefaultTextAreaTheme {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let palette = self.tokens.stylesheet.recipes.palette("textarea", &self.tokens.palette);
        let typography = &self.tokens.typography;

        let (background, foreground, border, placeholder, selection_background, selection_foreground, caret) =
            if enabled {
                let background = palette.form.input.background;
                let border = if state.invalid {
                    palette.form.input.invalid_border
                } else {
                    palette.form.input.border
                };

                (
                    background,
                    palette.form.input.foreground,
                    border,
                    palette.form.input.placeholder,
                    palette.state.selected.background,
                    palette.state.selected.foreground,
                    palette.form.input.foreground,
                )
            } else {
                (
                    palette.state.disabled.background,
                    palette.state.disabled.foreground,
                    palette.form.input.border,
                    palette.state.disabled.foreground,
                    palette.state.selected.background,
                    palette.state.selected.foreground,
                    palette.state.disabled.foreground,
                )
            };

        TextAreaPalette {
            focus_border: if state.invalid {
                gpui::transparent_black()
            } else {
                palette.focus.ring
            },
            background,
            foreground,
            border,
            placeholder,
            selection_background,
            selection_foreground,
            caret,
            shadow: None,
            typography: typography.text.body,
            font_family: typography.font.sans.family.clone(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn resolve_look(
        &self,
        state: TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextAreaLook {
        let mut palette = self.resolve(state, enabled);
        apply_control_size_typography(&mut palette.typography, &self.tokens.typography, size);
        let mut look = compose_textarea_look(&palette, scale, self.metrics().border_width.default);
        look.helper_geometry = self.tokens.stylesheet.shared_geometry();
        crate::theme::stylesheet::apply_common_geometry!(
            self.tokens.stylesheet.textarea, size.stylesheet_key(), TextAreaGeometry, look,
            [min_height => min_height, padding_x => padding_x, padding_y => padding_y, font_size => typography.size, line_height => typography.line_height, caret_width => caret_width, caret_height_extra => caret_height_extra, disabled_opacity => disabled_opacity]
        );
        look
    }
}

pub fn compose_textarea_look(palette: &TextAreaPalette, scale: &StandardBoxScale, border_width: f32) -> TextAreaLook {
    TextAreaLook {
        helper_geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry(),
        caret_width: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textarea
            .geometry
            .caret_width
            .unwrap_or_default(),
        caret_height_extra: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textarea
            .geometry
            .caret_height_extra
            .unwrap_or_default(),
        disabled_opacity: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textarea
            .geometry
            .disabled_opacity
            .unwrap_or_default(),
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        focus_border: palette.focus_border,
        placeholder: palette.placeholder,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        shadow: palette.shadow.clone(),
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        radius: scale.radius,
        border_width,
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    TextAreaStylesheet,
    TextAreaGeometryStylesheet,
    TextAreaGeometry,
    ResolvedTextAreaGeometry,
    "textarea",
    [min_height, padding_x, padding_y, font_size, line_height, caret_width, caret_height_extra],
    unit[disabled_opacity]
);

impl TextAreaStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TextAreaGeometryStylesheet {
                caret_width: Some(1.5),
                caret_height_extra: Some(2.0),
                disabled_opacity: Some(0.6),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
