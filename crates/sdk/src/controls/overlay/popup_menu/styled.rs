//! The popup-menu trigger read from `.lss` (issue #285): `popup_menu.lss`, which extends the button. The
//! menu is supplied separately (a floating menu, which has its own `.lss`).

use std::sync::Arc;

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value, box_shadows};

use super::PopupMenuTriggerStyle;
use super::theme::{PopupMenuLook, PopupMenuPalette, PopupMenuTheme, PopupMenuTriggerMetrics};
use crate::controls::floating_menu::FloatingMenuLook;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{FocusMetricTokens, InteractionState, LumaTextStyle, MetricTokens};

/// Supplies a popup's menu look.
pub type MenuSource = Arc<dyn Fn() -> FloatingMenuLook + Send + Sync>;

fn color(root: PartView<'_>, name: &str) -> Option<Hsla> {
    root.value(name).and_then(Value::as_color).map(Hsla::from)
}

fn number(root: PartView<'_>, name: &str) -> f32 {
    root.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

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
            menu: Arc::new(|| crate::controls::floating_menu::default_floating_menu_theme().resolve()),
        }
    }

    /// Change the binding (a look's size or tone).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding), ..self }
    }

    fn resolve_root(
        &self,
        style: PopupMenuTriggerStyle,
        state: InteractionState,
        open: bool,
        scale: f32,
    ) -> Arc<xgpui_luma_style::Resolved> {
        let variant: &[(&str, &str)] = match style {
            PopupMenuTriggerStyle::Primary => &[],
            PopupMenuTriggerStyle::Secondary => &[("variant", "secondary")],
            PopupMenuTriggerStyle::Outline => &[("variant", "outline")],
            PopupMenuTriggerStyle::Ghost => &[("variant", "ghost")],
        };
        let states = [
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
            ("invalid", state.invalid),
            ("open", open),
        ];
        self.binding.resolve_for(variant, &states, scale)
    }
}

impl PopupMenuTheme for StyledPopupMenuTheme {
    fn resolve(
        &self,
        style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let resolved = self.resolve_root(style, state, metrics.open, 1.0);
        let root = resolved.root();
        let transparent = gpui::transparent_black();
        PopupMenuPalette {
            trigger_focus_border: color(root, "focus_border").unwrap_or(transparent),
            trigger_background: color(root, "background").unwrap_or(transparent),
            trigger_foreground: color(root, "foreground").unwrap_or(transparent),
            trigger_border: color(root, "border"),
            trigger_shadow: if metrics.without_elevation {
                None
            } else {
                root.shadow().filter(|layers| !layers.is_empty()).map(|layers| box_shadows(&layers))
            },
            trigger_typography: LumaTextStyle {
                size: root.font_size().unwrap_or(14.0),
                line_height: root.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            floating_menu: (self.menu)(),
        }
    }

    /// The focus geometry and border width, for templates that read them from the theme's metrics.
    fn metrics(&self) -> MetricTokens {
        let resolved = self.resolve_root(PopupMenuTriggerStyle::Outline, InteractionState::default(), false, 1.0);
        let root = resolved.root();
        MetricTokens {
            border_width: number(root, "border_width"),
            focus: FocusMetricTokens {
                gap: number(root, "focus_gap"),
                width: number(root, "focus_width"),
                offset: number(root, "focus_offset"),
            },
            ..MetricTokens::default()
        }
    }

    fn resolve_look(
        &self,
        style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> PopupMenuLook {
        let palette = self.resolve(style, metrics, state);
        let resolved = self.resolve_root(style, state, metrics.open, scale_factor);
        let root = resolved.root();
        PopupMenuLook {
            trigger_focus_border: palette.trigger_focus_border,
            trigger_background: palette.trigger_background,
            trigger_foreground: palette.trigger_foreground,
            trigger_icon: color(root, "icon").unwrap_or(palette.trigger_foreground),
            trigger_border: palette.trigger_border,
            trigger_shadow: palette.trigger_shadow,
            trigger_typography: palette.trigger_typography,
            trigger_radius: metrics.trigger_radius_override.unwrap_or_else(|| root.radius().unwrap_or(0.0)),
            trigger_padding_x: number(root, "padding_x"),
            trigger_padding_y: number(root, "padding_y"),
            trigger_gap: root.gap().unwrap_or(0.0),
            trigger_height: root.height().unwrap_or(0.0),
            trigger_icon_size: number(root, "icon_size"),
            menu_offset_y: number(root, "menu_offset_y"),
            trigger_opacity: crate::style::part_opacity(root),
            floating_menu: palette.floating_menu,
        }
    }
}
