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
