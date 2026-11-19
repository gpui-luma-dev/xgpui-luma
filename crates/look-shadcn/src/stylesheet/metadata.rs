use crate::provenance::TableRuleMetadata;

use super::config::{ButtonColorRule, ControlGroupListColorRule, ProgressColorRule, SplitViewColorRule, StylesheetConfig};
use super::embedded_stylesheet;

pub fn resolve_button_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .button
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
                rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: button_rule_output_strings(rule),
        })
        .collect()
}

pub fn resolve_switch_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .switch
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.on.map(|on| on.to_string()).unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("thumb_background", &rule.thumb_background),
                color_output_label("thumb_border", &rule.thumb_border),
                color_output_label("label_color", &rule.label_color),
            ],
        })
        .collect()
}

pub fn resolve_slider_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .slider
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("fill_background", &rule.fill_background),
                color_output_label("thumb_background", &rule.thumb_background),
                color_output_label("thumb_border", &rule.thumb_border),
            ],
        })
        .collect()
}

pub fn resolve_scrollbar_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .scrollbar
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("thumb_background", &rule.thumb_background),
            ],
        })
        .collect()
}

pub fn resolve_accordion_trigger_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .accordion
        .trigger
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
                color_output_label("chevron_color", &rule.chevron_color),
                color_output_label("border_color", &rule.border_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_accordion_content_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .accordion
        .content
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.expanded.map(|expanded| expanded.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("foreground", &rule.foreground),
                optional_color_output_label("background", rule.background.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_resizable_panels_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .resizable_panels
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("border", &rule.border),
                color_output_label("divider", &rule.divider),
                color_output_label("grip", &rule.grip),
                color_output_label("grip_emphasis", &rule.grip_emphasis),
            ],
        })
        .collect()
}

pub fn resolve_listbox_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .listbox
        .list
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("border", &rule.border),
                color_output_label("divider", &rule.divider),
            ],
        })
        .collect()
}

pub fn resolve_listbox_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .listbox
        .row
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("label_color", &rule.label_color),
                color_output_label("background", &rule.background),
            ],
        })
        .collect()
}

pub fn resolve_table_surface_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .table
        .surface
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("border", &rule.border),
                color_output_label("header_background", &rule.header_background),
                color_output_label("header_label_color", &rule.header_label_color),
            ],
        })
        .collect()
}

pub fn resolve_table_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .table
        .row
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("label_color", &rule.label_color),
                color_output_label("divider", &rule.divider),
            ],
        })
        .collect()
}

pub fn resolve_floating_menu_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .floating_menu
        .surface
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
                color_output_label("item_hover_background", &rule.item_hover_background),
                color_output_label("item_hover_foreground", &rule.item_hover_foreground),
                color_output_label("item_disabled_foreground", &rule.item_disabled_foreground),
            ],
        })
        .collect()
}

pub fn resolve_ghost_trigger_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .floating_menu
        .trigger
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
            ],
        })
        .collect()
}

pub fn resolve_tabs_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tabs
        .list
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![color_output_label("disabled_background", &rule.disabled_background)],
        })
        .collect()
}

pub fn resolve_tabs_item_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tabs
        .item
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.active.map(|active| active.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("label_color", &rule.label_color),
                optional_color_output_label("indicator", rule.indicator.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_tree_view_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tree_view
        .row
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
                color_output_label("chevron_color", &rule.chevron_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_sidebar_container_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .container
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
            ],
        })
        .collect()
}

pub fn resolve_sidebar_section_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .section
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![color_output_label("label_color", &rule.label_color)],
        })
        .collect()
}

pub fn resolve_sidebar_branch_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .branch
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_sidebar_item_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .item
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_textfield_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .textfield
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.invalid.map(|invalid| invalid.to_string()).unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
                color_output_label("placeholder", &rule.placeholder),
                color_output_label("icon", &rule.icon),
                color_output_label("selection_background", &rule.selection_background),
                color_output_label("selection_foreground", &rule.selection_foreground),
                color_output_label("caret", &rule.caret),
            ],
        })
        .collect()
}

pub fn resolve_autocomplete_chrome_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .autocomplete
        .chrome
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("status_color", &rule.status_color),
                color_output_label("muted_text_color", &rule.muted_text_color),
                color_output_label("clear_icon_color", &rule.clear_icon_color),
                color_output_label("clear_icon_hover_color", &rule.clear_icon_hover_color),
            ],
        })
        .collect()
}

pub fn resolve_progress_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .progress
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, progress_rule_output_strings(rule)))
        .collect()
}

pub fn resolve_badge_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .badge
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                optional_color_output_label("border", rule.border.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_split_view_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .split_view
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, split_view_rule_output_strings(rule)))
        .collect()
}

pub fn resolve_control_group_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .control_group
        .list
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, control_group_list_rule_output_strings(rule)))
        .collect()
}

fn enabled_color_rule_metadata(enabled: Option<bool>, outputs: Vec<String>) -> TableRuleMetadata {
    TableRuleMetadata { inputs: vec![enabled.map(|value| value.to_string()).unwrap_or_else(|| "any".into())], outputs }
}

