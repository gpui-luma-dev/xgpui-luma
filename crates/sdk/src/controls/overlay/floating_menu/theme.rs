use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::LumaTextStyle;

#[derive(Clone, Debug)]
pub struct FloatingMenuLook {
    /// Shared popup edge margin in logical pixels.
    pub window_margin: f32,
    /// Positive completion/status adornments.
    pub positive: Hsla,
    /// Popup border thickness in pixels.
    pub border_width: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
    pub item_hover_foreground: Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
    pub disabled_opacity: f32,
    pub submenu_offset_x: f32,
    /// Divider line color.
    pub separator_color: Hsla,
    /// Divider line thickness in pixels.
    pub separator_thickness: f32,
    /// Vertical padding on each side of the divider, in pixels.
    pub separator_spacing: f32,
    /// Horizontal inset on each side of the divider, in pixels.
    pub separator_inset: f32,
}

impl FloatingMenuLook {
    /// Total row height, including dividers, excluding the panel padding.
    pub fn rows_height(&self, items: &[crate::infra::menu_item::MenuItem]) -> f32 {
        items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    self.separator_thickness + 2.0 * self.separator_spacing
                } else {
                    self.item_height
                }
            })
            .sum()
    }
}

pub trait FloatingMenuTheme: Send + Sync {
    fn resolve(&self) -> FloatingMenuLook;
}

pub fn default_floating_menu_theme() -> Arc<dyn FloatingMenuTheme> {
    static THEME: OnceLock<Arc<dyn FloatingMenuTheme>> = OnceLock::new();

    THEME.get_or_init(super::styled::sdk_theme).clone()
}

crate::theme::stylesheet::geometry::geometry_family!(
    FloatingMenuStylesheet,
    FloatingMenuGeometryStylesheet,
    FloatingMenuGeometry,
    ResolvedFloatingMenuGeometry,
    "floating_menu",
    [
        border_width,
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        submenu_offset_x,
        padding_scale,
        item_height_scale,
        item_padding_scale,
        submenu_offset_scale,
        separator_thickness,
        separator_spacing,
        separator_inset,
        radius,
        item_radius
    ],
    unit[disabled_opacity]
);

impl FloatingMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: FloatingMenuGeometryStylesheet {
                min_width: Some(180.0),
                radius: Some(8.0),
                item_radius: Some(3.0),
                padding_scale: Some(0.5),
                item_height_scale: Some(0.9),
                item_padding_scale: Some(0.75),
                submenu_offset_scale: Some(0.5),
                disabled_opacity: Some(0.56),
                separator_thickness: Some(1.0),
                separator_inset: Some(0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
