use std::sync::{Arc, OnceLock};

use gpui::Hsla;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarVariant {
    /// Bordered, filled shell.
    #[default]
    Outline,
    /// Transparent / borderless shell for denser embedding.
    Ghost,
}

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
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook;
}

pub fn default_toolbar_theme() -> Arc<dyn ToolbarTheme> {
    static THEME: OnceLock<Arc<dyn ToolbarTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(super::styled::StyledToolbarTheme::sdk())).clone()
}

crate::theme::stylesheet::geometry::geometry_family!(
    ToolbarStylesheet,
    ToolbarGeometryStylesheet,
    ToolbarGeometry,
    ResolvedToolbarGeometry,
    "toolbar",
    [padding_x, padding_y, gap, border_width, separator_width, radius]
);
