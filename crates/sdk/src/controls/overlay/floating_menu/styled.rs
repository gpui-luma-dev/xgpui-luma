//! The floating menu read from `.lss` (issue #285): `floating_menu.lss`. Menus, selector popups and
//! selection panels build their looks from this one.

use std::sync::Arc;

use super::theme::FloatingMenuTheme;
use crate::style::StyleBinding;

/// A floating-menu theme read from `.lss`.
#[derive(Clone)]
pub struct StyledFloatingMenuTheme {
    binding: StyleBinding,
}

impl StyledFloatingMenuTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("floating_menu") }
    }
}

impl FloatingMenuTheme for StyledFloatingMenuTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}

/// The SDK's floating-menu theme: `floating_menu.lss` alone.
pub(super) fn sdk_theme() -> Arc<dyn FloatingMenuTheme> {
    Arc::new(StyledFloatingMenuTheme::sdk())
}

/// The menu a popup opens, read from `source`'s `floating_menu.lss` each time it is drawn (#397): the menu hook
/// for popup menus, split buttons, context menus and autocompletes, so they open the host's menu, not the SDK's.
pub fn floating_menu_source(source: crate::style::ThemeSource) -> crate::controls::popup_menu::MenuSource {
    let binding = StyleBinding::new("floating_menu", source);
    Arc::new(move || binding.clone())
}

#[cfg(test)]
mod tests {}
