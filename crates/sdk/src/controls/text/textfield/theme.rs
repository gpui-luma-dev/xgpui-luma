use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::{ControlSize, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens};
use crate::controls::textfield::TextFieldState;

#[derive(Clone, Debug)]
pub struct TextFieldPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct TextFieldLook {
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
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon_size: f32,
}

pub trait TextFieldTheme: Send + Sync {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        let _ = size;
        compose_textfield_look(&self.resolve(state, enabled), scale, self.metrics().border_width.default)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextFieldTheme {
    tokens: ThemeTokens,
}

pub fn default_textfield_theme() -> Arc<dyn TextFieldTheme> {
    static THEME: OnceLock<Arc<dyn TextFieldTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextFieldTheme::default())).clone()
}

impl DefaultTextFieldTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextFieldTheme for DefaultTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        let palette = self.tokens.stylesheet.recipes.palette("textfield", &self.tokens.palette);
        let typography = &self.tokens.typography;

        let (background, foreground, border, placeholder, icon, selection_background, selection_foreground, caret) =
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
                    palette.state.disabled.foreground,
                    palette.state.selected.background,
                    palette.state.selected.foreground,
                    palette.state.disabled.foreground,
                )
            };

        TextFieldPalette {
            focus_border: if state.invalid {
                gpui::transparent_black()
            } else {
                palette.focus.ring
            },
            background,
            foreground,
            border,
            placeholder,
            icon,
            selection_background,
            selection_foreground,
            caret,
            shadow: None,
            typography: typography.text.body,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        let mut palette = self.resolve(state, enabled);
        apply_control_size_typography(&mut palette.typography, &self.tokens.typography, size);
        let mut look = compose_textfield_look(&palette, scale, self.metrics().border_width.default);
        look.helper_geometry = self.tokens.stylesheet.shared_geometry();
        crate::theme::stylesheet::apply_common_geometry!(
            self.tokens.stylesheet.textfield, size.stylesheet_key(), TextFieldGeometry, look,
            [min_height => min_height, padding_x => padding_x, padding_y => padding_y, gap => gap, icon_size => icon_size, font_size => typography.size, line_height => typography.line_height, caret_width => caret_width, caret_height_extra => caret_height_extra, disabled_opacity => disabled_opacity]
        );
        look
    }
}

/// Scales text-field typography from `LumaTypography` size roles for lookless fallbacks.
pub fn apply_control_size_typography(
    typography: &mut LumaTextStyle,
    tokens: &crate::theme::LumaTypography,
    size: ControlSize,
) {
    let font_size = match size {
        ControlSize::Sm => tokens.text.scale.sm.size,
        ControlSize::Md => tokens.text.body.size,
        ControlSize::Lg => tokens.text.scale.lg.size,
    };
    let base_size = typography.size;
    typography.size = font_size;
    if base_size > 0.0 {
        typography.line_height = font_size * (typography.line_height / base_size);
    }
}

pub fn compose_textfield_look(
    palette: &TextFieldPalette,
    scale: &StandardBoxScale,
    border_width: f32,
) -> TextFieldLook {
    TextFieldLook {
        helper_geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry(),
        caret_width: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textfield
            .geometry
            .caret_width
            .unwrap_or_default(),
        caret_height_extra: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textfield
            .geometry
            .caret_height_extra
            .unwrap_or_default(),
        disabled_opacity: crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .textfield
            .geometry
            .disabled_opacity
            .unwrap_or_default(),
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        focus_border: palette.focus_border,
        placeholder: palette.placeholder,
        icon: palette.icon,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        shadow: palette.shadow.clone(),
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        gap: scale.gap,
        radius: scale.radius,
        border_width,
        icon_size: scale.icon_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_scales_typography_with_control_size() {
        let theme = DefaultTextFieldTheme::default();
        let scale_md = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0);
        let scale_sm = StandardBoxScale::compute(ControlSize::Sm, &theme.metrics(), 1.0);
        let scale_lg = StandardBoxScale::compute(ControlSize::Lg, &theme.metrics(), 1.0);

        let md = theme.resolve_look(TextFieldState::default(), true, ControlSize::Md, &scale_md);
        let sm = theme.resolve_look(TextFieldState::default(), true, ControlSize::Sm, &scale_sm);
        let lg = theme.resolve_look(TextFieldState::default(), true, ControlSize::Lg, &scale_lg);

        assert!((md.typography.size - theme.tokens.typography.text.body.size).abs() < f32::EPSILON);
        assert!(sm.typography.size < md.typography.size);
        assert!(lg.typography.size > md.typography.size);
        assert_eq!(sm.icon_size, theme.metrics().control.sm.icon_size);
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    TextFieldStylesheet,
    TextFieldGeometryStylesheet,
    TextFieldGeometry,
    ResolvedTextFieldGeometry,
    "textfield",
    [
        min_height,
        padding_x,
        padding_y,
        gap,
        icon_size,
        font_size,
        line_height,
        caret_width,
        caret_height_extra
    ],
    unit[disabled_opacity]
);

impl TextFieldStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TextFieldGeometryStylesheet {
                caret_width: Some(1.5),
                caret_height_extra: Some(2.0),
                disabled_opacity: Some(0.6),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
