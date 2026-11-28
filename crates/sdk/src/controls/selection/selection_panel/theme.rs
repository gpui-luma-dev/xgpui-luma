#![allow(deprecated)] // SelectionPanelLook is this module's compatibility type.

use gpui::{BoxShadow, Hsla};

use crate::theme::{LumaTextStyle, ThemeTokens};

/// A selection panel's values, copied out of its `selection_panel` style (#433). A compatibility type: the template
/// draws from the style itself (see [`SelectionPanelRenderModel::menu`](super::SelectionPanelRenderModel::menu)).
#[deprecated(
    note = "the template draws from the `selection_panel` style; resolve it and read its parts instead (#433)"
)]
#[derive(Clone, Debug)]
pub struct SelectionPanelLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    /// Panel border thickness in logical pixels; zero removes it.
    pub border_width: f32,
    /// A disabled row's opacity (the floating menu's `item` part).
    pub item_disabled_opacity: f32,
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
}

/// The SDK's selection panel: the SDK's `.lss` `selection_panel` with `tokens`' metrics and geometry.
#[deprecated(note = "use `StyleBinding::sdk(\"selection_panel\")`; the template draws from the style (#433)")]
pub fn default_selection_panel_look(tokens: &ThemeTokens) -> SelectionPanelLook {
    styled_selection_panel_look(&crate::style::StyleBinding::sdk("selection_panel"), &tokens.metrics)
}

/// A selection panel from a `.lss` `selection_panel` binding (the floating menu's surface and rows unless a
/// look restyles it), through the selector items panel.
#[deprecated(note = "the template draws from the `selection_panel` binding itself (#433)")]
pub fn styled_selection_panel_look(
    binding: &crate::style::StyleBinding,
    metrics: &crate::theme::MetricTokens,
) -> SelectionPanelLook {
    let panel = crate::controls::selector_list::styled_selector_items_panel_look(binding, metrics);
    SelectionPanelLook::from_items_panel(panel)
}

impl SelectionPanelLook {
    /// The panel `menu` (a `selection_panel` binding) describes.
    pub fn from_style(menu: &crate::style::StyleBinding) -> Self {
        Self::from_items_panel(crate::controls::selector_list::SelectorItemsPanelLook::from_style(menu))
    }

    /// The panel's surface and rows from a selector items panel.
    pub fn from_items_panel(panel: crate::controls::selector_list::SelectorItemsPanelLook) -> Self {
        Self {
            background: panel.background,
            foreground: panel.foreground,
            border: panel.border,
            border_width: panel.border_width,
            item_disabled_opacity: panel.item_disabled_opacity,
            shadow: panel.shadow,
            radius: panel.radius,
            padding: panel.padding,
            min_width: panel.min_width,
            item_disabled_foreground: panel.item_disabled_foreground,
            item_hover_background: panel.item_hover_background,
            item_hover_foreground: panel.item_hover_foreground,
            item_typography: panel.item_typography,
            item_height: panel.item_height,
            item_padding_x: panel.item_padding_x,
            item_gap: panel.item_gap,
            item_icon_size: panel.item_icon_size,
            item_radius: panel.item_radius,
        }
    }
}
