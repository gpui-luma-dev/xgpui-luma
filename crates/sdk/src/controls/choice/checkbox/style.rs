//! Declarative checkbox style (issue #282 spike).
//!
//! One struct holds what `CheckboxScale`, `CheckboxPalette` and
//! `DefaultCheckboxTheme::resolve` compute today. The template does not read it
//! yet; the parity test below checks it against the current theme.

use xgpui_luma_style::State::*;
use xgpui_luma_style::{Color, FontFamily, Opacity, Px, Scalar, Shadow, States, Style, V, Weight, none, px};

use crate::theme::style_tokens::tok::{geometry as g, metrics as m, palette as p, typography as t};

/// SDK checkbox.
#[derive(Style, Clone, Debug)]
#[style(path = "checkbox", snap)]
pub struct CheckboxStyle {
    /// Whole-control opacity when disabled.
    #[style(range = 0.0..=1.0)]
    pub disabled_opacity: V<Opacity>,
    /// Corner radius of the whole control row.
    #[style(range = 0.0..=32.0, snap = false)]
    pub control_radius: V<Px>,
    #[style(range = 0.0..=32.0)]
    pub control_padding_x: V<Px>,
    #[style(range = 0.0..=32.0)]
    pub control_padding_y: V<Px>,
    /// Minimum row height.
    #[style(range = 16.0..=64.0)]
    pub height: V<Px>,
    /// Space between the indicator and the label.
    #[style(range = 0.0..=32.0)]
    pub gap: V<Px>,
    /// Border width of the control row and the indicator.
    #[style(range = 0.0..=4.0, snap = false)]
    pub border_width: V<Px>,
    /// Indicator size as a fraction of the control height.
    #[style(range = 0.25..=1.0)]
    pub indicator_scale: V<Scalar>,
    /// Indicator box size.
    #[style(range = 8.0..=48.0)]
    pub indicator_size: V<Px>,
    #[style(range = 0.0..=24.0)]
    pub indicator_radius: V<Px>,
    /// Checkmark inset inside the indicator.
    #[style(range = 0.0..=12.0)]
    pub glyph_inset: V<Px>,
    /// Checkmark size.
    #[style(range = 0.0..=48.0)]
    pub glyph_size: V<Px>,
    /// Shift aligning the label baseline with the indicator.
    #[style(range = -8.0..=8.0, snap = false)]
    pub label_baseline_shift: V<Px>,
    pub root: CheckboxRootPart,
    pub indicator: CheckboxIndicatorPart,
    pub glyph: CheckboxGlyphPart,
    pub label: CheckboxLabelPart,
}

/// The whole control row.
#[derive(Style, Clone, Debug)]
pub struct CheckboxRootPart {
    pub background: V<Option<Color>>,
    pub border: V<Option<Color>>,
    /// Focus-ring paint.
    pub focus_border: V<Color>,
}

/// The checkbox box.
#[derive(Style, Clone, Debug)]
pub struct CheckboxIndicatorPart {
    pub background: States<V<Color>>,
    pub border: States<V<Color>>,
    pub shadow: States<V<Option<Shadow>>>,
}

/// The checkmark.
#[derive(Style, Clone, Debug)]
pub struct CheckboxGlyphPart {
    pub foreground: States<V<Color>>,
}

/// The label text.
#[derive(Style, Clone, Debug)]
pub struct CheckboxLabelPart {
    pub foreground: States<V<Color>>,
    pub font_family: V<FontFamily>,
    pub font_size: V<Px>,
    pub line_height: V<Px>,
    pub font_weight: V<Weight>,
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self {
            disabled_opacity: Opacity(0.56).into(),
            control_radius: m::RADIUS.into(),
            control_padding_x: px(0.0).into(),
            control_padding_y: px(0.0).into(),
            height: m::CONTROL_HEIGHT.into(),
            gap: m::GAP.into(),
            border_width: m::BORDER_WIDTH.into(),
            indicator_scale: Scalar(0.5).into(),
            indicator_size: m::CONTROL_HEIGHT * Self::INDICATOR_SCALE,
            indicator_radius: px(4.0).into(),
            glyph_inset: px(3.0).into(),
            glyph_size: (Self::INDICATOR_SIZE - Self::GLYPH_INSET * 2.0).max(px(0.0)),
            label_baseline_shift: g::LABEL_BASELINE.into(),
            root: CheckboxRootPart { background: none(), border: none(), focus_border: p::FOCUS_RING.into() },
            indicator: CheckboxIndicatorPart {
                background: States::new(p::FORM_INPUT_BACKGROUND)
                    .on(Hovered, p::STATE_HOVER_BACKGROUND)
                    .on(Pressed, p::STATE_PRESSED_BACKGROUND)
                    .on(Checked, p::STATE_SELECTED_BACKGROUND)
                    .on(Checked + Hovered, p::STATE_SELECTED_BACKGROUND)
                    .on(Disabled, p::STATE_DISABLED_BACKGROUND),
                border: States::new(p::FORM_INPUT_BORDER)
                    .on(Checked, CheckboxIndicatorPart::BACKGROUND)
                    .on(Disabled, p::FORM_INPUT_BORDER),
                shadow: States::new(none::<Shadow>()),
            },
            glyph: CheckboxGlyphPart {
                foreground: States::new(p::APP_FOREGROUND)
                    .on(Checked, p::STATE_SELECTED_FOREGROUND)
                    .on(Disabled, p::STATE_DISABLED_FOREGROUND),
            },
            label: CheckboxLabelPart {
                foreground: States::new(p::APP_FOREGROUND).on(Disabled, p::STATE_DISABLED_FOREGROUND),
                font_family: t::SANS.into(),
                font_size: t::BODY_SIZE.into(),
                line_height: CheckboxLabelPart::FONT_SIZE * t::LABEL_LINE_HEIGHT / t::LABEL_SIZE,
                font_weight: t::LABEL_WEIGHT.into(),
            },
        }
    }
}

