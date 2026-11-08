//! Captured motion roles for the control Theme Inspector.
//!
//! Look defaults and the example's spawn snapshot are read separately. A later
//! look edit changes only the default column.

use std::sync::Arc;
use std::time::Duration;

use gpui::{App, Entity, SharedString};
use xgpui_luma::controls::accordion::{AccordionControl, AccordionMotion};
use xgpui_luma::controls::autocomplete::AutocompleteControl;
use xgpui_luma::controls::combobox::ComboBoxControl;
use xgpui_luma::controls::context_menu::{ContextMenu, ContextMenuMotion};
use xgpui_luma::controls::control_group::{ControlGroupControl, ControlGroupItemLike, ControlGroupMotion};
use xgpui_luma::controls::listbox::ListBoxMotion;
use xgpui_luma::controls::popup_menu::{PopupMenu, PopupMenuMotion};
use xgpui_luma::controls::progress::{Progress, ProgressMotion};
use xgpui_luma::controls::resizable_panels::{ResizablePanels, ResizablePanelsMotion};
use xgpui_luma::controls::search_selector::{SearchSelectorControl, SearchSelectorMotion};
use xgpui_luma::controls::selector::Selector;
use xgpui_luma::controls::sidebar::{SidebarControl, SidebarMotion};
use xgpui_luma::controls::slide_panel::SlidePanelMotion;
use xgpui_luma::controls::slider::{SliderControl, SliderMotion};
use xgpui_luma::controls::stepper::{Stepper, StepperMotion};
use xgpui_luma::controls::switch::{Switch, SwitchMotion};
use xgpui_luma::controls::table::{TableControl, TableMotion};
use xgpui_luma::controls::tabs::{Tabs, TabsMotion};
use xgpui_luma::controls::tree_view::{TreeView, TreeViewMotion};
use xgpui_luma::motion::{Easing, PresenceMotion, Transition, TransitionDescription};
use xgpui_luma::theme::{ControlSize, LumaTextStyle};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_shadcn::tables::typography::resolve_control_typography;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook};

use super::components::InspectorSection;
use crate::studio::controls::control_exposition::template::controls_mono_font;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionRow {
    pub role: String,
    pub example: String,
    pub look_default: String,
    pub inherited: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionExample {
    pub label: SharedString,
    pub enabled: bool,
    pub rows: Vec<MotionRow>,
}

pub trait MotionReport: Send + Sync {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample>;
}

pub type SharedMotionReport = Arc<dyn MotionReport>;

pub struct TabsMotionReport {
    samples: Vec<(SharedString, Entity<Tabs>)>,
}

impl TabsMotionReport {
    pub fn new(examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<Tabs>)>) -> Self {
        Self { samples: examples.into_iter().map(|(label, tabs)| (label.into(), tabs)).collect() }
    }
}

impl MotionReport for TabsMotionReport {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample> {
        let defaults = look.motion_defaults().tabs;
        self.samples
            .iter()
            .map(|(label, tabs)| {
                let tabs = tabs.read(cx);
                motion_example(label.clone(), tabs.motion_enabled(), tabs_rows(tabs.motion_profile(), &defaults))
            })
            .collect()
    }
}

pub fn format_transition(transition: &Transition) -> String {
    match transition.description() {
        TransitionDescription::Timed { duration, easing } => match easing {
            Easing::Linear => format!("{} ms · linear", duration.as_millis()),
            Easing::OutCubic => format!("{} ms · ease-out", duration.as_millis()),
        },
        TransitionDescription::Spring { stiffness, damping } => format!("spring · {stiffness} / {damping}"),
        TransitionDescription::Instant => "instant".into(),
        TransitionDescription::Custom => "custom".into(),
    }
}

/// Loop period for roles such as progress.
pub fn format_period(period: Duration) -> String {
    format!("period · {} ms", period.as_millis())
}

pub fn policy_label(enabled: bool) -> &'static str {
    if enabled {
        "Motion enabled"
    } else {
        "Motion disabled · authored profile kept"
    }
}

pub fn transition_row(role: &str, example: &Transition, defaults: &Transition) -> MotionRow {
    MotionRow {
        role: role.into(),
        example: format_transition(example),
        look_default: format_transition(defaults),
        inherited: example.description() == defaults.description(),
    }
}

