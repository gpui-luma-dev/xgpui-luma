//! The popup-menu trigger read from `.lss` (issue #285): `popup_menu.lss`, which extends the button. The
//! menu is supplied separately (a floating menu, which has its own `.lss`).
// Builds the compatibility `PopupMenuLook` (#433).

use crate::style::StyleStates;
use crate::style::PartRead;
use std::sync::Arc;

use super::PopupMenuTriggerStyle;
use super::theme::PopupMenuTheme;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, InteractionState, MetricTokens};

/// Supplies a popup's menu style (the `floating_menu` control, #433); menu templates resolve it and draw from it.
pub type MenuSource = Arc<dyn Fn() -> StyleBinding + Send + Sync>;

/// A popup-menu theme whose trigger is read from `.lss`.
#[derive(Clone)]
pub struct StyledPopupMenuTheme {
    binding: StyleBinding,
    menu: MenuSource,
}

impl StyledPopupMenuTheme {
    /// The `popup_menu` control from `source`, with its menu from `menu`.
    pub fn new(source: ThemeSource, menu: MenuSource) -> Self {
        Self { binding: StyleBinding::new("popup_menu", source), menu }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("popup_menu"),
            menu: Arc::new(|| crate::style::StyleBinding::sdk("floating_menu")),
        }
    }

    /// Change the binding (a look's size or tone).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding), ..self }
    }
}

impl PopupMenuTheme for StyledPopupMenuTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn menu(&self) -> StyleBinding {
        (self.menu)()
    }
}

/// The trigger's resolved style in `trigger_style` and `state`, open or not (#433): what the template draws from.
/// Read its parts for the trigger's values (`background`, `foreground`, `icon`, `icon_size`, …).
pub fn resolve_trigger(
    binding: &StyleBinding,
    trigger_style: PopupMenuTriggerStyle,
    state: InteractionState,
    open: bool,
    scale: f32,
) -> Arc<xgpui_luma_style::Resolved> {
    let variant: &[(&str, &str)] = match trigger_style {
        PopupMenuTriggerStyle::Primary => &[],
        PopupMenuTriggerStyle::Secondary => &[("variant", "secondary")],
        PopupMenuTriggerStyle::Outline => &[("variant", "outline")],
        PopupMenuTriggerStyle::Ghost => &[("variant", "ghost")],
    };
    binding.resolve_states_for(variant, StyleStates::new(state).open(open), scale)
}

/// The focus geometry and border width at rest.
pub(crate) fn metrics(binding: &StyleBinding) -> MetricTokens {
    let resolved = resolve_trigger(binding, PopupMenuTriggerStyle::Outline, InteractionState::default(), false, 1.0);
    let root = resolved.root();
    MetricTokens {
        border_width: root.number("border_width"),
        focus: FocusMetricTokens {
            gap: root.number("focus_gap"),
            width: root.number("focus_width"),
            offset: root.number("focus_offset"),
        },
        ..MetricTokens::default()
    }
}