impl CheckboxStyle {
    /// The SDK default with a common stylesheet's authored checkbox geometry
    /// applied, so apps and looks that author `[common.checkbox]` keep working.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let authored = stylesheet.checkbox.authored_geometry();
        let mut style = Self::default();
        // Field names match the stylesheet keys, so the generated path setter applies them.
        for (path, value) in [
            ("disabled_opacity", authored.disabled_opacity),
            ("indicator_scale", authored.indicator_scale),
            ("indicator_size", authored.indicator_size),
            ("indicator_radius", authored.indicator_radius),
            ("glyph_size", authored.glyph_size),
            ("glyph_inset", authored.glyph_inset),
            ("control_padding_x", authored.control_padding_x),
            ("control_padding_y", authored.control_padding_y),
            ("gap", authored.gap),
        ] {
            if let Some(value) = value {
                let applied = style.set(path, xgpui_luma_style::StateSet::NONE, xgpui_luma_style::Dyn::Number(value));
                debug_assert!(applied, "checkbox style has no numeric field `{path}`");
            }
        }
        style
    }
}

/// The interaction part of a state set, for themes that take an `InteractionState`.
pub(crate) fn interaction_state(at: xgpui_luma_style::StateSet) -> crate::theme::InteractionState {
    crate::theme::InteractionState {
        hovered: at.contains(Hovered),
        pressed: at.contains(Pressed),
        focused: at.contains(Focused),
        disabled: at.contains(Disabled),
        invalid: at.contains(Invalid),
    }
}

