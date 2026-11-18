use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Debug)]
pub struct SplitViewLook {
    /// Resolved separator hit target and decoration in pixels.
    pub geometry: crate::theme::stylesheet::SplitViewGeometry,
    pub separator: Hsla,
    pub separator_hover: Hsla,
}

pub trait SplitViewTheme: Send + Sync {
    fn resolve(&self, hovered: bool, enabled: bool) -> SplitViewLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSplitViewTheme {
    tokens: ThemeTokens,
}

pub fn default_split_view_theme() -> Arc<dyn SplitViewTheme> {
    static THEME: OnceLock<Arc<dyn SplitViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSplitViewTheme::default())).clone()
}

impl DefaultSplitViewTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SplitViewTheme for DefaultSplitViewTheme {
    fn resolve(&self, _hovered: bool, enabled: bool) -> SplitViewLook {
        let palette = self.tokens.stylesheet.recipes.palette("split_view", &self.tokens.palette);
        let separator = if enabled {
            palette.border.default
        } else {
            palette.state.disabled.background
        };
        let separator_hover = if enabled {
            palette.border.strong
        } else {
            palette.state.disabled.foreground
        };

        SplitViewLook {
            geometry: self.tokens.stylesheet.split_view.resolve_geometry(Default::default()),
            separator,
            separator_hover,
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::{DefaultSplitViewTheme, SplitViewTheme};

    #[test]
    fn default_theme_uses_stronger_border_when_hovered_path_is_resolved() {
        let theme = DefaultSplitViewTheme::default();
        let idle = theme.resolve(false, true);
        let hover = theme.resolve(true, true);

        assert_ne!(idle.separator, hover.separator_hover);
        assert_eq!(hover.separator_hover, theme.tokens.palette.border.strong);
    }

    #[test]
    fn disabled_theme_mutes_separator_colors() {
        let theme = DefaultSplitViewTheme::default();
        let enabled = theme.resolve(false, true);
        let disabled = theme.resolve(false, false);

        assert_ne!(enabled.separator, disabled.separator);
        assert_eq!(disabled.separator, theme.tokens.palette.state.disabled.background);
    }

    #[test]
    fn disabled_theme_does_not_panic_with_default_tokens() {
        let theme = DefaultSplitViewTheme::default();
        let look = theme.resolve(false, false);
        assert!(look.separator.a >= 0.0);
        assert!(look.separator_hover.a >= 0.0);
        let _ = hsla(0.0, 0.0, 0.0, look.separator.a);
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    SplitViewStylesheet,
    SplitViewGeometryStylesheet,
    SplitViewGeometry,
    ResolvedSplitViewGeometry,
    "split_view",
    [hitbox_width, expand_inset_y, expand_cue_left, cue_inset_y, cue_radius, cue_width, cue_hovered_width]
);

impl SplitViewStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SplitViewGeometryStylesheet {
                hitbox_width: Some(20.0),
                expand_inset_y: Some(10.0),
                expand_cue_left: Some(2.0),
                cue_inset_y: Some(16.0),
                cue_radius: Some(4.0),
                cue_width: Some(4.0),
                cue_hovered_width: Some(8.0),
            },
            ..Default::default()
        }
    }
}
