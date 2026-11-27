use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::FloatingMenuLook;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens};

#[derive(Clone, Debug)]
pub struct ContextMenuLook {
    /// Target stroke thickness in pixels.
    pub border_width: f32,
    /// The target's opacity: the root's resolved `opacity`, which a look sets under `:disabled`.
    pub opacity: f32,
    /// Minimum popup distance from window edges in pixels.
    pub window_margin: f32,
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub floating_menu: FloatingMenuLook,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledContextMenuTheme::sdk())).clone()
}
