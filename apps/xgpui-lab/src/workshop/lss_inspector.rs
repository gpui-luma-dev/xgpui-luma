//! `.lss` inspector for a Workshop work pane (issue #291): one control's style laid out like its file.
//! Properties lists each part's properties with the resolved value, the rule that set it and the
//! expression as written; a property opens to every rule for it (wins, beaten or inactive for the
//! key) and what the winning value read. Source shows the file with each rule line marked for the key.
//! Driven only by `Inspection` and `Explanation` from the style crate. Read-only.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FontWeight, Hsla, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::Button;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::{
    Category, ControlId, Inspection, PartInspection, Row, RuleInfo, Source, StateId, StyleKey, Theme, Value,
    display_value,
};

use super::box_model;
use crate::lab_controls::Segmented;
use crate::theme::{ColorVizLookExt, Look, MONO_FAMILY, TextSize, TypographyExt};

const NAME_WIDTH: f32 = 200.;
const ROW_INDENT: f32 = 22.;

#[derive(Clone, Copy, PartialEq)]
enum View {
    Properties,
    Source,
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    /// Properties with at least one state rule.
    HasStates,
    /// Properties whose value comes from a state rule for the current key.
    ByStateNow,
}

#[derive(Clone, Copy, PartialEq)]
enum Grouping {
    /// One section per part, rows in file order.
    File,
    /// One section per category (colors, layout, …).
    Kind,
}

#[derive(Clone, Copy, PartialEq)]
enum RuleStatus {
    Wins,
    Beaten,
    Inactive,
}

impl RuleStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Wins => "wins",
            Self::Beaten => "beaten",
            Self::Inactive => "inactive",
        }
    }
}