pub struct SwitchMotionReport {
    samples: Vec<(SharedString, Switch)>,
}

impl SwitchMotionReport {
    pub fn new(examples: impl IntoIterator<Item = (impl Into<SharedString>, Switch)>) -> Self {
        Self { samples: examples.into_iter().map(|(label, switch)| (label.into(), switch)).collect() }
    }
}

impl MotionReport for SwitchMotionReport {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample> {
        let defaults = look.motion_defaults().switch;
        self.samples
            .iter()
            .map(|(label, switch)| {
                let switch = switch.read(cx);
                motion_example(label.clone(), switch.motion_enabled(), switch_rows(switch.motion_profile(), &defaults))
            })
            .collect()
    }
}

/// One captured transition role, read from each displayed example.
pub struct StateMotionReport<T: 'static> {
    role: &'static str,
    samples: Vec<(SharedString, Entity<T>)>,
    enabled: fn(&T) -> bool,
    state: fn(&T) -> Transition,
    defaults: fn(&ShadcnLook) -> Transition,
}

impl<T: 'static> StateMotionReport<T> {
    pub fn new(
        role: &'static str,
        examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<T>)>,
        enabled: fn(&T) -> bool,
        state: fn(&T) -> Transition,
        defaults: fn(&ShadcnLook) -> Transition,
    ) -> Self {
        Self {
            role,
            samples: examples.into_iter().map(|(label, entity)| (label.into(), entity)).collect(),
            enabled,
            state,
            defaults,
        }
    }
}

impl<T: 'static> MotionReport for StateMotionReport<T> {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample> {
        let defaults = (self.defaults)(look);
        self.samples
            .iter()
            .map(|(label, entity)| {
                let entity = entity.read(cx);
                let state = (self.state)(entity);
                motion_example(
                    label.clone(),
                    (self.enabled)(entity),
                    vec![transition_row(self.role, &state, &defaults)],
                )
            })
            .collect()
    }
}

pub struct ControlGroupMotionReport {
    samples: Vec<(SharedString, Arc<dyn Fn(&App) -> (bool, ControlGroupMotion) + Send + Sync>)>,
}

impl ControlGroupMotionReport {
    pub fn new(
        examples: impl IntoIterator<
            Item = (impl Into<SharedString>, Arc<dyn Fn(&App) -> (bool, ControlGroupMotion) + Send + Sync>),
        >,
    ) -> Self {
        Self { samples: examples.into_iter().map(|(label, read)| (label.into(), read)).collect() }
    }
}

impl MotionReport for ControlGroupMotionReport {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample> {
        let defaults = look.motion_defaults().control_group;
        self.samples
            .iter()
            .map(|(label, read)| {
                let (enabled, profile) = read(cx);
                motion_example(label.clone(), enabled, control_group_rows(&profile, &defaults))
            })
            .collect()
    }
}

pub fn control_group_motion_source<T>(
    group: Entity<ControlGroupControl<T>>,
) -> Arc<dyn Fn(&App) -> (bool, ControlGroupMotion) + Send + Sync>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |cx| {
        let group = group.read(cx);
        (group.motion_enabled(), group.motion_profile().clone())
    })
}

pub fn tabs_rows(example: &TabsMotion, defaults: &TabsMotion) -> Vec<MotionRow> {
    vec![
        transition_row("Indicator", &example.indicator, &defaults.indicator),
        transition_row("Content", &example.content, &defaults.content),
        transition_row("Disclosure", &example.disclosure, &defaults.disclosure),
        transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout),
    ]
}

pub fn switch_rows(example: &SwitchMotion, defaults: &SwitchMotion) -> Vec<MotionRow> {
    vec![transition_row("State", &example.state, &defaults.state)]
}

pub fn control_group_rows(example: &ControlGroupMotion, defaults: &ControlGroupMotion) -> Vec<MotionRow> {
    vec![transition_row("Selection", &example.selection, &defaults.selection)]
}

pub fn accordion_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<AccordionControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        AccordionControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().accordion.clone(),
        accordion_rows,
    )
}

pub fn sidebar_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<SidebarControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        SidebarControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().sidebar.clone(),
        sidebar_rows,
    )
}

