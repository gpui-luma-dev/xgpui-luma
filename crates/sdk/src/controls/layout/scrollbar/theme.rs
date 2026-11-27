use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

use crate::theme::{InteractionState, MetricTokens};

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarLook {
    /// Whole-scrollbar fade in this state: the root's resolved `opacity` (`:disabled { opacity }`), 1 when unset.
    pub opacity: f32,
    pub track_background: Hsla,
    /// The track's outline, under the thumb. The SDK draws none (width 0); a look or design can set one.
    pub track_border: Hsla,
    pub track_border_width: f32,
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