/// A property's identity in the inspector: part index and property name.
type RowId = (usize, &'static str);

pub(super) struct LssInspector {
    look: Arc<Look>,
    theme: Theme,
    control: ControlId,
    states: Vec<StateId>,
    key: StyleKey,
    /// The control's style file, shown by the Source view.
    source: Option<Source>,
    part: usize,
    view: View,
    filter: Filter,
    grouping: Grouping,
    view_tabs: Segmented,
    part_tabs: Segmented,
    filter_tabs: Segmented,
    grouping_tabs: Segmented,
    /// Each property's name, as a button that opens its rules and references.
    names: HashMap<RowId, Entity<Button>>,
    expanded: HashSet<RowId>,
}

impl LssInspector {
    pub(super) fn new(look: Arc<Look>, theme: Theme, control: ControlId, cx: &mut Context<Self>) -> Self {
        let key = StyleKey::new(control);
        let states = theme.stylesheet().axes(control).states;
        let inspection = theme.inspect(&key, None);
        let parts: Vec<SharedString> = inspection.parts.iter().map(|p| p.part.name().to_string().into()).collect();
        let source = control_source(&inspection)
            .and_then(|(file, _)| theme.stylesheet().sources().find(|(_, s)| *s.name == *file).map(|(_, s)| s.clone()));

        let view_tabs = Segmented::new(
            "lss-inspector-view",
            "",
            look.as_ref(),
            &["Properties", "Source"],
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.view = if i == 0 { View::Properties } else { View::Source };
                cx.notify();
            },
        );
        let part_names: Vec<&str> = parts.iter().map(|p| p.as_ref()).collect();
        let part_tabs = Segmented::new(
            "lss-inspector-part",
            "Part",
            look.as_ref(),
            &part_names,
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.part = i;
                cx.notify();
            },
        );
        let filter_tabs = Segmented::new(
            "lss-inspector-filter",
            "Show",
            look.as_ref(),
            &["All", "Has states", "By state now"],
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.filter = [Filter::All, Filter::HasStates, Filter::ByStateNow][i];
                cx.notify();
            },
        );
        let grouping_tabs = Segmented::new(
            "lss-inspector-grouping",
            "Group",
            look.as_ref(),
            &["File", "Kind"],
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.grouping = if i == 0 { Grouping::File } else { Grouping::Kind };
                cx.notify();
            },
        );

        let accent = look.accent(11);
        let mut names = HashMap::new();
        for (index, part) in inspection.parts.iter().enumerate() {
            for row in &part.rows {
                let (property, kind) = (row.property, row.kind.name());
                let button =
                    xgpui_luma_look_radix::Button::new(format!("lss-inspector-name-{}-{property}", part.part.name()))
                        .look(look.as_ref())
                        .content_only()
                        .content(move |_, _| {
                            div()
                                .flex()
                                .font_family(MONO_FAMILY)
                                .child(property)
                                .child(div().text_color(accent).child(format!(": {kind}")))
                        })
                        .on_click(cx, move |this: &mut Self, cx| this.toggle((index, property), cx))
                        .spawn(cx);
                names.insert((index, property), button);
            }
        }

        Self {
            look,
            theme,
            control,
            states,
            key,
            source,
            part: 0,
            view: View::Properties,
            filter: Filter::All,
            grouping: Grouping::File,
            view_tabs,
            part_tabs,
            filter_tabs,
            grouping_tabs,
            names,
            expanded: HashSet::new(),
        }
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    /// Inspect the control for `key` (the pane's style key).
    pub(super) fn set_key(&mut self, key: StyleKey, cx: &mut Context<Self>) {
        self.key = key;
        cx.notify();
    }

    fn toggle(&mut self, row: RowId, cx: &mut Context<Self>) {
        if !self.expanded.remove(&row) {
            self.expanded.insert(row);
        }
        cx.notify();
    }

    /// `key` with every declared state on. Every state rule matches it, so its explanation lists all of a
    /// property's rules; the real key's explanation then says which of them win, lose or do not apply.
    fn every_state(&self) -> StyleKey {
        self.states.iter().fold(self.key, |key, &state| key.set_state(state, true))
    }

    /// Every rule for one property, in file order, with its status for the current key.
    fn rules(&self, part: &PartInspection, row: &Row) -> Vec<(RuleInfo, RuleStatus)> {
        let every = self.theme.explain(&self.every_state(), part.part, row.property);
        let actual = &row.explanation;
        let mut rules: Vec<(RuleInfo, RuleStatus)> = every
            .winner
            .into_iter()
            .chain(every.overridden)
            .map(|rule| {
                let status = if actual.winner.as_ref() == Some(&rule) {
                    RuleStatus::Wins
                } else if actual.overridden.contains(&rule) {
                    RuleStatus::Beaten
                } else {
                    RuleStatus::Inactive
                };
                (rule, status)
            })
            .collect();
        rules.sort_by_key(|(rule, _)| (rule.line, rule.column));
        rules
    }

    fn keep(&self, row: &Row, rules: &[(RuleInfo, RuleStatus)]) -> bool {
        match self.filter {
            Filter::All => true,
            Filter::HasStates => rules.iter().any(|(rule, _)| !rule.states.is_empty()),
            Filter::ByStateNow => by_state(row),
        }
    }

    fn properties(&self, inspection: &Inspection) -> AnyElement {
        let chrome = self.look.chrome();
        let Some(part) = inspection.parts.get(self.part) else {
            return div().into_any_element();
        };
        let mut rows: Vec<(&Row, Vec<(RuleInfo, RuleStatus)>)> = part
            .rows
            .iter()
            .map(|row| (row, self.rules(part, row)))
            .filter(|(row, rules)| self.keep(row, rules))
            .collect();
        let diagram = || {
            let id = format!("style-inspector-box-{}", part.part.name());
            div().pl(px(ROW_INDENT)).pr(px(12.)).py(px(6.)).child(box_model::diagram(
                id.into(),
                &part.box_model,
                self.look.mode(),
            ))
        };
        let sections: Vec<AnyElement> = match self.grouping {
            Grouping::File => {
                rows.sort_by_key(|(row, _)| row.explanation.winner.as_ref().map_or(u32::MAX, |w| w.line));
                let title = if self.part == 0 {
                    format!("{} {{ … }}", self.control)
                } else {
                    format!("part {} {{ … }}", part.part.name())
                };
                vec![self.section(part, title, rows.iter().collect(), Some(diagram().into_any_element()))]
            }
            Grouping::Kind => [
                (Category::Colors, "Colors"),
                (Category::Layout, "Layout"),
                (Category::Typography, "Typography"),
                (Category::Elevation, "Elevation"),
                (Category::Motion, "Motion"),
                (Category::Other, "Other"),
            ]
            .into_iter()
            .filter_map(|(category, title)| {
                let in_category: Vec<_> = rows.iter().filter(|(row, _)| row.category == category).collect();
                let box_diagram = (category == Category::Layout).then(|| diagram().into_any_element());
                (box_diagram.is_some() || !in_category.is_empty())
                    .then(|| self.section(part, title.to_string(), in_category, box_diagram))
            })
            .collect(),
        };
        div()
            .id("lss-inspector-properties")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .text_color(chrome.muted_text)
                    .child(part.doc.clone().map_or_else(String::new, |d| format!("/// {d}"))),
            )
            .children(sections)
            .into_any_element()
    }

    fn section(
        &self,
        part: &PartInspection,
        title: String,
        rows: Vec<&(&Row, Vec<(RuleInfo, RuleStatus)>)>,
        diagram: Option<AnyElement>,
    ) -> AnyElement {
        let chrome = self.look.chrome();
        let set_by_state = rows.iter().filter(|(row, _)| by_state(row)).count();
        let name = part.part.name();
        div()
            .debug_selector(move || format!("style-inspector-part-{name}"))
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(8.))
                    .px(px(12.))
                    .py(px(5.))
                    .border_t_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .child(div().font_family(MONO_FAMILY).text_color(chrome.title_text).child(title))
                    .child(div().text_color(chrome.muted_text).child(format!("{} properties", rows.len())))
                    .child(div().flex_1())
                    .when(set_by_state > 0, |d| {
                        d.child(div().text_color(self.look.accent(11)).child(format!("{set_by_state} set by state")))
                    }),
            )
            .children(diagram)
            .children(rows.into_iter().map(|(row, rules)| self.row(part, row, rules)))
            .into_any_element()
    }

    fn row(&self, part: &PartInspection, row: &Row, rules: &[(RuleInfo, RuleStatus)]) -> AnyElement {
        let chrome = self.look.chrome();
        let e = &row.explanation;
        let id: RowId = (self.part, row.property);
        let expanded = self.expanded.contains(&id);
        let swatch = match &e.value {
            Some(Value::Color(c)) => Some(Hsla::from(*c)),
            _ => None,
        };
        let changed = by_state(row);
        let (selector, line) = match &e.winner {
            Some(w) => (selector(w), format!(":{}", w.line)),
            None => (e.source_line(), String::new()),
        };
        let expression = e.winner.as_ref().map_or_else(String::new, |w| format!("= {}", w.expression));
        let name: AnyElement = match self.names.get(&id) {
            Some(button) => button.clone().into_any_element(),
            None => div().font_family(MONO_FAMILY).child(row.property).into_any_element(),
        };
        let selector_id = format!("style-inspector-row-{}-{}", part.part.name(), row.property);
        div()
            .debug_selector(move || selector_id.clone())
            .w_full()
            .flex()
            .flex_col()
            .when(expanded, |d| d.bg(chrome.panel_background))
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .pl(px(ROW_INDENT))
                    .pr(px(12.))
                    .pt(px(4.))
                    .when(changed, |d| d.child(self.dot()))
                    .child(div().w(px(NAME_WIDTH)).flex_shrink_0().child(name))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .font_family(MONO_FAMILY)
                            .children(swatch.map(|c| self.swatch(c)))
                            .child(div().truncate().child(display_value(e.value.as_ref(), row.kind))),
                    )
                    .child(self.pill(selector, changed))
                    .child(
                        div()
                            .w(px(30.))
                            .flex_shrink_0()
                            .font_family(MONO_FAMILY)
                            .text_color(chrome.muted_text)
                            .text_right()
                            .child(line),
                    ),
            )
            .child(
                div()
                    .pl(px(ROW_INDENT))
                    .pr(px(12.))
                    .pb(px(4.))
                    .font_family(MONO_FAMILY)
                    .text_color(chrome.muted_text)
                    .truncate()
                    .child(expression),
            )
            .when(expanded, |d| d.child(self.details(row, rules)))
            .into_any_element()
    }

    /// The rules for a property and what its winning value read.
    fn details(&self, row: &Row, rules: &[(RuleInfo, RuleStatus)]) -> AnyElement {
        let chrome = self.look.chrome();
        let heading = |text: String| {
            div().font_weight(FontWeight::SEMIBOLD).text_color(chrome.muted_text).child(text.to_uppercase())
        };
        let references = row.explanation.reference_lines();
        let no_references = references.is_empty();
        div()
            .mx(px(12.))
            .ml(px(ROW_INDENT))
            .mb(px(8.))
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(4.))
            .rounded(px(6.))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.content_background)
            .children(row.doc.clone().map(|d| div().text_color(chrome.body_text).child(format!("/// {d}"))))
            .child(heading(format!("Rules for {}", row.property)))
            .children(rules.iter().map(|(rule, status)| self.rule_line(rule, *status)))
            .child(heading("Reads".to_string()))
            .children(
                references
                    .into_iter()
                    .map(|line| div().font_family(MONO_FAMILY).text_color(chrome.body_text).child(line)),
            )
            .when(no_references, |d| {
                d.child(div().font_family(MONO_FAMILY).text_color(chrome.muted_text).child("literal — reads nothing"))
            })
            .children(
                row.explanation
                    .notes()
                    .map(|n| div().font_family(MONO_FAMILY).text_color(chrome.muted_text).child(n)),
            )
            .into_any_element()
    }

    fn rule_line(&self, rule: &RuleInfo, status: RuleStatus) -> AnyElement {
        let chrome = self.look.chrome();
        let color = match status {
            RuleStatus::Wins => self.look.accent(11),
            RuleStatus::Beaten => chrome.body_text,
            RuleStatus::Inactive => chrome.muted_text,
        };
        div()
            .flex()
            .gap(px(8.))
            .px(px(4.))
            .rounded(px(4.))
            .font_family(MONO_FAMILY)
            .text_color(color)
            .when(status == RuleStatus::Wins, |d| d.bg(self.look.accent(3)))
            .when(status == RuleStatus::Inactive, |d| d.opacity(0.6))
            .child(div().w(px(30.)).flex_shrink_0().child(format!(":{}", rule.line)))
            .child(div().w(px(110.)).flex_shrink_0().child(selector(rule)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .when(status == RuleStatus::Beaten, |d| d.line_through())
                    .child(rule.expression.clone()),
            )
            .child(div().flex_shrink_0().child(status.label()))
            .into_any_element()
    }

    /// The control's style file, each rule line marked for the current key.
    fn source(&self, inspection: &Inspection) -> AnyElement {
        let chrome = self.look.chrome();
        let Some(source) = &self.source else {
            return div().p(px(12.)).text_color(chrome.muted_text).child("No style file found.").into_any_element();
        };
        let mut lines: HashMap<u32, Vec<RuleStatus>> = HashMap::new();
        for part in &inspection.parts {
            for row in &part.rows {
                for (rule, status) in self.rules(part, row) {
                    if *rule.source == *source.name {
                        lines.entry(rule.line).or_default().push(status);
                    }
                }
            }
        }
        let accent_soft = self.look.accent(4);
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .gap(px(14.))
                    .px(px(12.))
                    .py(px(6.))
                    .border_b_1()
                    .border_color(chrome.border)
                    .text_color(chrome.muted_text)
                    .child(div().font_family(MONO_FAMILY).text_color(chrome.title_text).child(source.name.to_string()))
                    .child(div().flex_1())
                    .child(div().px(px(4.)).bg(accent_soft).child("wins for key"))
                    .child(div().line_through().child("beaten"))
                    .child(div().opacity(0.45).child("state not active")),
            )
            .child(
                div()
                    .id("lss-inspector-source")
                    .debug_selector(|| "lss-inspector-source".into())
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .py(px(6.))
                    .font_family(MONO_FAMILY)
                    .children(source.text.lines().enumerate().map(|(index, text)| {
                        let number = index as u32 + 1;
                        let statuses = lines.get(&number);
                        let status = statuses.map(|s| {
                            if s.contains(&RuleStatus::Wins) {
                                RuleStatus::Wins
                            } else if s.contains(&RuleStatus::Beaten) {
                                RuleStatus::Beaten
                            } else {
                                RuleStatus::Inactive
                            }
                        });
                        let trimmed = text.trim_start();
                        let color = if trimmed.starts_with("//") {
                            chrome.muted_text
                        } else {
                            chrome.body_text
                        };
                        div()
                            .flex()
                            .whitespace_nowrap()
                            .when(status == Some(RuleStatus::Wins), |d| d.bg(accent_soft))
                            .child(
                                div()
                                    .w(px(30.))
                                    .flex_shrink_0()
                                    .pr(px(10.))
                                    .text_right()
                                    .text_color(chrome.muted_text)
                                    .child(number.to_string()),
                            )
                            .child(
                                div()
                                    .text_color(color)
                                    .when(status == Some(RuleStatus::Beaten), |d| d.line_through())
                                    .when(status == Some(RuleStatus::Inactive), |d| d.opacity(0.45))
                                    .child(if text.is_empty() {
                                        " ".to_string()
                                    } else {
                                        text.to_string()
                                    }),
                            )
                    })),
            )
            .into_any_element()
    }

    fn dot(&self) -> AnyElement {
        div()
            .absolute()
            .left(px(9.))
            .top(px(10.))
            .size(px(6.))
            .rounded(px(3.))
            .bg(self.look.accent(9))
            .into_any_element()
    }

    fn swatch(&self, color: Hsla) -> AnyElement {
        div()
            .size(px(12.))
            .flex_shrink_0()
            .rounded(px(2.))
            .border_1()
            .border_color(self.look.chrome().border)
            .bg(color)
            .into_any_element()
    }

    fn pill(&self, text: String, accent: bool) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .flex_shrink_0()
            .px(px(6.))
            .rounded(px(4.))
            .font_family(MONO_FAMILY)
            .bg(if accent {
                self.look.accent(3)
            } else {
                chrome.panel_background
            })
            .text_color(if accent {
                self.look.accent(11)
            } else {
                chrome.muted_text
            })
            .child(text)
            .into_any_element()
    }

    /// The current inspection, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn inspection(&self) -> Inspection {
        self.theme.inspect(&self.key, None)
    }

    #[cfg(all(test, feature = "test-support"))]
    fn part_index(&self, name: &str) -> Option<usize> {
        self.inspection().parts.iter().position(|p| p.part.name() == name)
    }

    /// Show a part, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn select_part(&self, name: &str, cx: &mut gpui::App) {
        if let Some(index) = self.part_index(name) {
            self.part_tabs.select(index, cx);
        }
    }

    /// Show the Source view, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn show_source(&self, cx: &mut gpui::App) {
        self.view_tabs.select(1, cx);
    }

    /// Open a property's rules, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn expand(&mut self, part: &str, property: &'static str, cx: &mut Context<Self>) {
        if let Some(index) = self.part_index(part) {
            self.toggle((index, property), cx);
        }
    }
}

