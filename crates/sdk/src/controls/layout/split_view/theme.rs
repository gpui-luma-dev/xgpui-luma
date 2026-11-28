#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

#[deprecated(note = "the split view draws from `SplitViewTheme::style()`; resolve it with `resolve_split_view` (#433)")]
#[derive(Clone, Debug)]
pub struct SplitViewLook {
    /// Resolved separator hit target and decoration in pixels.
    pub geometry: crate::theme::stylesheet::SplitViewGeometry,
    pub separator: Hsla,
    pub separator_hover: Hsla,
}

pub trait SplitViewTheme: Send + Sync {
    /// The split view's style (#433): the binding the template resolves (see
    /// [`resolve_split_view`](super::resolve_split_view)) and draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("split_view")
    }

    /// The split view's look, built from [`Self::style`] for code that still reads a [`SplitViewLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_split_view` (#433)")]
    fn resolve(&self, _hovered: bool, enabled: bool) -> SplitViewLook {
        SplitViewLook::from_resolved(&super::resolve_split_view(&self.style(), enabled))
    }
}

pub fn default_split_view_theme() -> Arc<dyn SplitViewTheme> {
    static THEME: OnceLock<Arc<dyn SplitViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledSplitViewTheme::sdk())).clone()
}

/// The split view's geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SplitViewGeometry {
    pub hitbox_width: f32,
    pub expand_inset_y: f32,
    pub expand_cue_left: f32,
    pub cue_inset_y: f32,
    pub cue_radius: f32,
    pub cue_width: f32,
    pub cue_hovered_width: f32,
}
