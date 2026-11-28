use std::sync::{Arc, OnceLock};

use gpui::Hsla;
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

#[deprecated(note = "templates draw from the resolved style; use `ToolbarTheme::resolve_style` (#433)")]
#[derive(Clone, Debug)]
pub struct ToolbarLook {
    /// Shell stroke thickness in pixels.
    pub border_width: f32,
    /// Vertical separator thickness in pixels.
    pub separator_width: f32,
    pub background: Hsla,
    pub border: Hsla,
    pub separator: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub separator_height: f32,
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

    #[deprecated(note = "templates draw from the resolved style; use `resolve_style` (#433)")]
    #[allow(deprecated)]
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
        ToolbarLook::from_resolved(&self.resolve_style(enabled, variant))
    }
}

pub fn default_toolbar_theme() -> Arc<dyn ToolbarTheme> {
    static THEME: OnceLock<Arc<dyn ToolbarTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(super::styled::StyledToolbarTheme::sdk())).clone()
}