pub fn stepper_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Stepper)>,
) -> SharedMotionReport {
    Arc::new(ProfileMotionReport::new(
        examples.into_iter().map(|(label, stepper)| {
            (
                label.into(),
                Arc::new(move |cx: &App| {
                    let stepper = stepper.read(cx);
                    (stepper.motion_enabled(), stepper.motion_profile().clone())
                }) as Arc<dyn Fn(&App) -> (bool, StepperMotion) + Send + Sync>,
            )
        }),
        |look| look.motion_defaults().stepper.clone(),
        stepper_rows,
    ))
}

pub fn resizable_panels_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<ResizablePanels>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        ResizablePanels::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().resizable_panels.clone(),
        resizable_panels_rows,
    )
}

pub fn slide_panel_motion_report(
    examples: impl IntoIterator<
        Item = (impl Into<SharedString>, Arc<dyn Fn(&App) -> (bool, SlidePanelMotion) + Send + Sync>),
    >,
) -> SharedMotionReport {
    Arc::new(ProfileMotionReport::new(
        examples,
        |look| look.motion_defaults().slide_panel.clone(),
        slide_panel_rows,
    ))
}

fn accordion_rows(example: &AccordionMotion, defaults: &AccordionMotion) -> Vec<MotionRow> {
    vec![
        transition_row("Expansion", &example.expansion, &defaults.expansion),
        transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout),
    ]
}

fn sidebar_rows(example: &SidebarMotion, defaults: &SidebarMotion) -> Vec<MotionRow> {
    let mut rows = vec![
        transition_row("Expansion", &example.expansion, &defaults.expansion),
        transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout),
    ];
    rows.extend(presence_rows("Popup enter", "Popup exit", &example.popup, &defaults.popup));
    rows
}

fn stepper_rows(example: &StepperMotion, defaults: &StepperMotion) -> Vec<MotionRow> {
    vec![transition_row("Selection", &example.selection, &defaults.selection)]
}

fn resizable_panels_rows(example: &ResizablePanelsMotion, defaults: &ResizablePanelsMotion) -> Vec<MotionRow> {
    vec![transition_row("Extent", &example.extent, &defaults.extent)]
}

fn slide_panel_rows(example: &SlidePanelMotion, defaults: &SlidePanelMotion) -> Vec<MotionRow> {
    presence_rows("Enter", "Exit", &example.presence, &defaults.presence)
}

pub fn tree_view_motion_source<T: Clone + Send + Sync + 'static>(
    tree: TreeView<T>,
) -> Arc<dyn Fn(&App) -> (bool, TreeViewMotion) + Send + Sync> {
    Arc::new(move |cx| {
        let tree = tree.read(cx);
        (tree.motion_enabled(), tree.motion_profile().clone())
    })
}

pub fn tree_view_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Arc<dyn Fn(&App) -> (bool, TreeViewMotion) + Send + Sync>)>,
) -> SharedMotionReport {
    Arc::new(ProfileMotionReport::new(examples, |look| look.motion_defaults().tree_view.clone(), tree_view_rows))
}

pub fn table_motion_report<T: 'static>(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<TableControl<T>>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        TableControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().table.clone(),
        table_rows,
    )
}

pub fn listbox_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Arc<dyn Fn(&App) -> (bool, ListBoxMotion) + Send + Sync>)>,
) -> SharedMotionReport {
    Arc::new(ProfileMotionReport::new(examples, |look| look.motion_defaults().listbox.clone(), listbox_rows))
}

pub fn slider_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<SliderControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        SliderControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().slider.clone(),
        slider_rows,
    )
}

pub fn progress_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Progress)>,
) -> SharedMotionReport {
    Arc::new(ProfileMotionReport::new(
        examples.into_iter().map(|(label, progress)| {
            (
                label.into(),
                Arc::new(move |cx: &App| {
                    let progress = progress.read(cx);
                    (progress.motion_enabled(), progress.motion_profile().clone())
                }) as Arc<dyn Fn(&App) -> (bool, ProgressMotion) + Send + Sync>,
            )
        }),
        |look| look.motion_defaults().progress.clone(),
        progress_rows,
    ))
}

pub fn popup_menu_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<PopupMenu>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        PopupMenu::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().popup_menu.clone(),
        popup_menu_rows,
    )
}

