use xgpui_luma::theme::InteractionLayer;

use crate::controls::ShadcnButtonStyle;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::config::{
    AccordionContentColorRule, AccordionTriggerColorRule, AutocompleteChromeColorRule, BadgeColorRule, ButtonColorRule,
    CardColorRule, CheckboxColorRule, ControlGroupListColorRule, FloatingMenuSurfaceColorRule,
    FloatingMenuTriggerColorRule, ListboxListColorRule, ListboxRowColorRule, ProgressColorRule, RadioColorRule,
    ResizablePanelsColorRule, ScrollbarColorRule, SidebarBranchColorRule, SidebarContainerColorRule,
    SidebarItemColorRule, SidebarSectionColorRule, SliderColorRule, SplitViewColorRule, SwitchColorRule,
    TableRowColorRule, TableSurfaceColorRule, TabsItemColorRule, TabsListColorRule, TextfieldColorRule,
    TreeViewRowColorRule,
};

use super::{
    ResolveContext, ResolvedFields, resolve_color_ref, resolve_optional_stylesheet_color, resolve_stylesheet_color,
};

pub fn resolve_button_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ButtonColorRule,
) -> anyhow::Result<ResolvedButtonColors> {
    let mut fields = ResolvedFields::default();

    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());

    let foreground = resolve_color_ref(resolver, &rule.foreground, &fields)?;
    fields.insert("foreground", foreground.clone());

    let border = match &rule.border {
        Some(value) => Some(resolve_color_ref(resolver, value, &fields)?),
        None => None,
    };

    Ok(ResolvedButtonColors { background, foreground, border })
}

pub fn resolve_progress_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ProgressColorRule,
) -> anyhow::Result<ResolvedProgressColors> {
    Ok(ResolvedProgressColors {
        track_color: resolve_color_ref(resolver, &rule.track_color, &ResolvedFields::default())?,
        progress_color: resolve_color_ref(resolver, &rule.progress_color, &ResolvedFields::default())?,
    })
}

pub fn resolve_card_color_rule(
    resolver: &LookResolver<'_>,
    rule: &CardColorRule,
) -> anyhow::Result<ResolvedCardColors> {
    let mut fields = ResolvedFields::default();
    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());
    let foreground = resolve_color_ref(resolver, &rule.foreground, &fields)?;
    fields.insert("foreground", foreground.clone());
    let muted_foreground = resolve_color_ref(resolver, &rule.muted_foreground, &fields)?;
    let border = resolve_color_ref(resolver, &rule.border, &fields)?;
    Ok(ResolvedCardColors { background, foreground, muted_foreground, border })
}

pub fn resolve_badge_color_rule(
    resolver: &LookResolver<'_>,
    rule: &BadgeColorRule,
) -> anyhow::Result<ResolvedBadgeColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_optional_stylesheet_color(resolver, rule.border.as_deref(), &fields, &ctx)?;
    Ok(ResolvedBadgeColors { background, foreground, border })
}

pub fn resolve_split_view_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SplitViewColorRule,
) -> anyhow::Result<ResolvedSplitViewColors> {
    Ok(ResolvedSplitViewColors {
        separator: resolve_color_ref(resolver, &rule.separator, &ResolvedFields::default())?,
        separator_hover: resolve_color_ref(resolver, &rule.separator_hover, &ResolvedFields::default())?,
    })
}

pub fn resolve_control_group_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ControlGroupListColorRule,
) -> anyhow::Result<ResolvedControlGroupListColors> {
    let mut fields = ResolvedFields::default();
    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());
    let border = resolve_color_ref(resolver, &rule.border, &fields)?;
    Ok(ResolvedControlGroupListColors { background, border })
}

pub fn resolve_checkbox_color_rule(
    resolver: &LookResolver<'_>,
    rule: &CheckboxColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedCheckboxColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let indicator_background = resolve_stylesheet_color(resolver, &rule.indicator_background, &fields, &ctx)?;
    fields.insert("indicator_background", indicator_background.clone());
    let checkmark_color = resolve_stylesheet_color(resolver, &rule.checkmark_color, &fields, &ctx)?;
    fields.insert("checkmark_color", checkmark_color.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedCheckboxColors { indicator_background, checkmark_color, label_color })
}

pub fn resolve_radio_color_rule(
    resolver: &LookResolver<'_>,
    rule: &RadioColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedRadioColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let indicator_background = resolve_stylesheet_color(resolver, &rule.indicator_background, &fields, &ctx)?;
    fields.insert("indicator_background", indicator_background.clone());
    let selection_ring = resolve_stylesheet_color(resolver, &rule.selection_ring, &fields, &ctx)?;
    fields.insert("selection_ring", selection_ring.clone());
    let dot_color = resolve_stylesheet_color(resolver, &rule.dot_color, &fields, &ctx)?;
    fields.insert("dot_color", dot_color.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedRadioColors { indicator_background, selection_ring, dot_color, label_color })
}

