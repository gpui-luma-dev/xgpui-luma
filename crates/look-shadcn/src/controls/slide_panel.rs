use gpui::Hsla;

use xgpui_luma::controls::resizable_panels::ResizablePanelsLook;
use xgpui_luma::theme::InteractionState;

use crate::ShadcnLook;

pub fn slide_panel_panels_look(look: &ShadcnLook) -> ResizablePanelsLook {
    look.resizable_panels_theme().resolve(InteractionState::default())
}

pub fn slide_panel_background(look: &ShadcnLook) -> Hsla {
    look.token_color("card").unwrap_or_else(|_| look.chrome().panel_background)
}
