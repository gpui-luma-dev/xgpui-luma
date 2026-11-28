//! The popup-menu trigger read from `.lss` (issue #285): `popup_menu.lss`, which extends the button. The
//! menu is supplied separately (a floating menu, which has its own `.lss`).
// Builds the compatibility `PopupMenuLook` (#433).
#![allow(deprecated)]

use crate::style::StyleStates;
use crate::style::PartRead;
use std::sync::Arc;

use super::PopupMenuTriggerStyle;
use super::theme::{PopupMenuLook, PopupMenuPalette, PopupMenuTheme, PopupMenuTriggerMetrics};
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

/// The palette `binding` gives, for code that still reads a [`PopupMenuPalette`].
pub(crate) fn palette(
    binding: &StyleBinding,
    menu: StyleBinding,
    trigger_style: PopupMenuTriggerStyle,
    metrics: PopupMenuTriggerMetrics,
    state: InteractionState,
) -> PopupMenuPalette {
    let resolved = resolve_trigger(binding, trigger_style, state, metrics.open, 1.0);
    let root = resolved.root();
    let transparent = gpui::transparent_black();
    PopupMenuPalette {
        trigger_focus_border: root.color_opt("focus_border").unwrap_or(transparent),
        trigger_background: root.color_opt("background").unwrap_or(transparent),
        trigger_foreground: root.color_opt("foreground").unwrap_or(transparent),
        trigger_border: root.color_opt("border"),
        trigger_shadow: if metrics.without_elevation {
            None
        } else {
            root.box_shadow_opt()
        },
        trigger_typography: root.typography(14.0, 20.0),
        floating_menu: menu,
    }
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

/// The look `binding` gives, for code that still reads a [`PopupMenuLook`].
pub(crate) fn look(
    binding: &StyleBinding,
    menu: StyleBinding,
    trigger_style: PopupMenuTriggerStyle,
    metrics: PopupMenuTriggerMetrics,
    state: InteractionState,
    scale_factor: f32,
) -> PopupMenuLook {
    let palette = palette(binding, menu, trigger_style, metrics, state);
    let resolved = resolve_trigger(binding, trigger_style, state, metrics.open, scale_factor);
    let root = resolved.root();
    PopupMenuLook {
        trigger_focus_border: palette.trigger_focus_border,
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: root.color_opt("icon").unwrap_or(palette.trigger_foreground),
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow,
        trigger_typography: palette.trigger_typography,
        trigger_radius: metrics.trigger_radius_override.unwrap_or_else(|| root.number("radius")),
        trigger_padding_x: root.number("padding_x"),
        trigger_padding_y: root.number("padding_y"),
        trigger_gap: root.number("gap"),
        trigger_height: root.number("height"),
        trigger_icon_size: root.number("icon_size"),
        menu_offset_y: root.number("menu_offset_y"),
        trigger_opacity: root.opacity(),
        floating_menu: palette.floating_menu,
    }
}
