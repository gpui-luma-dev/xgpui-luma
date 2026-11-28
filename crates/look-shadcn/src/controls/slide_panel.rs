use gpui::Hsla;

use crate::ShadcnLook;

/// The drawer's resize chrome: the resizable panels' resolved style (#433).
pub fn slide_panel_panels_style(look: &ShadcnLook) -> std::sync::Arc<xgpui_luma::style::lss::Resolved> {
    xgpui_luma::controls::resizable_panels::resolve_resizable_panels(
        &look.resizable_panels_theme().style(),
        Default::default(),
    )
}

pub fn slide_panel_background(look: &ShadcnLook) -> Hsla {
    look.token_color("card").unwrap_or_else(|_| look.chrome().panel_background)
}
