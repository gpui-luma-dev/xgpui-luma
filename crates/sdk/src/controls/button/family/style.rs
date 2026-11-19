//! Declarative button style (issue #282 spike).
//!
//! Covers the Text and Icon button roles. The Icon role is a variant of the
//! default style ([`ButtonStyle::icon`]). The Toggle role keeps the older
//! theme path until the toggle gets its own style.

use xgpui_luma_style::State::*;
use xgpui_luma_style::{Color, FontFamily, Px, Shadow, StateSet, States, Style, V, Weight, none, px};

use crate::theme::style_tokens::tok::{metrics as m, palette as p, typography as t};

/// SDK button face.
#[derive(Style, Clone, Debug)]
#[style(path = "button")]
pub struct ButtonStyle {
    /// Focus-ring paint.
    pub focus_border: V<Color>,
    pub background: States<V<Color>>,
    pub foreground: States<V<Color>>,
    /// Secondary text inside the face.
    pub muted_foreground: V<Color>,
    /// Face border; `None` is borderless.
    pub border: V<Option<Color>>,
    pub shadow: V<Option<Shadow>>,
    pub font_family: V<FontFamily>,
    #[style(range = 8.0..=32.0)]
    pub font_size: V<Px>,
    #[style(range = 8.0..=48.0)]
    pub line_height: V<Px>,
    pub font_weight: V<Weight>,
    #[style(range = 0.0..=999.0)]
    pub radius: V<Px>,
    #[style(range = 0.0..=32.0)]
    pub padding_x: V<Px>,
    #[style(range = 0.0..=32.0)]
    pub padding_y: V<Px>,
    /// Space between icon and label.
    #[style(range = 0.0..=32.0)]
    pub gap: V<Px>,
    #[style(range = 16.0..=64.0)]
    pub height: V<Px>,
    #[style(range = 8.0..=48.0)]
    pub icon_size: V<Px>,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            focus_border: p::FOCUS_RING.into(),
            background: States::new(p::SURFACE_SUBTLE_BACKGROUND)
                .on(Hovered, p::STATE_HOVER_BACKGROUND)
                .on(Pressed, p::STATE_PRESSED_BACKGROUND)
                .on(Disabled, p::STATE_DISABLED_BACKGROUND),
            foreground: States::new(p::APP_FOREGROUND).on(Disabled, p::STATE_DISABLED_FOREGROUND),
            muted_foreground: p::APP_MUTED_FOREGROUND.into(),
            // A color token for an optional field: the token resolves to `Some`.
            border: V::Token(p::BORDER_DEFAULT.path),
            shadow: none(),
            font_family: t::SANS.into(),
            font_size: t::LABEL_SIZE.into(),
            line_height: t::LABEL_LINE_HEIGHT.into(),
            font_weight: t::LABEL_WEIGHT.into(),
            radius: m::RADIUS.into(),
            // Control-scale lengths snap to device pixels; authored values stay exact.
            padding_x: m::PADDING_X.snapped(),
            padding_y: m::PADDING_Y.snapped(),
            gap: m::GAP.snapped(),
            height: m::CONTROL_HEIGHT.snapped(),
            // The label size token, not `font_size`: an authored font size does
            // not move the icon, as before.
            icon_size: t::LABEL_SIZE.into(),
        }
    }
}