impl From<crate::theme::InteractionState> for xgpui_luma_style::StateSet {
    fn from(state: crate::theme::InteractionState) -> Self {
        let mut set = xgpui_luma_style::StateSet::NONE;
        for (active, flag) in [
            (state.hovered, Hovered),
            (state.pressed, Pressed),
            (state.focused, Focused),
            (state.disabled, Disabled),
            (state.invalid, Invalid),
        ] {
            if active {
                set = set.with(flag);
            }
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::checkbox::{CheckboxTheme, DefaultCheckboxTheme};
    use crate::theme::style_tokens::ThemeTokenSource;
    use crate::theme::{InteractionState, ThemeTokens};
    use xgpui_luma_style::{ResolveCx, StateSet};

    fn every_interaction_state() -> impl Iterator<Item = InteractionState> {
        (0..32).map(|bits| InteractionState {
            hovered: bits & 1 != 0,
            pressed: bits & 2 != 0,
            focused: bits & 4 != 0,
            disabled: bits & 8 != 0,
            invalid: bits & 16 != 0,
        })
    }

    #[test]
    fn matches_the_current_sdk_checkbox_theme() {
        let style = CheckboxStyle::default();
        for (name, tokens) in [("light", ThemeTokens::light()), ("dark", ThemeTokens::dark())] {
            let theme = DefaultCheckboxTheme::new(tokens.clone());
            let source = ThemeTokenSource::for_family(&tokens, "checkbox");
            for scale_factor in [1.0, 1.25, 1.5, 2.0] {
                let scale = theme.scale(scale_factor);
                for state in every_interaction_state() {
                    for checked in [false, true] {
                        let at = StateSet::from(state);
                        let at = if checked { at.with(Checked) } else { at };
                        let cx = ResolveCx::new(&source, scale_factor);
                        let resolved = style.resolve(&cx, at);
                        let case = format!("{name} @{scale_factor}x, {at}");
                        assert!(cx.errors().is_empty(), "{case}: {:?}", cx.errors());

                        let palette = theme.resolve(checked, state);
                        assert_eq!(resolved.disabled_opacity, scale.disabled_opacity, "{case}: disabled_opacity");
                        assert_eq!(resolved.control_radius, scale.control_radius, "{case}: control_radius");
                        assert_eq!(resolved.control_padding_x, scale.control_padding_x, "{case}: padding_x");
                        assert_eq!(resolved.control_padding_y, scale.control_padding_y, "{case}: padding_y");
                        assert_eq!(resolved.height, scale.height, "{case}: height");
                        assert_eq!(resolved.gap, scale.gap, "{case}: gap");
                        assert_eq!(resolved.border_width, tokens.metrics.border_width, "{case}: border_width");
                        assert_eq!(resolved.indicator_size, scale.indicator_size, "{case}: indicator_size");
                        assert_eq!(resolved.indicator_radius, scale.indicator_radius, "{case}: indicator_radius");
                        assert_eq!(resolved.glyph_size, scale.glyph_size, "{case}: glyph_size");
                        assert_eq!(
                            resolved.label_baseline_shift, scale.label_baseline_shift,
                            "{case}: label_baseline_shift"
                        );

                        assert_eq!(resolved.root.focus_border, palette.focus_border, "{case}: focus_border");
                        assert_eq!(resolved.root.background, palette.control_background, "{case}: root background");
                        assert_eq!(resolved.root.border, palette.control_border, "{case}: root border");
                        assert_eq!(
                            resolved.indicator.background, palette.indicator_background,
                            "{case}: indicator background"
                        );
                        assert_eq!(resolved.indicator.border, palette.indicator_border, "{case}: indicator border");
                        assert_eq!(resolved.indicator.shadow, palette.indicator_shadow, "{case}: indicator shadow");
                        assert_eq!(resolved.glyph.foreground, palette.checkmark_color, "{case}: checkmark");
                        assert_eq!(resolved.label.foreground, palette.label_color, "{case}: label color");
                        assert_eq!(resolved.label.font_family, palette.label_font_family, "{case}: font family");
                        assert_eq!(resolved.label.font_size, palette.label_typography.size, "{case}: font size");
                        assert_eq!(
                            resolved.label.line_height, palette.label_typography.line_height,
                            "{case}: line height"
                        );
                        assert_eq!(resolved.label.font_weight, palette.label_typography.weight, "{case}: weight");
                    }
                }
            }
        }
    }

    #[test]
    fn theme_resolved_style_matches_the_adapter() {
        for tokens in [ThemeTokens::light(), ThemeTokens::dark()] {
            let theme = DefaultCheckboxTheme::new(tokens.clone());
            for scale_factor in [1.0, 1.5, 2.0] {
                for state in every_interaction_state() {
                    for checked in [false, true] {
                        let at = StateSet::from(state);
                        let at = if checked { at.with(Checked) } else { at };
                        let mut expected = crate::controls::checkbox::theme::resolved_from_palette(
                            &theme.resolve(checked, state),
                            &theme.scale(scale_factor),
                            &theme.metrics(),
                        );
                        let actual = theme.resolved_style(at, scale_factor);
                        // The adapter derives these two; the style authors them.
                        expected.indicator_scale = actual.indicator_scale;
                        expected.glyph_inset = actual.glyph_inset;
                        assert_eq!(actual, expected, "@{scale_factor}x, {at}");
                    }
                }
            }
        }
    }

    #[test]
    fn authored_stylesheet_geometry_still_applies() {
        let mut tokens = ThemeTokens::light();
        let mut sheet = (*tokens.stylesheet).clone();
        sheet
            .checkbox
            .indicator_size(18.0)
            .indicator_radius(2.0)
            .glyph_inset(4.0)
            .gap(10.0)
            .control_padding_x(3.0)
            .disabled_opacity(0.4);
        tokens.stylesheet = std::sync::Arc::new(sheet);
        let theme = DefaultCheckboxTheme::new(tokens);
        for scale_factor in [1.0, 1.5] {
            let scale = theme.scale(scale_factor);
            let resolved = theme.resolved_style(StateSet::NONE, scale_factor);
            assert_eq!(resolved.indicator_size, scale.indicator_size);
            assert_eq!(resolved.indicator_radius, scale.indicator_radius);
            assert_eq!(resolved.glyph_size, scale.glyph_size);
            assert_eq!(resolved.gap, scale.gap);
            assert_eq!(resolved.control_padding_x, scale.control_padding_x);
            assert_eq!(resolved.disabled_opacity, scale.disabled_opacity);
        }
    }

    #[test]
    fn is_registered_for_tools() {
        assert!(xgpui_luma_style::registered().any(|shape| shape.name == "checkbox"));
    }
}