impl Render for LssInspector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let inspection = self.theme.inspect(&self.key, None);
        let toolbar = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(12.))
            .py(px(8.))
            .border_b_1()
            .border_color(chrome.border)
            .child(self.view_tabs.clone())
            .child(div().flex_1())
            .child(div().font_family(MONO_FAMILY).text_color(chrome.muted_text).child(key_line(
                &self.theme,
                self.control,
                &self.key,
            )));
        let body = match self.view {
            View::Properties => div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .px(px(12.))
                        .py(px(8.))
                        .border_b_1()
                        .border_color(chrome.border)
                        .child(self.part_tabs.clone())
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(12.))
                                .child(self.filter_tabs.clone())
                                .child(self.grouping_tabs.clone()),
                        ),
                )
                .child(self.properties(&inspection))
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .gap(px(14.))
                        .px(px(12.))
                        .py(px(6.))
                        .border_t_1()
                        .border_color(chrome.border)
                        .text_color(chrome.muted_text)
                        .child("● value from a state rule")
                        .child("click a property for its rules and reads"),
                ),
            View::Source => div().flex_1().min_h_0().flex().flex_col().child(self.source(&inspection)),
        };
        div()
            .id("style-inspector")
            .debug_selector(|| "style-inspector".into())
            .size_full()
            .flex()
            .flex_col()
            .typography_style(self.look.typography_scale(TextSize::Xs))
            .text_color(chrome.body_text)
            .child(toolbar)
            .child(body)
    }
}

