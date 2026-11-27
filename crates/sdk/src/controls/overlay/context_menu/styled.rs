//! The context-menu target read from `.lss` (issue #285): `context_menu.lss`. The menu is supplied
//! separately (a floating menu, which has its own `.lss`).

use super::theme::ContextMenuTheme;
use crate::controls::popup_menu::MenuSource;
use crate::style::{StyleBinding, ThemeSource};

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
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn menu(&self) -> crate::controls::floating_menu::FloatingMenuLook {
        (self.menu)()
    }
}
