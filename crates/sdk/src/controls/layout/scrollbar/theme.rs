#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

use crate::theme::{InteractionState, MetricTokens};

#[deprecated(
    note = "scrollbar templates draw from `ScrollbarTheme::style()`; resolve it with `resolve_scrollbar` (#433)"
)]
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
    /// The scrollbar's style (#433): the binding templates resolve (see
    /// [`resolve_scrollbar`](super::resolve_scrollbar)) and draw from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("scrollbar")
    }

    /// The scrollbar's style in `state`: [`Self::style`] unless a theme sets values per state (with
    /// [`StyleBinding::with_overrides`](crate::style::StyleBinding::with_overrides)).
    fn state_style(&self, _state: InteractionState) -> crate::style::StyleBinding {
        self.style()
    }

    /// The scrollbar's look, built from [`Self::style`] for code that still reads a [`ScrollbarLook`].
    #[deprecated(note = "templates draw from `style()`; resolve it with `resolve_scrollbar` (#433)")]
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        ScrollbarLook::from_resolved(&super::resolve_scrollbar(&self.state_style(state), state, style), orientation)
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_scrollbar_theme() -> Arc<dyn ScrollbarTheme> {
    static THEME: OnceLock<Arc<dyn ScrollbarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledScrollbarTheme::sdk())).clone()
}
