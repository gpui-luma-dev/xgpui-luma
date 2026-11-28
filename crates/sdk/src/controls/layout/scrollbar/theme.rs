#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use crate::theme::{InteractionState, MetricTokens};

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

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_scrollbar_theme() -> Arc<dyn ScrollbarTheme> {
    static THEME: OnceLock<Arc<dyn ScrollbarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledScrollbarTheme::sdk())).clone()
}