pub fn context_menu_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<ContextMenu>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        ContextMenu::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().context_menu.clone(),
        context_menu_rows,
    )
}

pub fn selector_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<Selector>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        Selector::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().selector.clone(),
        |example, defaults| presence_rows("Enter", "Exit", &example.presence, &defaults.presence),
    )
}

pub fn combobox_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<ComboBoxControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        ComboBoxControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().combobox.clone(),
        |example, defaults| presence_rows("Enter", "Exit", &example.presence, &defaults.presence),
    )
}

pub fn autocomplete_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<AutocompleteControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        AutocompleteControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().autocomplete.clone(),
        |example, defaults| presence_rows("Enter", "Exit", &example.presence, &defaults.presence),
    )
}

pub fn search_selector_motion_report(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<SearchSelectorControl>)>,
) -> SharedMotionReport {
    entity_report(
        examples,
        SearchSelectorControl::motion_enabled,
        |control| control.motion_profile().clone(),
        |look| look.motion_defaults().search_selector.clone(),
        search_selector_rows,
    )
}

fn tree_view_rows(example: &TreeViewMotion, defaults: &TreeViewMotion) -> Vec<MotionRow> {
    vec![
        transition_row("Expansion", &example.expansion, &defaults.expansion),
        transition_row("Position", &example.position, &defaults.position),
        transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout),
    ]
}

fn table_rows(example: &TableMotion, defaults: &TableMotion) -> Vec<MotionRow> {
    vec![transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout)]
}

fn listbox_rows(example: &ListBoxMotion, defaults: &ListBoxMotion) -> Vec<MotionRow> {
    vec![
        transition_row("Scroll", &example.scroll, &defaults.scroll),
        transition_row("Drop layout", &example.drop_layout, &defaults.drop_layout),
    ]
}

fn slider_rows(example: &SliderMotion, defaults: &SliderMotion) -> Vec<MotionRow> {
    vec![transition_row("Thumb", &example.thumb, &defaults.thumb)]
}

fn progress_rows(example: &ProgressMotion, defaults: &ProgressMotion) -> Vec<MotionRow> {
    vec![
        transition_row("Value", &example.value, &defaults.value),
        period_row("Period", example.period, defaults.period),
    ]
}

fn popup_menu_rows(example: &PopupMenuMotion, defaults: &PopupMenuMotion) -> Vec<MotionRow> {
    let mut rows = vec![
        transition_row("Submenu", &example.submenu, &defaults.submenu),
        transition_row("Highlight", &example.highlight, &defaults.highlight),
        transition_row("Disclosure", &example.disclosure, &defaults.disclosure),
    ];
    rows.extend(presence_rows("Enter", "Exit", &example.presence, &defaults.presence));
    rows
}

fn context_menu_rows(example: &ContextMenuMotion, defaults: &ContextMenuMotion) -> Vec<MotionRow> {
    let mut rows = presence_rows("Submenu enter", "Submenu exit", &example.submenu, &defaults.submenu);
    rows.push(transition_row("Disclosure", &example.disclosure, &defaults.disclosure));
    rows.extend(presence_rows("Enter", "Exit", &example.presence, &defaults.presence));
    rows
}

fn search_selector_rows(example: &SearchSelectorMotion, defaults: &SearchSelectorMotion) -> Vec<MotionRow> {
    let mut rows = vec![transition_row("Disclosure", &example.disclosure, &defaults.disclosure)];
    rows.extend(presence_rows("Enter", "Exit", &example.presence, &defaults.presence));
    rows
}

fn period_row(role: &str, example: Duration, defaults: Duration) -> MotionRow {
    MotionRow {
        role: role.into(),
        example: format_period(example),
        look_default: format_period(defaults),
        inherited: example == defaults,
    }
}

fn presence_rows(enter: &str, exit: &str, example: &PresenceMotion, defaults: &PresenceMotion) -> Vec<MotionRow> {
    vec![
        transition_row(enter, &example.enter, &defaults.enter),
        transition_row(exit, &example.exit, &defaults.exit),
    ]
}

struct ProfileMotionReport<P: 'static> {
    samples: Vec<(SharedString, Arc<dyn Fn(&App) -> (bool, P) + Send + Sync>)>,
    defaults: fn(&ShadcnLook) -> P,
    rows: fn(&P, &P) -> Vec<MotionRow>,
}