pub fn resolve_switch_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SwitchColorRule,
    style: ShadcnButtonStyle,
) -> anyhow::Result<ResolvedSwitchColors> {
    let ctx = ResolveContext { style, layer: InteractionLayer::Default, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    fields.insert("thumb_background", thumb_background.clone());
    let thumb_border = resolve_stylesheet_color(resolver, &rule.thumb_border, &fields, &ctx)?;
    fields.insert("thumb_border", thumb_border.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedSwitchColors { track_background, thumb_background, thumb_border, label_color })
}

pub fn resolve_slider_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SliderColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSliderColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let fill_background = resolve_stylesheet_color(resolver, &rule.fill_background, &fields, &ctx)?;
    fields.insert("fill_background", fill_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    fields.insert("thumb_background", thumb_background.clone());
    let thumb_border = resolve_stylesheet_color(resolver, &rule.thumb_border, &fields, &ctx)?;
    Ok(ResolvedSliderColors { track_background, fill_background, thumb_background, thumb_border })
}

pub fn resolve_scrollbar_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ScrollbarColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedScrollbarColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    Ok(ResolvedScrollbarColors { track_background, thumb_background })
}

pub fn resolve_accordion_trigger_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AccordionTriggerColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedAccordionTriggerColors> {
    let disabled = rule.disabled.unwrap_or(false);
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let chevron_color = resolve_stylesheet_color(resolver, &rule.chevron_color, &fields, &ctx)?;
    fields.insert("chevron_color", chevron_color.clone());
    let border_color = resolve_stylesheet_color(resolver, &rule.border_color, &fields, &ctx)?;
    fields.insert("border_color", border_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedAccordionTriggerColors { foreground, icon_color, chevron_color, border_color, background })
}

pub fn resolve_accordion_content_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AccordionContentColorRule,
) -> anyhow::Result<ResolvedAccordionContentColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedAccordionContentColors { foreground, background })
}

pub fn resolve_resizable_panels_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ResizablePanelsColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedResizablePanelsColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    fields.insert("divider", divider.clone());
    let grip = resolve_stylesheet_color(resolver, &rule.grip, &fields, &ctx)?;
    fields.insert("grip", grip.clone());
    let grip_emphasis = resolve_stylesheet_color(resolver, &rule.grip_emphasis, &fields, &ctx)?;
    Ok(ResolvedResizablePanelsColors { border, divider, grip, grip_emphasis })
}

pub fn resolve_listbox_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListboxListColorRule,
) -> anyhow::Result<ResolvedListboxListColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    Ok(ResolvedListboxListColors { background, border, divider })
}

pub fn resolve_listbox_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListboxRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedListboxRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    Ok(ResolvedListboxRowColors { label_color, background })
}

pub fn resolve_table_surface_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TableSurfaceColorRule,
) -> anyhow::Result<ResolvedTableSurfaceColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let header_background = resolve_stylesheet_color(resolver, &rule.header_background, &fields, &ctx)?;
    fields.insert("header_background", header_background.clone());
    let header_label_color = resolve_stylesheet_color(resolver, &rule.header_label_color, &fields, &ctx)?;
    Ok(ResolvedTableSurfaceColors { background, border, header_background, header_label_color })
}

pub fn resolve_table_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TableRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedTableRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    Ok(ResolvedTableRowColors { background, label_color, divider })
}

pub fn resolve_floating_menu_surface_color_rule(
    resolver: &LookResolver<'_>,
    rule: &FloatingMenuSurfaceColorRule,
) -> anyhow::Result<ResolvedFloatingMenuColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let item_hover_background = resolve_stylesheet_color(resolver, &rule.item_hover_background, &fields, &ctx)?;
    fields.insert("item_hover_background", item_hover_background.clone());
    let item_hover_foreground = resolve_stylesheet_color(resolver, &rule.item_hover_foreground, &fields, &ctx)?;
    fields.insert("item_hover_foreground", item_hover_foreground.clone());
    let item_disabled_foreground = resolve_stylesheet_color(resolver, &rule.item_disabled_foreground, &fields, &ctx)?;
    Ok(ResolvedFloatingMenuColors {
        background,
        foreground,
        border,
        item_hover_background,
        item_hover_foreground,
        item_disabled_foreground,
    })
}

pub fn resolve_floating_menu_trigger_color_rule(
    resolver: &LookResolver<'_>,
    rule: &FloatingMenuTriggerColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedGhostTriggerColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    Ok(ResolvedGhostTriggerColors { background, foreground })
}

pub fn resolve_tabs_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TabsListColorRule,
) -> anyhow::Result<ResolvedTabsListColors> {
    let disabled_background = resolve_stylesheet_color(
        resolver,
        &rule.disabled_background,
        &ResolvedFields::default(),
        &ResolveContext::default(),
    )?;
    Ok(ResolvedTabsListColors { disabled_background })
}

