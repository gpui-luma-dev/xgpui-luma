//! The context-menu target read from `.lss` (issue #285): `context_menu.lss`. The menu is supplied
//! separately (a floating menu, which has its own `.lss`).

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::theme::{ContextMenuLook, ContextMenuTheme};
use crate::controls::popup_menu::MenuSource;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::{InteractionState, LumaTextStyle};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// A context-menu theme whose target is read from `.lss`.
#[derive(Clone)]
pub struct StyledContextMenuTheme {
    binding: StyleBinding,
    menu: MenuSource,
}

impl StyledContextMenuTheme {
    /// The `context_menu` control from `source`, with its menu from `menu`.
    pub fn new(source: ThemeSource, menu: MenuSource) -> Self {
        Self { binding: StyleBinding::new("context_menu", source), menu }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("context_menu"),
            menu: std::sync::Arc::new(|| crate::controls::floating_menu::default_floating_menu_theme().resolve()),
        }
    }

    /// Change the binding (a look's tone).
    pub fn with(self, f: impl FnOnce(StyleBinding) -> StyleBinding) -> Self {
        Self { binding: f(self.binding), ..self }
    }
}

impl ContextMenuTheme for StyledContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        let states = [
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let root = resolved.root();
        ContextMenuLook {
            border_width: number(root, "border_width"),
            disabled_opacity: number(root, "disabled_opacity"),
            window_margin: number(root, "window_margin"),
            target_background: color(root, "background"),
            target_foreground: color(root, "foreground"),
            target_border: color(root, "border"),
            target_typography: LumaTextStyle {
                size: root.font_size().unwrap_or(14.0),
                line_height: root.line_height().unwrap_or(20.0),
                weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            },
            target_radius: number(root, "radius"),
            target_padding_x: number(root, "padding_x"),
            target_padding_y: number(root, "padding_y"),
            target_min_width: number(root, "min_width"),
            floating_menu: (self.menu)(),
        }
    }
}
