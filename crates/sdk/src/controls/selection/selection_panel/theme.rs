use gpui::{BoxShadow, Hsla};

use crate::theme::{LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectionPanelLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    /// Panel border thickness in logical pixels; zero removes it.
    pub border_width: f32,
    /// Additional opacity applied to disabled rows.
    pub disabled_opacity: f32,
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
pub fn default_selection_panel_look(tokens: &ThemeTokens) -> SelectionPanelLook {
    styled_selection_panel_look(
        &crate::style::StyleBinding::sdk("selection_panel"),
        &tokens.metrics,
        &tokens.stylesheet,
    )
}

/// A selection panel from a `.lss` `selection_panel` binding (the floating menu's surface and rows unless a
/// look restyles it), through the selector items panel.
pub fn styled_selection_panel_look(
    binding: &crate::style::StyleBinding,
    metrics: &crate::theme::MetricTokens,
    stylesheet: &crate::theme::stylesheet::CommonStylesheet,
) -> SelectionPanelLook {
    let panel = crate::controls::selector_list::styled_selector_items_panel_look(binding, metrics, stylesheet);
    SelectionPanelLook::from_items_panel(panel)
}

impl SelectionPanelLook {
    /// The panel's surface and rows from a selector items panel.
    pub fn from_items_panel(panel: crate::controls::selector_list::SelectorItemsPanelLook) -> Self {
        Self {
            background: panel.background,
            foreground: panel.foreground,
            border: panel.border,
            border_width: panel.border_width,
            disabled_opacity: panel.disabled_opacity,
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