impl<P: 'static> ProfileMotionReport<P> {
    fn new(
        examples: impl IntoIterator<Item = (impl Into<SharedString>, Arc<dyn Fn(&App) -> (bool, P) + Send + Sync>)>,
        defaults: fn(&ShadcnLook) -> P,
        rows: fn(&P, &P) -> Vec<MotionRow>,
    ) -> Self {
        Self { samples: examples.into_iter().map(|(label, read)| (label.into(), read)).collect(), defaults, rows }
    }
}

impl<P: 'static> MotionReport for ProfileMotionReport<P> {
    fn examples(&self, look: &ShadcnLook, cx: &App) -> Vec<MotionExample> {
        let defaults = (self.defaults)(look);
        self.samples
            .iter()
            .map(|(label, read)| {
                let (enabled, profile) = read(cx);
                motion_example(label.clone(), enabled, (self.rows)(&profile, &defaults))
            })
            .collect()
    }
}

fn entity_report<T, P>(
    examples: impl IntoIterator<Item = (impl Into<SharedString>, Entity<T>)>,
    enabled: fn(&T) -> bool,
    profile: fn(&T) -> P,
    defaults: fn(&ShadcnLook) -> P,
    rows: fn(&P, &P) -> Vec<MotionRow>,
) -> SharedMotionReport
where
    T: 'static,
    P: 'static,
{
    Arc::new(ProfileMotionReport::new(
        examples.into_iter().map(|(label, entity)| (label.into(), entity_profile(entity, enabled, profile))),
        defaults,
        rows,
    ))
}

fn entity_profile<T, P>(
    entity: Entity<T>,
    enabled: fn(&T) -> bool,
    profile: fn(&T) -> P,
) -> Arc<dyn Fn(&App) -> (bool, P) + Send + Sync>
where
    T: 'static,
    P: 'static,
{
    Arc::new(move |cx| {
        let entity = entity.read(cx);
        (enabled(entity), profile(entity))
    })
}

fn motion_example(label: SharedString, enabled: bool, rows: Vec<MotionRow>) -> MotionExample {
    MotionExample { label, enabled, rows }
}

pub fn render_motion_examples(look: &Arc<ShadcnLook>, examples: &[MotionExample]) -> gpui::AnyElement {
    use gpui::{prelude::*, px};

    let mut column = vstack! { gap=8.0; };
    for example in examples {
        column = column.child(render_motion_example(look, example));
    }
    column.w_full().min_w(px(0.0)).into_any_element()
}

fn render_motion_example(look: &Arc<ShadcnLook>, example: &MotionExample) -> gpui::AnyElement {
    use gpui::{AnyElement, div, prelude::*, px};

    let chrome = look.chrome();
    let text = motion_text_style(look);
    let mut header = vstack! {
        gap=2.0;
        hstack! {
            gap=12.0 align=center justify=between;
            div()
                .min_w(px(0.0))
                .flex_1()
                .truncate()
                .typography_style(text)
                .text_color(chrome.title_text)
                .child(example.label.clone()),
            div()
                .flex_none()
                .whitespace_nowrap()
                .typography_style(text)
                .text_color(chrome.muted_text)
                .child(if example.enabled { "Enabled" } else { "Disabled" }),
        },
    }
    .px(px(9.0))
    .py(px(8.0));
    if !example.enabled {
        header = header.child(div().typography_style(text).text_color(chrome.muted_text).child(policy_label(false)));
    }
    let mut rows: Vec<AnyElement> = vec![header.into_any_element()];
    rows.extend(example.rows.iter().map(|row| render_motion_row(look, row)));
    InspectorSection::new(look, rows).into_any_element()
}

fn render_motion_row(look: &Arc<ShadcnLook>, row: &MotionRow) -> gpui::AnyElement {
    use gpui::{div, prelude::*, px};

    let chrome = look.chrome();
    let text = motion_text_style(look);
    let mut lines = vstack! {
        gap=2.0;
        hstack! {
            gap=12.0 align=center justify=between;
            div().flex_shrink_0().typography_style(text).text_color(chrome.body_text).child(row.role.clone()),
            detail_text(look).child(row.example.clone()),
        },
    };
    if !row.inherited {
        lines = lines.child(hstack! {
            justify=end;
            detail_text(look).child(format!("look default {}", row.look_default)),
        });
    }
    lines
        .w_full()
        .min_w(px(0.0))
        .border_t_1()
        .border_color(chrome.border)
        .px(px(9.0))
        .py(px(6.0))
        .into_any_element()
}

