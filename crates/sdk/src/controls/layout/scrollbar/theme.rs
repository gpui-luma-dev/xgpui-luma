use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

use crate::theme::{InteractionState, MetricTokens};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarLook {
    /// Whole-scrollbar opacity when disabled.
    pub disabled_opacity: f32,
    pub track_background: Hsla,
    pub thumb_background: Hsla,
    pub length: f32,
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
    pub radius: f32,
}

pub trait ScrollbarTheme: Send + Sync {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        style: ScrollbarStyle,
    ) -> ScrollbarLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_scrollbar_theme() -> Arc<dyn ScrollbarTheme> {
    static THEME: OnceLock<Arc<dyn ScrollbarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledScrollbarTheme::sdk())).clone()
}

crate::theme::stylesheet::geometry::geometry_family!(
    ScrollbarStylesheet,
    ScrollbarGeometryStylesheet,
    ScrollbarGeometry,
    ResolvedScrollbarGeometry,
    "scrollbar",
    [thickness, track_thickness, thumb_thickness, min_thumb_length, length_h, length_v],
    unit[disabled_opacity, thumb_fraction]
);

impl ScrollbarStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ScrollbarGeometryStylesheet {
                thumb_fraction: Some(0.25),
                disabled_opacity: Some(0.56),
                thickness: Some(12.0),
                track_thickness: Some(4.0),
                thumb_thickness: Some(8.0),
                min_thumb_length: Some(28.0),
                length_h: Some(260.0),
                length_v: Some(180.0),
            },
            sizes: Default::default(),
        }
    }
}