impl ButtonStyle {
    /// The default with a common stylesheet's authored `[common.button]`
    /// geometry applied, so apps and looks that author it keep working.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let mut style = Self::default();
        style.apply_stylesheet(stylesheet);
        style
    }

    fn apply_stylesheet(&mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) {
        let authored = stylesheet.button.authored_geometry();
        for (path, value) in [
            ("height", authored.height),
            ("padding_x", authored.padding_x),
            ("padding_y", authored.padding_y),
            ("gap", authored.gap),
            ("icon_size", authored.icon_size),
            ("font_size", authored.font_size),
            ("line_height", authored.line_height),
        ] {
            if let Some(value) = value {
                let applied = self.set(path, StateSet::NONE, xgpui_luma_style::Dyn::Number(value));
                debug_assert!(applied, "button style has no numeric field `{path}`");
            }
        }
    }

    /// The icon-only variant: a square face with a pill radius, no padding,
    /// and the control icon size.
    pub fn icon(&self) -> Self {
        let mut style = self.clone();
        style.radius = px(crate::theme::FULL_RADIUS_PX).into();
        style.padding_x = px(0.0).into();
        style.padding_y = px(0.0).into();
        style.icon_size = m::ICON_SIZE.snapped();
        style
    }

    /// The icon variant of a stylesheet-authored style: authored geometry
    /// still wins over the icon defaults, as before.
    pub fn icon_from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let mut style = Self::default().icon();
        style.apply_stylesheet(stylesheet);
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::button_family::{
        ButtonFamilyRole, ButtonFamilyTheme, DefaultButtonFamilyTheme, resolved_from_look, theme_look,
    };
    use crate::theme::{InteractionState, ThemeTokens};

    fn every_interaction_state() -> impl Iterator<Item = InteractionState> {
        (0..32).map(|bits| InteractionState {
            hovered: bits & 1 != 0,
            pressed: bits & 2 != 0,
            focused: bits & 4 != 0,
            disabled: bits & 8 != 0,
            invalid: bits & 16 != 0,
        })
    }

    fn assert_parity(theme: &DefaultButtonFamilyTheme, scale_factors: &[f32], case: &str) {
        for role in [ButtonFamilyRole::Text, ButtonFamilyRole::Icon] {
            for &scale_factor in scale_factors {
                for state in every_interaction_state() {
                    let expected = resolved_from_look(&theme_look(theme, role, state, scale_factor));
                    let actual = theme.resolved_style(role, state, scale_factor);
                    assert_eq!(actual, expected, "{case} {role:?} @{scale_factor}x {state:?}");
                }
            }
        }
    }

    #[test]
    fn matches_the_current_sdk_button_theme() {
        for (name, tokens) in [("light", ThemeTokens::light()), ("dark", ThemeTokens::dark())] {
            assert_parity(&DefaultButtonFamilyTheme::new(tokens), &[1.0, 1.25, 1.5, 2.0], name);
        }
    }

    #[test]
    fn authored_stylesheet_geometry_still_applies() {
        let mut tokens = ThemeTokens::light();
        let mut sheet = (*tokens.stylesheet).clone();
        let geometry = &mut sheet.button.geometry;
        geometry.height = Some(40.0);
        geometry.padding_x = Some(10.0);
        geometry.gap = Some(4.0);
        geometry.icon_size = Some(18.0);
        geometry.font_size = Some(15.0);
        geometry.line_height = Some(22.0);
        tokens.stylesheet = std::sync::Arc::new(sheet);
        assert_parity(&DefaultButtonFamilyTheme::new(tokens), &[1.0, 1.25, 1.5, 2.0], "authored");
    }

    #[test]
    fn icon_differs_only_in_radius_padding_and_icon_size() {
        let tokens = ThemeTokens::light();
        let source = crate::theme::style_tokens::ThemeTokenSource::for_family(&tokens, "button");
        let cx = xgpui_luma_style::ResolveCx::new(&source, 1.0);
        let text = ButtonStyle::default().resolve(&cx, StateSet::NONE);
        let icon = ButtonStyle::default().icon().resolve(&cx, StateSet::NONE);
        assert_eq!(icon.radius, crate::theme::FULL_RADIUS_PX);
        assert_eq!((icon.padding_x, icon.padding_y), (0.0, 0.0));
        assert_eq!(
            ResolvedButtonStyle {
                radius: text.radius,
                padding_x: text.padding_x,
                padding_y: text.padding_y,
                icon_size: text.icon_size,
                ..icon
            },
            text
        );
    }

    #[test]
    fn is_registered_for_tools() {
        assert!(xgpui_luma_style::registered().any(|shape| shape.name == "button"));
    }
}