/// Same size, line height, and weight as the inspector accordion triggers.
fn motion_text_style(look: &ShadcnLook) -> LumaTextStyle {
    resolve_control_typography(&look.mode_tokens(), ControlSize::Md, false).style
}

/// Caption mono used by the other inspector property values.
fn detail_text(look: &ShadcnLook) -> gpui::Div {
    use gpui::{div, prelude::*, px};

    let caption = &look.mode_tokens().typography.text.caption;
    div()
        .whitespace_nowrap()
        .font_family(controls_mono_font())
        .text_size(px(caption.size))
        .line_height(px(caption.line_height))
        .text_color(look.chrome().muted_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn formatter_names_each_transition_kind() {
        assert_eq!(format_transition(&Transition::instant()), "instant");
        assert_eq!(
            format_transition(&Transition::timed(Duration::from_millis(180), Easing::OutCubic)),
            "180 ms · ease-out"
        );
        assert_eq!(format_transition(&Transition::spring(300.0, 28.0)), "spring · 300 / 28");
        assert_eq!(format_period(Duration::from_millis(900)), "period · 900 ms");
        assert_eq!(policy_label(false), "Motion disabled · authored profile kept");
    }

    #[test]
    fn role_row_marks_only_matching_descriptions_inherited() {
        let defaults = Transition::timed(Duration::from_millis(180), Easing::OutCubic);
        let same = Transition::timed(Duration::from_millis(180), Easing::OutCubic);
        let edited = Transition::instant();
        assert!(transition_row("Content", &same, &defaults).inherited);
        let row = transition_row("Content", &edited, &defaults);
        assert!(!row.inherited);
        assert_eq!(row.example, "instant");
        assert_eq!(row.look_default, "180 ms · ease-out");
    }

    #[test]
    fn switch_state_row_marks_an_override() {
        let defaults = SwitchMotion::default();
        let mut edited = defaults.clone();
        edited.state = Transition::timed(Duration::from_millis(400), Easing::Linear);
        let rows = switch_rows(&edited, &defaults);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].role, "State");
        assert!(!rows[0].inherited);
        assert_eq!(rows[0].example, "400 ms · linear");
        assert!(switch_rows(&defaults, &defaults)[0].inherited);
    }

    #[test]
    fn control_group_selection_row_marks_an_override() {
        let defaults = ControlGroupMotion::default();
        let mut edited = defaults.clone();
        edited.selection = Transition::timed(Duration::from_millis(400), Easing::Linear);
        let rows = control_group_rows(&edited, &defaults);
        assert_eq!(rows[0].role, "Selection");
        assert!(!rows[0].inherited);
        assert_eq!(rows[0].example, "400 ms · linear");
        assert!(control_group_rows(&defaults, &defaults)[0].inherited);
    }

    #[test]
    fn accordion_rows_mark_an_override_and_keep_the_other_role() {
        let defaults = AccordionMotion::default();
        let mut edited = defaults.clone();
        edited.expansion = Transition::timed(Duration::from_millis(400), Easing::Linear);
        let rows = accordion_rows(&edited, &defaults);
        assert_eq!(rows[0].role, "Expansion");
        assert!(!rows[0].inherited);
        assert_eq!(rows[0].example, "400 ms · linear");
        assert!(rows[1].inherited);
        assert_eq!(rows[1].role, "Drop layout");
    }

    #[test]
    fn presence_rows_name_enter_and_exit() {
        let defaults = PresenceMotion::default();
        let mut edited = defaults.clone();
        edited.exit = Transition::instant();
        let rows = presence_rows("Enter", "Exit", &edited, &defaults);
        assert_eq!(rows[0].role, "Enter");
        assert!(rows[0].inherited);
        assert_eq!(rows[1].role, "Exit");
        assert!(!rows[1].inherited);
        assert_eq!(rows[1].example, "instant");
    }
}
