use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::FloatingMenuLook;
use crate::style::{PartRead, StyleBinding, StyleStates};
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens};
use xgpui_luma_style::Resolved;

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

impl ContextMenuLook {
    /// The look `resolved` describes, with `floating_menu` as the menu: the compatibility path for code that
    /// still reads a `ContextMenuLook`. Templates read the resolved style directly (#433).
    pub fn from_resolved(resolved: &Resolved, floating_menu: FloatingMenuLook) -> Self {
        let root = resolved.root();
        Self {
            border_width: root.number("border_width"),
            opacity: root.opacity(),
            window_margin: root.number("window_margin"),
            target_background: root.color("background"),
            target_foreground: root.color("foreground"),
            target_border: root.color("border"),
            target_typography: root.typography(14.0, 20.0),
            target_radius: root.number("radius"),
            target_padding_x: root.number("padding_x"),
            target_padding_y: root.number("padding_y"),
            target_min_width: root.number("min_width"),
            floating_menu,
        }
    }
}

pub trait ContextMenuTheme: Send + Sync {
    /// The target's style (#433): the binding the template resolves in the control's states and draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("context_menu")
    }

    /// The menu the target opens. The floating menu moves to its own style separately.
    fn menu(&self) -> FloatingMenuLook {
        crate::controls::floating_menu::default_floating_menu_theme().resolve()
    }

    /// The look in `state`, built from [`Self::style`] for code that still reads a [`ContextMenuLook`].
    #[deprecated(note = "templates draw from `style()`; resolve that binding and read its parts instead (#433)")]
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        ContextMenuLook::from_resolved(&self.style().resolve_states(StyleStates::new(state), 1.0), self.menu())
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledContextMenuTheme::sdk())).clone()
}
