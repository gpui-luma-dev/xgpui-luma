//! Shadcn CSS token usage metadata for gallery sidebars and theme docs.

use xgpui_luma::theme::{ThemePartUsage, ThemeUsage};

pub fn all_shadcn_theme_usages() -> &'static [&'static ThemeUsage] {
    RADIX_THEME_USAGES
}

const TEXTFIELD_PARTS: &[ThemePartUsage] = &[
    part("surface fill", "background", &["surface light"], &["TextFieldPalette.background"]),
    part("surface fill", "input", &["surface dark"], &["TextFieldPalette.background"]),
    part("surface border", "input", &["surface default"], &["TextFieldPalette.border"]),
    part("filled fill", "background", &["filled default"], &["TextFieldPalette.background"]),
    part("filled border", "border", &["filled default"], &["TextFieldPalette.border"]),
    part("input fill", "background", &["input light"], &["TextFieldPalette.background"]),
    part("input fill", "input", &["input dark"], &["TextFieldPalette.background"]),
    part("input border", "border", &["input default"], &["TextFieldPalette.border"]),
    part(
        "foreground",
        "foreground",
        &["surface default", "soft default", "filled default"],
        &["TextFieldPalette.foreground"],
    ),
    part(
        "placeholder",
        "muted-foreground",
        &["surface default", "soft default"],
        &["TextFieldPalette.placeholder"],
    ),
    part(
        "selection",
        "primary",
        &["surface focused", "soft focused"],
        &["TextFieldPalette.selection_background"],
    ),
    part(
        "selection text",
        "primary-foreground",
        &["surface focused", "soft focused"],
        &["TextFieldPalette.selection_foreground"],
    ),
    part("primary elevation", "shadow-xs", &["primary default"], &["TextFieldPalette.shadow"]),
];

const BUTTON_PARTS: &[ThemePartUsage] = &[
    part("primary background", "primary", &["prominent default"], &["ButtonFamilyLook.background"]),
    part("primary foreground", "primary-foreground", &["prominent default"], &["ButtonFamilyLook.foreground"]),
    part("secondary background", "secondary", &["standard default"], &["ButtonFamilyLook.background"]),
    part(
        "secondary foreground",
        "secondary-foreground",
        &["standard default"],
        &["ButtonFamilyLook.foreground"],
    ),
    part("outline border", "border", &["subtle default"], &["ButtonFamilyLook.border"]),
    part("outline elevation", "shadow-xs", &["outline default"], &["ButtonFamilyLook.shadow"]),
    part("ghost hover", "accent", &["ghost hovered"], &["ButtonFamilyLook.background"]),
    part("disabled", "muted", &["disabled"], &["ButtonFamilyLook.background"]),
];

