//! Declarative toolbar style (issue #282 spike).
//!
//! Covers the toolbar shell and its separators. Hosted items (buttons,
//! toggles, selectors, menus) are separate controls with their own styles.
//!
//! `ToolbarVariant::Ghost` is still an SDK enum for now; here it is the
//! default style with the background and border removed ([`ToolbarStyle::ghost`]).

use xgpui_luma_style::State::*;
use xgpui_luma_style::{Color, Px, Style, StateSet, States, V, none, px};

use crate::theme::style_tokens::tok::{metrics as m, palette as p};

/// SDK toolbar shell.
#[derive(Style, Clone, Debug)]
#[style(path = "toolbar")]
pub struct ToolbarStyle {
    /// Shell fill.
    pub background: States<V<Color>>,
    /// Shell stroke; `None` draws no border.
    pub border: V<Option<Color>>,
    #[style(range = 0.0..=4.0)]
    pub border_width: V<Px>,
    #[style(range = 0.0..=24.0)]
    pub radius: V<Px>,
    #[style(range = 0.0..=24.0)]
    pub padding_x: V<Px>,
    #[style(range = 0.0..=24.0)]
    pub padding_y: V<Px>,
    /// Space between items.
    #[style(range = 0.0..=24.0)]
    pub gap: V<Px>,
    pub separator: ToolbarSeparatorPart,
}

/// The vertical rule between item groups.
#[derive(Style, Clone, Debug)]
pub struct ToolbarSeparatorPart {
    pub color: V<Color>,
    #[style(range = 0.0..=4.0)]
    pub width: V<Px>,
    #[style(range = 0.0..=64.0)]
    pub height: V<Px>,
}

impl Default for ToolbarStyle {
    fn default() -> Self {
        Self {
            background: States::new(p::SURFACE_SUBTLE_BACKGROUND).on(Disabled, p::STATE_DISABLED_BACKGROUND),
            // A color token for an optional field: the token resolves to `Some`.
            border: V::Token(p::BORDER_DEFAULT.path),
            border_width: m::BORDER_WIDTH.into(),
            radius: px(6.0).into(),
            padding_x: px(6.0).into(),
            padding_y: px(4.0).into(),
            gap: m::GAP.into(),
            separator: ToolbarSeparatorPart {
                color: p::BORDER_DEFAULT.into(),
                width: m::BORDER_WIDTH.into(),
                height: m::HEIGHT.into(),
            },
        }
    }
}

impl ToolbarStyle {
    /// The default with a common stylesheet's authored `[common.toolbar]`
    /// geometry applied, so apps and looks that author it keep working.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let authored = stylesheet.toolbar.authored_geometry();
        let mut style = Self::default();
        for (path, value) in [
            ("padding_x", authored.padding_x),
            ("padding_y", authored.padding_y),
            ("gap", authored.gap),
            ("border_width", authored.border_width),
            ("separator.width", authored.separator_width),
            ("radius", authored.radius),
        ] {
            if let Some(value) = value {
                let applied = style.set(path, StateSet::NONE, xgpui_luma_style::Dyn::Number(value));
                debug_assert!(applied, "toolbar style has no numeric field `{path}`");
            }
        }
        style
    }

    /// The ghost variant: no fill and no border.
    pub fn ghost(&self) -> Self {
        let mut style = self.clone();
        style.background = States::new(Color(gpui::transparent_black()));
        style.border = none();
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::toolbar::{DefaultToolbarTheme, ToolbarTheme, ToolbarVariant, resolved_from_look};
    use crate::theme::ThemeTokens;

    #[test]
    fn matches_the_current_sdk_toolbar_theme() {
        for (name, tokens) in [("light", ThemeTokens::light()), ("dark", ThemeTokens::dark())] {
            let theme = DefaultToolbarTheme::new(tokens);
            for variant in [ToolbarVariant::Outline, ToolbarVariant::Ghost] {
                for enabled in [true, false] {
                    let expected = resolved_from_look(&theme.resolve(enabled, variant), variant);
                    let actual = theme.resolved_style(enabled, variant);
                    assert_eq!(actual, expected, "{name} {variant:?} enabled={enabled}");
                }
            }
        }
    }

    #[test]
    fn authored_stylesheet_geometry_still_applies() {
        let mut tokens = ThemeTokens::light();
        let mut sheet = (*tokens.stylesheet).clone();
        sheet.toolbar.geometry.padding_x = Some(10.0);
        sheet.toolbar.geometry.separator_width = Some(2.0);
        sheet.toolbar.geometry.radius = Some(0.0);
        tokens.stylesheet = std::sync::Arc::new(sheet);
        let theme = DefaultToolbarTheme::new(tokens);
        for variant in [ToolbarVariant::Outline, ToolbarVariant::Ghost] {
            let expected = resolved_from_look(&theme.resolve(true, variant), variant);
            assert_eq!(theme.resolved_style(true, variant), expected, "{variant:?}");
        }
    }

    #[test]
    fn ghost_differs_only_in_fill_and_border() {
        let outline = ToolbarStyle::default();
        let ghost = outline.ghost();
        let tokens = ThemeTokens::light();
        let source = crate::theme::style_tokens::ThemeTokenSource::for_family(&tokens, "toolbar");
        let cx = xgpui_luma_style::ResolveCx::new(&source, 1.0);
        let (outline, ghost) = (outline.resolve(&cx, StateSet::NONE), ghost.resolve(&cx, StateSet::NONE));
        assert_eq!(ghost.border, None);
        assert_eq!(ghost.background, gpui::transparent_black());
        assert_eq!(
            ResolvedToolbarStyle { background: outline.background, border: outline.border, ..ghost },
            outline,
            "everything else is shared"
        );
    }

    #[test]
    fn is_registered_for_tools() {
        assert!(xgpui_luma_style::registered().any(|shape| shape.name == "toolbar"));
    }
}