/// Whether a row's value comes from a state rule.
fn by_state(row: &Row) -> bool {
    row.explanation.winner.as_ref().is_some_and(|w| !w.states.is_empty())
}

/// A rule's selector as written: scope and states, or `base`.
fn selector(rule: &RuleInfo) -> String {
    let parts: Vec<&str> = [rule.scope.as_str(), rule.states.as_str()].into_iter().filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        "base".to_string()
    } else {
        parts.join(" ")
    }
}

/// The style key as text, e.g. `checkbox:checked:hover`.
pub(super) fn key_line(theme: &Theme, control: ControlId, key: &StyleKey) -> String {
    let config = theme.stylesheet().config();
    let states: String = theme
        .stylesheet()
        .axes(control)
        .states
        .into_iter()
        .filter(|s| key.active_states().contains(*s))
        .map(|s| format!(":{}", config.state_name(s)))
        .collect();
    format!("{control}{states}")
}

/// The file and layer a control's rules come from, read from the first rule that sets a value.
pub(super) fn control_source(inspection: &Inspection) -> Option<(String, String)> {
    inspection
        .parts
        .iter()
        .flat_map(|p| &p.rows)
        .find_map(|r| r.explanation.winner.as_ref())
        .map(|w| (w.source.to_string(), w.layer.clone()))
}

trait AccentExt {
    /// A step of the look's accent scale.
    fn accent(&self, step: u8) -> Hsla;
}

impl AccentExt for Look {
    fn accent(&self, step: u8) -> Hsla {
        self.resolve_step(xgpui_luma_look_radix::ScaleFamily::Color, step).hsla()
    }
}
