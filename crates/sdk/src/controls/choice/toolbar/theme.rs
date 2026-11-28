use std::sync::{Arc, OnceLock};

use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarVariant {
    /// Bordered, filled shell.
    #[default]
    Outline,
    /// Transparent / borderless shell for denser embedding.
    Ghost,
}

pub trait ToolbarTheme: Send + Sync {
    /// The `toolbar` binding the shell is read from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("toolbar")
    }

    /// The shell's resolved style, enabled or not, in `variant` (#433).
    fn resolve_style(&self, enabled: bool, variant: ToolbarVariant) -> Arc<Resolved> {
        super::styled::resolve_toolbar(&self.style(), enabled, variant)
    }
}

pub fn default_toolbar_theme() -> Arc<dyn ToolbarTheme> {
    static THEME: OnceLock<Arc<dyn ToolbarTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(super::styled::StyledToolbarTheme::sdk())).clone()
}