const RADIX_THEME_USAGES: &[&ThemeUsage] = &[
    &ThemeUsage { label: "Button", parts: BUTTON_PARTS },
    &ThemeUsage { label: "Icon Button", parts: BUTTON_PARTS },
    &ThemeUsage { label: "Toggle", parts: BUTTON_PARTS },
    &ThemeUsage {
        label: "Checkbox",
        parts: &[
            part("indicator elevation", "shadow-sm", &["default", "checked"], &["CheckboxPalette.indicator_shadow"]),
            part("checked fill", "primary", &["checked"], &["CheckboxPalette.indicator_background"]),
            part("checkmark", "primary-foreground", &["checked"], &["CheckboxPalette.checkmark_color"]),
            part("border", "input", &["unchecked"], &["CheckboxPalette.indicator_border"]),
        ],
    },
    &ThemeUsage {
        label: "Radio Button",
        parts: &[
            part(
                "indicator elevation",
                "shadow-sm",
                &["default", "selected"],
                &["RadioButtonPalette.indicator_shadow"],
            ),
            part("selected fill", "primary", &["selected"], &["RadioButtonPalette.indicator_border"]),
            part("dot", "primary-foreground", &["selected"], &["RadioButtonPalette.dot_color"]),
            part("border", "input", &["unselected"], &["RadioButtonPalette.indicator_border"]),
        ],
    },
    &ThemeUsage {
        label: "Switch",
        parts: &[
            part("thumb elevation", "shadow-sm", &["default", "on"], &["SwitchPalette.thumb_shadow"]),
            part("off track", "input", &["off"], &["SwitchPalette.track_background"]),
            part("track border", "border", &["off", "disabled"], &["SwitchPalette.track_border"]),
            part("on track", "primary", &["on"], &["SwitchPalette.track_background", "SwitchPalette.track_border"]),
            part("off thumb", "background", &["off"], &["SwitchPalette.thumb_background"]),
            part("thumb border", "border", &["off"], &["SwitchPalette.thumb_border"]),
            part(
                "on thumb",
                "primary-foreground",
                &["on"],
                &["SwitchPalette.thumb_background", "SwitchPalette.thumb_border"],
            ),
            part("disabled thumb", "muted-foreground", &["disabled"], &["SwitchPalette.thumb_background"]),
            part("disabled thumb border", "muted", &["disabled"], &["SwitchPalette.thumb_border"]),
        ],
    },
    &ThemeUsage {
        label: "Slider",
        parts: &[
            part("thumb elevation", "shadow-sm", &["default"], &["SliderLook.thumb_shadow"]),
            part("track", "border", &["default"], &["SliderLook.track_background"]),
            part("fill", "primary", &["default"], &["SliderLook.fill_background"]),
            part("thumb border", "primary", &["default"], &["SliderLook.thumb_border"]),
        ],
    },
    &ThemeUsage {
        label: "Scrollbar",
        parts: &[
            part("track", "background", &["default"], &["ScrollbarLook.track_color"]),
            part("thumb", "border", &["default", "hovered"], &["ScrollbarLook.thumb_color"]),
        ],
    },
    &ThemeUsage { label: "TextField", parts: TEXTFIELD_PARTS },
    &ThemeUsage { label: "TextArea", parts: TEXTFIELD_PARTS },
    &ThemeUsage {
        label: "ComboBox",
        parts: &[
            part("textbox", "background", &["default"], &["TextFieldLook.background"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelLook.background"]),
            part("item hover bg", "accent", &["hovered"], &["SelectorItemsPanelLook.item_hover_background"]),
            part(
                "item hover fg",
                "accent-foreground",
                &["hovered"],
                &["SelectorItemsPanelLook.item_hover_foreground"],
            ),
        ],
    },
    &ThemeUsage {
        label: "AutocompleteTextField",
        parts: &[
            part("textbox fill", "background", &["default"], &["TextFieldLook.background"]),
            part("status", "primary", &["default"], &["AutocompleteLook.status_color"]),
            part("muted text", "muted-foreground", &["default"], &["AutocompleteLook.muted_text_color"]),
            part("panel surface", "popover", &["open"], &["FloatingMenuLook.background"]),
        ],
    },
    &ThemeUsage {
        label: "SearchSelector",
        parts: &[
            part("trigger fill", "background", &["default"], &["TextFieldLook.background"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelLook.background"]),
        ],
    },
    &ThemeUsage {
        label: "Floating Menu",
        parts: &[
            part("surface elevation", "shadow-md", &["default"], &["FloatingMenuLook.shadow"]),
            part("surface", "popover", &["default"], &["FloatingMenuLook.background"]),
            part("foreground", "popover-foreground", &["default"], &["FloatingMenuLook.foreground"]),
            part("item hover bg", "accent", &["hovered"], &["FloatingMenuLook.item_hover_background"]),
            part("item hover fg", "accent-foreground", &["hovered"], &["FloatingMenuLook.item_hover_foreground"]),
            part("border", "border", &["default"], &["FloatingMenuLook.border"]),
        ],
    },
    &ThemeUsage {
        label: "Popup Menu",
        parts: &[
            part("menu elevation", "shadow-md", &["open"], &["FloatingMenuLook.shadow"]),
            part("trigger hover fg", "accent-foreground", &["hovered"], &["PopupMenuPalette.trigger_foreground"]),
            part("item hover bg", "accent", &["hovered"], &["FloatingMenuLook.item_hover_background"]),
            part("menu surface", "popover", &["open"], &["FloatingMenuLook.background"]),
        ],
    },
    &ThemeUsage {
        label: "Selector",
        parts: &[
            part("trigger background", "background", &["outline default"], &["SelectorPalette.trigger_background"]),
            part("trigger hover fill", "accent", &["outline hovered"], &["SelectorPalette.trigger_background"]),
            part("trigger foreground", "foreground", &["outline default"], &["SelectorPalette.trigger_foreground"]),
            part("trigger border", "border", &["outline default"], &["SelectorPalette.trigger_border"]),
            part("trigger shadow", "shadow-xs", &["outline elevated"], &["SelectorPalette.trigger_shadow"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelLook.background"]),
            part("item hover bg", "accent", &["hovered"], &["SelectorItemsPanelLook.item_hover_background"]),
        ],
    },
    &ThemeUsage {
        label: "Context Menu",
        parts: &[
            part("target hover fg", "accent-foreground", &["hovered"], &["ContextMenuLook.target_foreground"]),
            part("item hover bg", "accent", &["hovered"], &["FloatingMenuLook.item_hover_background"]),
        ],
    },
    &ThemeUsage {
        label: "Accordion",
        parts: &[
            part("trigger label", "foreground", &["default"], &["AccordionPalette.foreground"]),
            part("trigger hover fg", "accent-foreground", &["hovered"], &["AccordionPalette.foreground"]),
            part("disabled label", "muted-foreground", &["disabled"], &["AccordionPalette.foreground"]),
            part("chevron", "muted-foreground", &["default"], &["AccordionPalette.chevron_color"]),
            part("item border", "border", &["default"], &["AccordionPalette.border_color"]),
            part("content label", "foreground", &["expanded"], &["AccordionContentPalette.foreground"]),
        ],
    },
    &ThemeUsage {
        label: "Tabs Navigation",
        parts: &[
            part("inactive label", "foreground", &["inactive"], &["TabsItemLook.foreground"]),
            part("active label", "foreground", &["active"], &["TabsItemLook.foreground"]),
        ],
    },
    &ThemeUsage {
        label: "Sidebar",
        parts: &[
            part("surface", "sidebar", &["default"], &["SidebarContainerLook.background"]),
            part("foreground", "sidebar-foreground", &["default"], &["SidebarItemLook.foreground"]),
            part("active", "sidebar-primary", &["selected"], &["SidebarItemLook.foreground"]),
            part("hover bg", "accent", &["hovered"], &["SidebarItemLook.background"]),
            part("hover fg", "accent-foreground", &["hovered"], &["SidebarItemLook.foreground"]),
            part("border", "sidebar-border", &["default"], &["SidebarContainerLook.border"]),
        ],
    },
    &ThemeUsage {
        label: "TreeView",
        parts: &[
            part("row label", "sidebar-foreground", &["default"], &["TreeViewPalette.foreground"]),
            part("row hover", "sidebar-accent", &["hovered"], &["TreeViewPalette.background"]),
            part("row pressed", "accent", &["pressed"], &["TreeViewPalette.background"]),
            part("disabled label", "muted-foreground", &["disabled"], &["TreeViewPalette.foreground"]),
            part("icon", "sidebar-foreground", &["default"], &["TreeViewPalette.icon_color"]),
            part("chevron", "sidebar-foreground", &["default"], &["TreeViewPalette.chevron_color"]),
        ],
    },
    &ThemeUsage {
        label: "Control Group",
        parts: &[
            part("list background", "muted", &["enabled"], &["ControlGroupListLook.background"]),
            part("list border", "border", &["enabled"], &["ControlGroupListLook.border"]),
        ],
    },
    &ThemeUsage {
        label: "ListBox",
        parts: &[
            part("list background", "background", &["enabled"], &["ListBoxSurfacePalette.background"]),
            part("list border", "input", &["enabled"], &["ListBoxSurfacePalette.border"]),
            part("row hover", "accent", &["hovered"], &["ListBoxRowPalette.background"]),
        ],
    },
    &ThemeUsage {
        label: "Table",
        parts: &[
            part("list background", "background", &["enabled"], &["TableLook.background"]),
            part("list border", "input", &["enabled"], &["TableLook.border"]),
            part("header background", "muted", &["enabled"], &["TableLook.header_background"]),
            part("header label", "muted-foreground", &["enabled"], &["TableLook.header_label_color"]),
            part("row hover", "accent", &["hovered"], &["TableRowPalette.background"]),
            part("row active", "muted", &["focused"], &["TableRowPalette.background"]),
            part("row divider", "border", &["enabled"], &["TableRowPalette.divider"]),
            part("selected row", "muted", &["selected"], &["TableRowPalette.background"]),
            part("selected label", "foreground", &["selected"], &["TableRowPalette.label_color"]),
        ],
    },
    &ThemeUsage {
        label: "ResizablePanels",
        parts: &[
            part("panel border", "border", &["enabled"], &["ResizablePanelsLook.border"]),
            part("divider line", "border", &["enabled"], &["ResizablePanelsLook.divider"]),
            part("handle grip", "border", &["enabled"], &["ResizablePanelsLook.grip"]),
            part("grip emphasis", "accent", &["enabled"], &["ResizablePanelsLook.grip_emphasis"]),
            part("disabled divider", "muted-foreground", &["disabled"], &["ResizablePanelsLook.divider"]),
        ],
    },
    &ThemeUsage {
        label: "SplitView",
        parts: &[
            part("separator", "border", &["default"], &["SplitViewLook.separator"]),
            part("separator hover", "border", &["hovered"], &["SplitViewLook.separator_hover"]),
            part(
                "disabled separator",
                "muted-foreground",
                &["disabled"],
                &["SplitViewLook.separator", "SplitViewLook.separator_hover"],
            ),
        ],
    },
    &ThemeUsage {
        label: "Progress",
        parts: &[
            part("track", "accent", &["enabled"], &["ProgressLook.track_color"]),
            part("fill", "accent-foreground", &["enabled"], &["ProgressLook.progress_color"]),
        ],
    },
    &ThemeUsage {
        label: "Selection Panel",
        parts: &[
            part("surface", "popover", &["default"], &["SelectionPanelLook.background"]),
            part("border", "border", &["default"], &["SelectionPanelLook.border"]),
            part("item hover", "accent", &["hovered"], &["SelectionPanelLook.item_hover_background"]),
        ],
    },
    &ThemeUsage { label: "Card", parts: &[part("surface elevation", "shadow", &["default"], &["CardLook.shadow"])] },
];

const fn part(
    part: &'static str,
    token: &'static str,
    states: &'static [&'static str],
    look_fields: &'static [&'static str],
) -> ThemePartUsage {
    ThemePartUsage { part, token, states, look_fields }
}

#[cfg(test)]
mod tests {
    use super::all_shadcn_theme_usages;

    #[test]
    fn shadcn_usage_tokens_resolve_in_css_theme() {
        let theme = crate::test_support::built_in_look("retro-arcade");
        for usage in all_shadcn_theme_usages() {
            for part in usage.parts {
                let resolves = theme.token_color(part.token).is_ok() || theme.parse_shadow_token(part.token).is_ok();
                assert!(resolves, "{} part `{}` missing token `--{}`", usage.label, part.part, part.token);
            }
        }
    }
}
