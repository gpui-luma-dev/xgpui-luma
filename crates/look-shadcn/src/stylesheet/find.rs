use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::controls::ShadcnButtonStyle;
use crate::elements::BadgeVariant;
use crate::tokens::{ShadcnTextRole, ShadcnTextSize};

use super::config::{
    AccordionContentColorRule, AccordionTriggerColorRule, AutocompleteChromeColorRule, BadgeColorRule, ButtonColorRule,
    CardColorRule, CheckboxColorRule, ControlGroupListColorRule, FloatingMenuSurfaceColorRule,
    FloatingMenuSurfaceElevationRule, FloatingMenuTriggerColorRule, ListboxListColorRule, ListboxRowColorRule,
    ProgressColorRule, RadioColorRule, ResizablePanelsColorRule, ScrollbarColorRule, SidebarContainerColorRule,
    SidebarBranchColorRule, SidebarItemColorRule, SidebarSectionColorRule, SliderColorRule, SplitViewColorRule,
    StylesheetConfig, SwitchColorRule, TableRowColorRule, TableSurfaceColorRule, TabsItemColorRule, TabsListColorRule,
    TextfieldColorRule, TreeViewRowColorRule, TypographyRule,
};
use super::selector::{AsSelectorState, ButtonSelectorState, badge_variant_key, theme_mode_key};

pub fn find_button_color_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> Option<&ButtonColorRule> {
    let state = ButtonSelectorState { style, layer, mode: theme_mode, selected };
    let map = state.to_selector_map();
    stylesheet.button.find_color_rule(
        map.get("style").expect("style key"),
        map.get("layer").expect("layer key"),
        map.get("mode").expect("mode key"),
        selected,
    )
}

pub fn find_button_elevation_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<&crate::stylesheet::config::ButtonElevationRule> {
    stylesheet.button.elevation_rule_for_style(crate::stylesheet::selector::button_style_key(style))
}

pub fn find_textfield_elevation_rule(
    stylesheet: &StylesheetConfig,
    style: crate::controls::ShadcnTextFieldStyle,
) -> Option<&crate::stylesheet::config::TextfieldElevationRule> {
    stylesheet.textfield.elevation_rule_for_style(textfield_style_key(style))
}

fn textfield_style_key(style: crate::controls::ShadcnTextFieldStyle) -> &'static str {
    match style {
        crate::controls::ShadcnTextFieldStyle::Outline => "outline",
        crate::controls::ShadcnTextFieldStyle::Input => "input",
        crate::controls::ShadcnTextFieldStyle::Primary => "primary",
        crate::controls::ShadcnTextFieldStyle::Surface => "surface",
    }
}

pub fn find_checkbox_color_rule(
    stylesheet: &StylesheetConfig,
    checked: bool,
    layer: InteractionLayer,
) -> Option<&CheckboxColorRule> {
    stylesheet.checkbox.find_color_rule(checked, layer)
}

pub fn find_radio_color_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    selected: bool,
    layer: InteractionLayer,
) -> Option<&RadioColorRule> {
    stylesheet.radio.find_color_rule(style, selected, layer)
}

pub fn find_switch_color_rule(stylesheet: &StylesheetConfig, on: bool, disabled: bool) -> Option<&SwitchColorRule> {
    stylesheet.switch.find_color_rule(on, disabled)
}

pub fn find_slider_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    layer: InteractionLayer,
) -> Option<&'a SliderColorRule> {
    stylesheet.slider.find_color_rule(style, layer)
}

pub fn find_scrollbar_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&'a ScrollbarColorRule> {
    stylesheet.scrollbar.find_color_rule(style, disabled, layer)
}

pub fn find_accordion_trigger_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&AccordionTriggerColorRule> {
    stylesheet.accordion.trigger.find_color_rule(disabled, layer)
}

pub fn find_accordion_content_color_rule(
    stylesheet: &StylesheetConfig,
    expanded: bool,
) -> Option<&AccordionContentColorRule> {
    stylesheet.accordion.content.find_color_rule(expanded)
}

pub fn find_resizable_panels_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&ResizablePanelsColorRule> {
    stylesheet.resizable_panels.find_color_rule(disabled, layer)
}

pub fn find_listbox_list_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&ListboxListColorRule> {
    stylesheet.listbox.list.find_color_rule(enabled)
}

