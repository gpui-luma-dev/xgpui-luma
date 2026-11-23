use std::sync::{Arc, OnceLock};

use gpui::Hsla;

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