pub fn resolve_tabs_item_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TabsItemColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedTabsItemColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: false };
    let mut fields = ResolvedFields::default();
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let indicator = resolve_optional_stylesheet_color(resolver, rule.indicator.as_deref(), &fields, &ctx)?;
    Ok(ResolvedTabsItemColors { label_color, indicator })
}

pub fn resolve_tree_view_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TreeViewRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedTreeViewRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let chevron_color = resolve_stylesheet_color(resolver, &rule.chevron_color, &fields, &ctx)?;
    fields.insert("chevron_color", chevron_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedTreeViewRowColors { foreground, icon_color, chevron_color, background })
}

pub fn resolve_sidebar_container_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarContainerColorRule,
) -> anyhow::Result<ResolvedSidebarContainerColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    Ok(ResolvedSidebarContainerColors { background, foreground, border })
}

pub fn resolve_sidebar_section_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarSectionColorRule,
) -> anyhow::Result<ResolvedSidebarSectionColors> {
    let label_color =
        resolve_stylesheet_color(resolver, &rule.label_color, &ResolvedFields::default(), &ResolveContext::default())?;
    Ok(ResolvedSidebarSectionColors { label_color })
}

pub fn resolve_sidebar_branch_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarBranchColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSidebarBranchColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedSidebarBranchColors { foreground, icon_color, background })
}

pub fn resolve_sidebar_item_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarItemColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSidebarItemColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedSidebarItemColors { foreground, icon_color, background })
}

pub fn resolve_textfield_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TextfieldColorRule,
) -> anyhow::Result<ResolvedTextfieldColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let placeholder = resolve_stylesheet_color(resolver, &rule.placeholder, &fields, &ctx)?;
    fields.insert("placeholder", placeholder.clone());
    let icon = resolve_stylesheet_color(resolver, &rule.icon, &fields, &ctx)?;
    fields.insert("icon", icon.clone());
    let selection_background = resolve_stylesheet_color(resolver, &rule.selection_background, &fields, &ctx)?;
    fields.insert("selection_background", selection_background.clone());
    let selection_foreground = resolve_stylesheet_color(resolver, &rule.selection_foreground, &fields, &ctx)?;
    fields.insert("selection_foreground", selection_foreground.clone());
    let caret = resolve_stylesheet_color(resolver, &rule.caret, &fields, &ctx)?;
    Ok(ResolvedTextfieldColors {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        selection_foreground,
        caret,
    })
}

pub fn resolve_autocomplete_chrome_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AutocompleteChromeColorRule,
) -> anyhow::Result<ResolvedAutocompleteChromeColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let status_color = resolve_stylesheet_color(resolver, &rule.status_color, &fields, &ctx)?;
    fields.insert("status_color", status_color.clone());
    let muted_text_color = resolve_stylesheet_color(resolver, &rule.muted_text_color, &fields, &ctx)?;
    fields.insert("muted_text_color", muted_text_color.clone());
    let clear_icon_color = resolve_stylesheet_color(resolver, &rule.clear_icon_color, &fields, &ctx)?;
    fields.insert("clear_icon_color", clear_icon_color.clone());
    let clear_icon_hover_color = resolve_stylesheet_color(resolver, &rule.clear_icon_hover_color, &fields, &ctx)?;
    Ok(ResolvedAutocompleteChromeColors { status_color, muted_text_color, clear_icon_color, clear_icon_hover_color })
}

#[derive(Clone, Debug)]
pub struct ResolvedProgressColors {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedStepperMetrics {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedCardColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub muted_foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedBadgeColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSplitViewColors {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedCheckboxColors {
    pub indicator_background: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedRadioColors {
    pub indicator_background: ResolvedColor,
    pub selection_ring: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSwitchColors {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSliderColors {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedScrollbarColors {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedAccordionTriggerColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub border_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedAccordionContentColors {
    pub foreground: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedResizablePanelsColors {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListboxListColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListboxRowColors {
    pub label_color: ResolvedColor,
    pub background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTableSurfaceColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTableRowColors {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedFloatingMenuColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedGhostTriggerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTabsListColors {
    pub disabled_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTabsItemColors {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedTreeViewRowColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarContainerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarSectionColors {
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarBranchColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarItemColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedTextfieldColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedAutocompleteChromeColors {
    pub status_color: ResolvedColor,
    pub muted_text_color: ResolvedColor,
    pub clear_icon_color: ResolvedColor,
    pub clear_icon_hover_color: ResolvedColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSliderMetrics {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedScrollbarMetrics {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedControlGroupListColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedProgressMetrics {
    pub size: f32,
    pub stroke_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedButtonColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedButtonMetrics {
    pub height: f32,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}