pub fn find_listbox_row_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> Option<&ListboxRowColorRule> {
    stylesheet.listbox.row.find_color_rule(disabled, focused, layer)
}

pub fn find_table_surface_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&TableSurfaceColorRule> {
    stylesheet.table.surface.find_color_rule(enabled)
}

pub fn find_table_row_color_rule(
    stylesheet: &StylesheetConfig,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&TableRowColorRule> {
    stylesheet.table.row.find_color_rule(selected, focused, disabled, layer)
}

pub fn find_floating_menu_surface_color_rule(stylesheet: &StylesheetConfig) -> Option<&FloatingMenuSurfaceColorRule> {
    stylesheet.floating_menu.surface.color_rule()
}

pub fn find_floating_menu_surface_elevation_rule(
    stylesheet: &StylesheetConfig,
) -> Option<&FloatingMenuSurfaceElevationRule> {
    stylesheet.floating_menu.surface.elevation_rule()
}

pub fn find_floating_menu_trigger_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&FloatingMenuTriggerColorRule> {
    stylesheet.floating_menu.trigger.find_color_rule(disabled, layer)
}

pub fn find_tabs_list_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&TabsListColorRule> {
    stylesheet.tabs.list.find_color_rule(enabled)
}

pub fn find_tabs_item_color_rule(
    stylesheet: &StylesheetConfig,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> Option<&TabsItemColorRule> {
    stylesheet.tabs.item.find_color_rule(active, layer, focused)
}

pub fn find_tree_view_row_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&TreeViewRowColorRule> {
    stylesheet.tree_view.row.find_color_rule(disabled, layer)
}

pub fn find_sidebar_metrics(stylesheet: &StylesheetConfig) -> &crate::stylesheet::config::SidebarMetricsRule {
    &stylesheet.sidebar.metrics
}

pub fn find_sidebar_container_color_rule(stylesheet: &StylesheetConfig) -> Option<&SidebarContainerColorRule> {
    stylesheet.sidebar.container.color_rule()
}

pub fn find_sidebar_section_color_rule(stylesheet: &StylesheetConfig) -> Option<&SidebarSectionColorRule> {
    stylesheet.sidebar.section.color_rule()
}

pub fn find_sidebar_branch_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&SidebarBranchColorRule> {
    stylesheet.sidebar.branch.find_color_rule(disabled, layer)
}

pub fn find_sidebar_item_color_rule(
    stylesheet: &StylesheetConfig,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&SidebarItemColorRule> {
    stylesheet.sidebar.item.find_color_rule(selected, disabled, layer)
}

pub fn find_textfield_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> Option<&'a TextfieldColorRule> {
    stylesheet.textfield.find_color_rule(style, enabled, invalid, theme_mode_key(theme_mode))
}

pub fn find_typography_scale_rule(stylesheet: &StylesheetConfig, size: ShadcnTextSize) -> Option<&TypographyRule> {
    stylesheet.typography.scale_rule(size.as_str())
}

pub fn find_typography_semantic_rule(stylesheet: &StylesheetConfig, role: ShadcnTextRole) -> Option<&TypographyRule> {
    stylesheet.typography.semantic_rule(role.as_str())
}

pub fn find_autocomplete_chrome_color_rule(stylesheet: &StylesheetConfig) -> Option<&AutocompleteChromeColorRule> {
    stylesheet.autocomplete.chrome.color_rule()
}

pub fn find_progress_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&ProgressColorRule> {
    stylesheet.progress.find_color_rule(enabled)
}

pub fn find_card_color_rule(stylesheet: &StylesheetConfig) -> Option<&CardColorRule> {
    stylesheet.card.color_rule()
}

pub fn find_card_elevation_rule(stylesheet: &StylesheetConfig) -> Option<&FloatingMenuSurfaceElevationRule> {
    stylesheet.card.elevation_rule()
}

pub fn find_badge_color_rule(
    stylesheet: &StylesheetConfig,
    variant: BadgeVariant,
    theme_mode: ThemeMode,
) -> Option<&BadgeColorRule> {
    stylesheet.badge.find_color_rule(badge_variant_key(variant), theme_mode_key(theme_mode))
}

pub fn find_split_view_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&SplitViewColorRule> {
    stylesheet.split_view.find_color_rule(enabled)
}

pub fn find_control_group_list_color_rule(
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> Option<&ControlGroupListColorRule> {
    stylesheet.control_group.list.find_color_rule(enabled)
}
