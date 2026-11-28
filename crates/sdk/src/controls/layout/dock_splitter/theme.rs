use std::sync::{Arc, OnceLock};

/// The dock splitter's geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DockSplitterGeometry {
    pub hit_target: f32,
    pub visible_line: f32,
    pub thumb_width: f32,
    pub thumb_height: f32,
    pub thumb_radius: f32,
    pub thumb_rest_opacity: f32,
}

pub trait DockSplitterTheme: Send + Sync {
    /// The splitter's style (#433): the binding the template resolves (see
    /// [`resolve_dock_splitter`](super::resolve_dock_splitter)) and draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("dock_splitter")
    }
}

pub fn default_dock_splitter_theme() -> Arc<dyn DockSplitterTheme> {
    static THEME: OnceLock<Arc<dyn DockSplitterTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledDockSplitterTheme::sdk())).clone()
}
