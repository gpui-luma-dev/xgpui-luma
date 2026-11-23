use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::button::{ButtonTemplate, default_button_template};
use crate::theme::LumaTextStyle;

use super::PagerStyle;

#[derive(Clone, Debug)]
pub struct PagerLook {
    /// Gap-jump glyph size in pixels.
    pub gap_icon_size: f32,
    /// Page-size border width in pixels.
    pub border_width: f32,
    /// Gap between the page-size label and trigger, in pixels.
    pub page_size_label_gap: f32,
    /// Vertical gap below the page-size trigger, in pixels.
    pub popup_offset_y: f32,
    /// Page-size popup gap in pixels.
    pub popup_gap: f32,
    /// Page-size popup padding in pixels.
    pub popup_padding: f32,
    /// Page-size popup item padding y in pixels.
    pub popup_item_padding_y: f32,
    /// Amount subtracted from the popup radius for each item, in pixels.
    pub popup_item_radius_inset: f32,

    pub panel_background: Hsla,
    pub border: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub hover_background: Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub typography: LumaTextStyle,
    pub button_size: f32,
    pub button_min_width: f32,
    pub control_height: f32,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub group_gap: f32,
    pub disabled_opacity: f32,
}

pub trait PagerTheme: Send + Sync {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook;

    fn button_template(&self, _pager_look: &PagerLook) -> Arc<dyn ButtonTemplate<()>> {
        default_button_template()
    }
}

pub fn default_pager_theme() -> Arc<dyn PagerTheme> {
    static THEME: OnceLock<Arc<dyn PagerTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledPagerTheme::sdk())).clone()
}
