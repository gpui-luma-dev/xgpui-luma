//! Context menu property mappings:
//!
//! | Part   | Token                              |
//! |--------|------------------------------------|
//! | Target | ghost (accent-foreground on hover) |
//! | Menu   | floating menu surface              |

/// Native common geometry owned by this control.
pub(crate) fn native_context_menu_stylesheet() -> xgpui_luma::theme::stylesheet::ContextMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ContextMenuStylesheet {
        geometry: ContextMenuGeometryStylesheet { min_width: Some(200.0), ..Default::default() },
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct ContextMenuMetricTable {
    pub target_padding_x: crate::ResolvedMetric,
    pub target_padding_y: crate::ResolvedMetric,
    pub target_radius: crate::ResolvedMetric,
    pub target_min_width: crate::ResolvedMetric,
    pub menu: crate::controls::floating_menu::FloatingMenuMetricTable,
}