fn color_output_label(field: &str, raw: &str) -> String {
    match raw.trim() {
        "transparent" => format!("{field}: transparent"),
        value if value.starts_with('@') || value.starts_with("first(") || value.starts_with("first_layer(") => {
            format!("{field}: {value}")
        }
        value => format!("{field}: --{}", value.trim_start_matches('@')),
    }
}

fn optional_color_output_label(field: &str, raw: Option<&str>) -> String {
    raw.map(|value| color_output_label(field, value)).unwrap_or_else(|| format!("{field}: none"))
}

fn button_rule_output_strings(rule: &ButtonColorRule) -> Vec<String> {
    let mut outputs = vec![
        format!("background: --{}", rule.background.trim_start_matches('@')),
        format!("foreground: --{}", rule.foreground.trim_start_matches('@')),
    ];
    if let Some(border) = &rule.border {
        outputs.push(format!("border: --{}", border.trim_start_matches('@')));
    }
    outputs
}

fn progress_rule_output_strings(rule: &ProgressColorRule) -> Vec<String> {
    vec![
        format!("track_color: --{}", rule.track_color.trim_start_matches('@')),
        format!("progress_color: --{}", rule.progress_color.trim_start_matches('@')),
    ]
}

fn split_view_rule_output_strings(rule: &SplitViewColorRule) -> Vec<String> {
    vec![
        format!("separator: --{}", rule.separator.trim_start_matches('@')),
        format!("separator_hover: --{}", rule.separator_hover.trim_start_matches('@')),
    ]
}

fn control_group_list_rule_output_strings(rule: &ControlGroupListColorRule) -> Vec<String> {
    vec![
        format!("background: --{}", rule.background.trim_start_matches('@')),
        format!("border: --{}", rule.border.trim_start_matches('@')),
    ]
}

/// One inspectable color-rule table from the loaded stylesheet.
#[derive(Clone, Debug)]
pub struct ColorRuleMetadataSection {
    pub control: &'static str,
    pub part: &'static str,
    pub rules: Vec<TableRuleMetadata>,
}

/// All color-rule metadata sections derived from a stylesheet config.
pub fn all_color_rule_metadata(stylesheet: &StylesheetConfig) -> Vec<ColorRuleMetadataSection> {
    vec![
        section(stylesheet, "button", "color", resolve_button_colors_metadata(stylesheet)),
        section(stylesheet, "badge", "color", resolve_badge_colors_metadata(stylesheet)),
        section(stylesheet, "switch", "color", resolve_switch_colors_metadata(stylesheet)),
        section(stylesheet, "slider", "color", resolve_slider_colors_metadata(stylesheet)),
        section(stylesheet, "scrollbar", "color", resolve_scrollbar_colors_metadata(stylesheet)),
        section(stylesheet, "accordion", "trigger", resolve_accordion_trigger_colors_metadata(stylesheet)),
        section(stylesheet, "accordion", "content", resolve_accordion_content_colors_metadata(stylesheet)),
        section(stylesheet, "resizable_panels", "color", resolve_resizable_panels_colors_metadata(stylesheet)),
        section(stylesheet, "listbox", "list", resolve_listbox_list_colors_metadata(stylesheet)),
        section(stylesheet, "listbox", "row", resolve_listbox_row_colors_metadata(stylesheet)),
        section(stylesheet, "table", "surface", resolve_table_surface_colors_metadata(stylesheet)),
        section(stylesheet, "table", "row", resolve_table_row_colors_metadata(stylesheet)),
        section(stylesheet, "floating_menu", "surface", resolve_floating_menu_colors_metadata(stylesheet)),
        section(stylesheet, "floating_menu", "trigger", resolve_ghost_trigger_colors_metadata(stylesheet)),
        section(stylesheet, "tabs", "list", resolve_tabs_list_colors_metadata(stylesheet)),
        section(stylesheet, "tabs", "item", resolve_tabs_item_colors_metadata(stylesheet)),
        section(stylesheet, "tree_view", "row", resolve_tree_view_row_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "container", resolve_sidebar_container_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "section", resolve_sidebar_section_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "branch", resolve_sidebar_branch_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "item", resolve_sidebar_item_colors_metadata(stylesheet)),
        section(stylesheet, "textfield", "color", resolve_textfield_colors_metadata(stylesheet)),
        section(stylesheet, "autocomplete", "chrome", resolve_autocomplete_chrome_colors_metadata(stylesheet)),
        section(stylesheet, "progress", "color", resolve_progress_colors_metadata(stylesheet)),
        section(stylesheet, "split_view", "color", resolve_split_view_colors_metadata(stylesheet)),
        section(stylesheet, "control_group", "list", resolve_control_group_list_colors_metadata(stylesheet)),
    ]
}

pub fn embedded_color_rule_metadata() -> Vec<ColorRuleMetadataSection> {
    all_color_rule_metadata(embedded_stylesheet())
}

fn section(
    _stylesheet: &StylesheetConfig,
    control: &'static str,
    part: &'static str,
    rules: Vec<TableRuleMetadata>,
) -> ColorRuleMetadataSection {
    ColorRuleMetadataSection { control, part, rules }
}
