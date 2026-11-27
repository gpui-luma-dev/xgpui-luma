//! The context-menu target read from `.lss` (issue #285): `context_menu.lss`. The menu is supplied
//! separately (a floating menu, which has its own `.lss`).

use crate::style::PartRead;

use super::theme::{ContextMenuLook, ContextMenuTheme};
use crate::controls::popup_menu::MenuSource;
use crate::style::{StyleBinding, ThemeSource};
use crate::theme::InteractionState;

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
            floating_menu: (self.menu)(),
        }
    }
}
