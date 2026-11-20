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

crate::theme::stylesheet::geometry::geometry_family!(
    PagerStylesheet,
    PagerGeometryStylesheet,
    PagerGeometry,
    ResolvedPagerGeometry,
    "pager",
    [
        gap_icon_size,
        button_size,
        button_min_width,
        control_height,
        padding_x,
        padding_y,
        gap,
        group_gap,
        font_size,
        line_height,
        radius,
        padding_x_inset_scale,
        padding_y_gap_scale,
        button_extra_width_scale,
        border_width,
        page_size_label_gap,
        popup_offset_y,
        popup_gap,
        popup_padding,
        popup_item_padding_y,
        popup_item_radius_inset
    ],
    unit[disabled_opacity]
);

impl PagerStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PagerGeometryStylesheet {
                gap_icon_size: Some(16.0),
                border_width: Some(1.0),
                page_size_label_gap: Some(8.0),
                popup_offset_y: Some(4.0),
                popup_gap: Some(2.0),
                popup_padding: Some(4.0),
                popup_item_padding_y: Some(4.0),
                popup_item_radius_inset: Some(2.0),
                padding_x_inset_scale: Some(1.0 / 2.0),
                padding_y_gap_scale: Some(1.0 / 2.0),
                button_extra_width_scale: Some(1.0),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: crate::theme::ControlSizes::from([(
                "minimal".into(),
                PagerGeometryStylesheet { button_extra_width_scale: Some(0.0), ..Default::default() },
            )]),
        }
    }
}
