use std::sync::{Arc, OnceLock};

use crate::style::StyleBinding;
use crate::theme::MetricTokens;

pub trait ContextMenuTheme: Send + Sync {
    /// The target's style (#433): the binding the template resolves in the control's states and draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("context_menu")
    }

    /// The style of the menu the target opens (the `floating_menu` control).
    fn menu(&self) -> StyleBinding {
        StyleBinding::sdk("floating_menu")
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledContextMenuTheme::sdk())).clone()
}
