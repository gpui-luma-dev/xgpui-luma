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
    /// A disabled row's opacity: the `item` part's `:disabled { opacity }`.
    pub item_disabled_opacity: f32,
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
