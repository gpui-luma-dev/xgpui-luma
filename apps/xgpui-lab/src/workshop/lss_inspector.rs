//! `.lss` inspector for a Workshop work pane (issue #291): one control's style laid out like its file.
//! Properties lists each part's properties with the resolved value, the rule that set it and the
//! expression as written; a property opens to every rule for it (wins, beaten or inactive for the
//! key) and what the winning value read. Source shows the file with each rule line marked for the key.
//! Driven only by `Inspection` and `Explanation` from the style crate. Read-only.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use gpui::{AnyElement, Context, Div, Entity, Hsla, Render, SharedString, WeakEntity, Window, div, prelude::*, px};
use xgpui_luma::controls::accordion::{Accordion, AccordionContent, AccordionEvent, AccordionItem, AccordionTrigger};
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::{
    Category, ControlId, Inspection, PartInspection, Reference, Row, RuleInfo, Source, StateId, StyleKey, Theme, Value,
    display_value,
};

use super::box_model;
use xgpui_luma::controls::tabs::TabsItem;
use super::pane_type;
use crate::theme::{ColorVizLookExt, Look, TypographyExt};

/// Height of a work pane's tab rows (tabs sit on the row's bottom border); the control side's stage
/// tabs and the inspector's view tabs share it so the two rows line up.
pub(super) const TAB_ROW_HEIGHT: f32 = 48.;
const ROW_INDENT: f32 = 22.;
/// Property rows: the name fills the line; the value (at most this wide) and the rule line sit at the end.
const VALUE_MAX_WIDTH: f32 = 140.;
/// Room for a rule line such as `:142`, so lines align at the right.
const LINE_WIDTH: f32 = 32.;
const COLUMN_GAP: f32 = 10.;
const ROW_HEIGHT: f32 = 24.;

#[derive(Clone, Copy, PartialEq)]
enum View {
    Properties,
    Source,
    Palette,
}

/// How one line of a style file is drawn.
#[derive(Clone, Copy, Default)]
struct LineMark {
    highlight: bool,
    strike: bool,
    dim: bool,
    swatch: Option<Hsla>,
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

/// What one property row shows. Written for each visible row when the inspector renders, then read by
/// that row's button content (the button is a retained SDK control, so it can't take the row as an
/// argument).
#[derive(Clone)]
struct RowHead {
    value: SharedString,
    swatch: Option<Hsla>,
    /// The winning rule's selector when a state or scope rule set the value; `None` for the base.
    selector: Option<SharedString>,
    line: SharedString,
    colors: RowColors,
}

#[derive(Clone, Copy)]
struct RowColors {
    text: Hsla,
    muted: Hsla,
    accent: Hsla,
    accent_soft: Hsla,
    border: Hsla,
}

/// What a picked part's accordion trigger shows.
#[derive(Clone)]
struct SectionHead {
    title: SharedString,
    count: SharedString,
    /// "n set by state", when the key's state rules set any of the part's values.
    by_state: Option<SharedString>,
    colors: RowColors,
}

/// The property list's shared data: what each row and each part's accordion trigger shows, and whether
/// the kind column fits. Written when the inspector renders; read by the rows' buttons and the triggers
/// (retained SDK controls, so they can't take it as an argument).
#[derive(Clone, Default)]
struct RowHeads {
    rows: Arc<RwLock<HashMap<RowId, RowHead>>>,
    sections: Arc<RwLock<HashMap<usize, SectionHead>>>,
}

/// One picked part's accordion content: the part's properties, drawn by the inspector.
struct PartBody {
    inspector: WeakEntity<LssInspector>,
    part: usize,
}

impl Render for PartBody {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let part = self.part;
        let body = self.inspector.update(cx, |inspector, _| inspector.part_body(part)).ok();
        div().w_full().children(body)
    }
}

pub(super) struct LssInspector {
    look: Arc<Look>,
    theme: Theme,
    control: ControlId,
    states: Vec<StateId>,
    key: StyleKey,
    /// The control's style file, shown by the Source view.
    source: Option<Source>,
    /// The theme's palette file (`theme/palette.lss` for the SDK: light values and `mode dark` overrides).
    palette: Option<Source>,
    /// The parts picked in the Part picker, in part order; each gets an accordion item.
    picked: Vec<usize>,
    /// Parts whose accordion item is open (all start open).
    open_parts: HashSet<usize>,
    /// The picked parts' accordion, rebuilt when the picks change.
    parts_accordion: Option<Accordion>,
    part_bodies: Vec<Entity<PartBody>>,
    view: View,
    filter: Filter,
    grouping: Grouping,
    /// Radix tabs: Properties · Source · Palette.
    view_tabs: Entity<xgpui_luma::controls::tabs::Tabs>,
    /// Radix toggle groups: Part (multiple), Show and Group (one each).
    part_tabs: ControlGroup<ControlGroupItem>,
    filter_tabs: ControlGroup<ControlGroupItem>,
    grouping_tabs: ControlGroup<ControlGroupItem>,
    /// Each property's row, as a button that opens its rules and references.
    row_buttons: HashMap<RowId, Entity<Button>>,
    row_heads: RowHeads,
    expanded: HashSet<RowId>,
}

impl LssInspector {
    pub(super) fn new(look: Arc<Look>, theme: Theme, control: ControlId, cx: &mut Context<Self>) -> Self {
        let key = StyleKey::new(control);
        let states = theme.stylesheet().axes(control).states;
        let inspection = theme.inspect(&key, None);
        let parts: Vec<SharedString> = inspection.parts.iter().map(|p| p.part.name().to_string().into()).collect();
        let palette = theme.stylesheet().sources().find(|(_, s)| s.name.contains("palette")).map(|(_, s)| s.clone());
        let source = control_source(&inspection)
            .and_then(|(file, _)| theme.stylesheet().sources().find(|(_, s)| *s.name == *file).map(|(_, s)| s.clone()));

        let view_tabs = xgpui_luma_look_radix::Tabs::new("lss-inspector-view")
            .look(look.as_ref())
            .items([
                TabsItem::new("properties").label("Properties"),
                TabsItem::new("source").label("Source"),
                TabsItem::new("palette").label("Palette"),
            ])
            .active("properties")
            .on_value_changed(cx, |this: &mut Self, id, _, cx| {
                this.view = match id.as_ref() {
                    "source" => View::Source,
                    "palette" => View::Palette,
                    _ => View::Properties,
                };
                cx.notify();
            })
            .spawn(cx);
        // Part, Show and Group are Radix toggle groups at the small size (Radix size 1).
        let small = look.with_size(xgpui_luma_look_radix::RadixSize::One);
        let part_tabs = xgpui_luma_look_radix::control_group_builder(&small, "lss-inspector-part", small.size())
            .horizontal()
            .multiple()
            // Each click turns one part on or off; without this a plain click selects only that part.
            .independent_toggles(true)
            .items(parts.iter().map(|p| ControlGroupItem::new(p.clone()).label(p.clone())))
            .selected_ids([parts[0].clone()])
            .spawn(cx);
        let part_ids = parts.clone();
        cx.subscribe(&part_tabs, move |this: &mut Self, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event {
                // In part order, whatever order they were picked in.
                this.picked = (0..part_ids.len()).filter(|&i| selected_ids.contains(&part_ids[i])).collect();
                this.rebuild_parts(cx);
            }
        })
        .detach();
        let inspector = cx.weak_entity();
        let part_bodies =
            (0..parts.len()).map(|part| cx.new(|_| PartBody { inspector: inspector.clone(), part })).collect();
        let filter_tabs = xgpui_luma_look_radix::control_group_builder(&small, "lss-inspector-filter", small.size())
            .horizontal()
            .single_required()
            .items([
                ControlGroupItem::new("all").label("All"),
                ControlGroupItem::new("has-states").label("Has states"),
                ControlGroupItem::new("by-state-now").label("By state now"),
            ])
            .selected_ids(["all"])
            .spawn(cx);
        cx.subscribe(&filter_tabs, |this: &mut Self, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event {
                this.filter = match selected_ids.first().map(|id| id.as_ref()) {
                    Some("has-states") => Filter::HasStates,
                    Some("by-state-now") => Filter::ByStateNow,
                    _ => Filter::All,
                };
                cx.notify();
            }
        })
        .detach();
        let grouping_tabs =
            xgpui_luma_look_radix::control_group_builder(&small, "lss-inspector-grouping", small.size())
                .horizontal()
                .single_required()
                .items([ControlGroupItem::new("file").label("File"), ControlGroupItem::new("kind").label("Kind")])
                .selected_ids(["file"])
                .spawn(cx);
        cx.subscribe(&grouping_tabs, |this: &mut Self, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event {
                this.grouping = if selected_ids.first().is_some_and(|id| id.as_ref() == "kind") {
                    Grouping::Kind
                } else {
                    Grouping::File
                };
                cx.notify();
            }
        })
        .detach();

        let row_heads = RowHeads::default();
        let mut row_buttons = HashMap::new();
        for (index, part) in inspection.parts.iter().enumerate() {
            let name = part.part.name();
            for row in &part.rows {
                let property = row.property;
                let id: RowId = (index, property);
                let heads = row_heads.clone();
                let button = list_button(
                    format!("lss-inspector-row-{name}-{property}"),
                    &look,
                    move || {
                        let head = heads.rows.read().ok().and_then(|rows| rows.get(&id).cloned());
                        row_columns(property, head.as_ref())
                    },
                    move |this, cx| this.toggle(id, cx),
                    cx,
                );
                row_buttons.insert(id, button);
            }
        }

        let mut this = Self {
            look,
            theme,
            control,
            states,
            key,
            source,
            palette,
            picked: vec![0],
            open_parts: (0..parts.len()).collect(),
            parts_accordion: None,
            part_bodies,
            view: View::Properties,
            filter: Filter::All,
            grouping: Grouping::File,
            view_tabs,
            part_tabs,
            filter_tabs,
            grouping_tabs,
            row_buttons,
            row_heads,
            expanded: HashSet::new(),
        };
        this.rebuild_parts(cx);
        this
    }

    /// Rebuild the picked parts' accordion: one item per picked part, in part order, each opening to the
    /// part's properties. Items keep whether they were open.
    fn rebuild_parts(&mut self, cx: &mut Context<Self>) {
        let names: Vec<String> =
            self.theme.inspect(&self.key, None).parts.iter().map(|p| p.part.name().to_string()).collect();
        let items: Vec<AccordionItem> = self
            .picked
            .iter()
            .filter_map(|&index| {
                let (name, body) = (names.get(index)?.clone(), self.part_bodies.get(index)?.clone());
                let sections = self.row_heads.sections.clone();
                let trigger = AccordionTrigger::custom(move |_, _| {
                    section_trigger(sections.read().ok().and_then(|s| s.get(&index).cloned()).as_ref())
                        .into_any_element()
                });
                let content = AccordionContent::custom(move |_, _| body.clone().into_any_element());
                Some(AccordionItem::new(name, trigger, content).expanded(self.open_parts.contains(&index)))
            })
            .collect();
        let accordion = xgpui_luma_look_radix::accordion_builder(&self.look, "lss-inspector-parts")
            .multiple()
            .collapsible(true)
            .items(items)
            .spawn(cx);
        cx.subscribe(&accordion, move |this: &mut Self, _, event: &AccordionEvent, _cx| {
            if let AccordionEvent::ExpandedChanged { item_id, expanded } = event
                && let Some(index) = names.iter().position(|n| n.as_str() == item_id.as_ref())
            {
                if *expanded {
                    this.open_parts.insert(index);
                } else {
                    this.open_parts.remove(&index);
                }
            }
        })
        .detach();
        self.parts_accordion = Some(accordion);
        cx.notify();
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    /// Inspect with `theme`: the same stylesheet in another mode (the pane's Modes picker).
    pub(super) fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    /// Inspect the control for `key` (the pane's style key).
    pub(super) fn set_key(&mut self, key: StyleKey, cx: &mut Context<Self>) {
        self.key = key;
        cx.notify();
    }

    /// A note naming the key's active states that no rule uses (e.g. `:focused`, which only the
    /// template draws), since they change no value here.
    pub(super) fn unused_states_note(&self) -> Option<String> {
        let config = self.theme.stylesheet().config();
        let unused: Vec<String> = config
            .state_names()
            .filter_map(|n| config.state(n).map(|s| (n, s)))
            .filter(|(_, s)| self.key.active_states().contains(*s) && !self.states.contains(s))
            .map(|(n, _)| format!(":{n}"))
            .collect();
        (!unused.is_empty()).then(|| format!("no rule uses {}", unused.join(" ")))
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

    /// The picked parts as an accordion, each item opening to the part's properties (see
    /// [`LssInspector::part_body`]). Writes each picked part's trigger line.
    fn properties(&self, inspection: &Inspection) -> AnyElement {
        let chrome = self.look.chrome();
        if let Ok(mut sections) = self.row_heads.sections.write() {
            for &index in &self.picked {
                let Some(part) = inspection.parts.get(index) else {
                    continue;
                };
                let rows: Vec<_> = part.rows.iter().filter(|row| self.keep(row, &self.rules(part, row))).collect();
                let set_by_state = rows.iter().filter(|row| by_state(row)).count();
                let title = if index == 0 {
                    format!("{} {{ … }}", self.control)
                } else {
                    format!("part {} {{ … }}", part.part.name())
                };
                sections.insert(
                    index,
                    SectionHead {
                        title: title.into(),
                        count: format!("{} properties", rows.len()).into(),
                        by_state: (set_by_state > 0).then(|| format!("{set_by_state} set by state").into()),
                        colors: self.row_colors(),
                    },
                );
            }
        }
        div()
            .id("lss-inspector-properties")
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .when_some(self.parts_accordion.clone().filter(|_| !self.picked.is_empty()), |d, accordion| {
                d.child(accordion)
            })
            .when(self.picked.is_empty(), |d| {
                d.child(div().p(px(16.)).text_color(chrome.muted_text).child("Pick one or more parts."))
            })
            .into_any_element()
    }

    /// One part's properties, as before the accordion: its doc, then (`Group: File`) its box model and its
    /// rows in file order, or (`Group: Kind`) a section per category with the box model in Layout.
    fn part_body(&self, index: usize) -> AnyElement {
        let chrome = self.look.chrome();
        let inspection = self.theme.inspect(&self.key, None);
        let Some(part) = inspection.parts.get(index) else {
            return div().into_any_element();
        };
        let mut rows: Vec<(&Row, Vec<(RuleInfo, RuleStatus)>)> = part
            .rows
            .iter()
            .map(|row| (row, self.rules(part, row)))
            .filter(|(row, rules)| self.keep(row, rules))
            .collect();
        rows.sort_by_key(|(row, _)| row.explanation.winner.as_ref().map_or(u32::MAX, |w| w.line));
        let diagram = || {
            let id = format!("style-inspector-box-{}", part.part.name());
            div().pl(px(ROW_INDENT)).pr(px(12.)).py(px(6.)).child(box_model::diagram(
                id.into(),
                &part.box_model,
                self.look.mode(),
            ))
        };
        // The column header sits inside each part's section, above its rows, so it lines up with them
        // whatever the accordion's content inset.
        let content: Vec<AnyElement> = match self.grouping {
            Grouping::File => std::iter::once(diagram().into_any_element())
                .chain(rows.iter().map(|(row, rules)| self.row(index, part, row, rules)))
                .collect(),
            Grouping::Kind => CATEGORIES
                .into_iter()
                .filter_map(|(category, title)| {
                    let in_category: Vec<_> = rows.iter().filter(|(row, _)| row.category == category).collect();
                    let box_diagram = (category == Category::Layout).then(|| diagram().into_any_element());
                    (box_diagram.is_some() || !in_category.is_empty())
                        .then(|| self.section(index, part, title.to_string(), in_category, box_diagram))
                })
                .collect(),
        };
        let name = part.part.name();
        div()
            .debug_selector(move || format!("style-inspector-part-{name}"))
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .px(px(12.))
                    .py(px(6.))
                    .text_color(chrome.muted_text)
                    .child(part.doc.clone().map_or_else(String::new, |d| format!("/// {d}"))),
            )
            .children(content)
            .into_any_element()
    }

    /// A category section of one part (`Group: Kind`).
    fn section(
        &self,
        index: usize,
        part: &PartInspection,
        title: String,
        rows: Vec<&(&Row, Vec<(RuleInfo, RuleStatus)>)>,
        diagram: Option<AnyElement>,
    ) -> AnyElement {
        let chrome = self.look.chrome();
        let set_by_state = rows.iter().filter(|(row, _)| by_state(row)).count();
        div()
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
                    .child(div().font_family(pane_type::MONO).text_color(chrome.title_text).child(title))
                    .child(div().text_color(chrome.muted_text).child(format!("{} properties", rows.len())))
                    .child(div().flex_1())
                    .when(set_by_state > 0, |d| {
                        d.child(div().text_color(self.look.accent(11)).child(format!("{set_by_state} set by state")))
                    }),
            )
            .children(diagram)
            .children(rows.into_iter().map(|(row, rules)| self.row(index, part, row, rules)))
            .into_any_element()
    }

    fn row_colors(&self) -> RowColors {
        let chrome = self.look.chrome();
        RowColors {
            text: chrome.title_text,
            muted: chrome.muted_text,
            accent: self.look.accent(11),
            accent_soft: self.look.accent(3),
            border: chrome.border,
        }
    }

    fn row(&self, index: usize, part: &PartInspection, row: &Row, rules: &[(RuleInfo, RuleStatus)]) -> AnyElement {
        let chrome = self.look.chrome();
        let e = &row.explanation;
        let id: RowId = (index, row.property);
        let expanded = self.expanded.contains(&id);
        let changed = by_state(row);
        let head = RowHead {
            value: display_value(e.value.as_ref(), row.kind).into(),
            swatch: match &e.value {
                Some(Value::Color(c)) => Some(Hsla::from(*c)),
                _ => None,
            },
            selector: e
                .winner
                .as_ref()
                .filter(|w| !w.states.is_empty() || !w.scope.is_empty())
                .map(|w| selector(w).into()),
            line: e.winner.as_ref().map_or_else(SharedString::default, |w| format!(":{}", w.line).into()),
            colors: self.row_colors(),
        };
        let columns: AnyElement = match self.row_buttons.get(&id) {
            Some(button) => {
                if let Ok(mut rows) = self.row_heads.rows.write() {
                    rows.insert(id, head);
                }
                button.clone().into_any_element()
            }
            None => row_columns(row.property, Some(&head)).into_any_element(),
        };
        let selector_id = format!("style-inspector-row-{}-{}", part.part.name(), row.property);
        div()
            .debug_selector(move || selector_id.clone())
            .w_full()
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(chrome.border.opacity(0.5))
            .when(expanded, |d| d.bg(chrome.panel_background))
            .child(
                div()
                    .relative()
                    .w_full()
                    // A column flex box, so the row's button stretches to the full width.
                    .flex()
                    .flex_col()
                    .pl(px(ROW_INDENT))
                    .pr(px(12.))
                    .when(changed, |d| d.child(self.dot()))
                    .child(columns),
            )
            .when(expanded, |d| d.child(self.details(row, rules)))
            .into_any_element()
    }

    /// The rules for a property and what its winning value read, side by side under its doc comment.
    fn details(&self, row: &Row, rules: &[(RuleInfo, RuleStatus)]) -> AnyElement {
        let chrome = self.look.chrome();
        let heading = |text: &'static str| {
            div().font_weight(pane_type::LABEL_WEIGHT).text_color(chrome.muted_text).child(text.to_uppercase())
        };
        let references = row.explanation.reference_lines();
        let no_references = references.is_empty();
        // The winning rule's expression, right under the row (the row shows only the value).
        let expression = row.explanation.winner.as_ref().map(|w| format!("= {}", w.expression));
        div()
            .pl(px(ROW_INDENT))
            .pr(px(12.))
            .pb(px(10.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .children(expression.map(|e| {
                let property = row.property;
                div()
                    .debug_selector(move || format!("lss-inspector-expression-{property}"))
                    .font_family(pane_type::MONO)
                    .text_color(chrome.muted_text)
                    .truncate()
                    .child(e)
            }))
            .children(row.doc.clone().map(|d| div().text_color(chrome.body_text).child(format!("/// {d}"))))
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(heading("Rules"))
                            .children(rules.iter().map(|(rule, status)| self.rule_line(rule, *status))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(heading("Reads"))
                            .children(references.into_iter().map(|line| {
                                div().font_family(pane_type::MONO).text_color(chrome.body_text).truncate().child(line)
                            }))
                            .when(no_references, |d| {
                                d.child(
                                    div().font_family(pane_type::MONO).text_color(chrome.muted_text).child("literal"),
                                )
                            })
                            .children(
                                row.explanation
                                    .notes()
                                    .map(|n| div().font_family(pane_type::MONO).text_color(chrome.muted_text).child(n)),
                            ),
                    ),
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
            .font_family(pane_type::MONO)
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
        let Some(source) = &self.source else {
            return self.missing("No style file found.");
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
        let marks = lines
            .into_iter()
            .map(|(line, statuses)| {
                let mark = if statuses.contains(&RuleStatus::Wins) {
                    LineMark { highlight: true, ..LineMark::default() }
                } else if statuses.contains(&RuleStatus::Beaten) {
                    LineMark { strike: true, ..LineMark::default() }
                } else {
                    LineMark { dim: true, ..LineMark::default() }
                };
                (line, mark)
            })
            .collect();
        let legend = vec![
            div().px(px(4.)).bg(self.look.accent(4)).child("wins for key").into_any_element(),
            div().line_through().child("beaten").into_any_element(),
            div().opacity(0.45).child("state not active").into_any_element(),
        ];
        self.file_view("lss-inspector-source", source, legend, &marks, LineMark::default())
    }

    /// The theme's palette file: lines the control reads in any state combination, highlighted where the
    /// current key uses them, each with its color.
    fn palette(&self) -> AnyElement {
        let Some(palette) = &self.palette else {
            return self.missing("No palette file found.");
        };
        // Every state combination, so a token read only under some states still counts as read.
        let base = self.states.iter().fold(self.key, |key, &state| key.set_state(state, false));
        let mut read: HashMap<u32, Option<Hsla>> = HashMap::new();
        for mask in 0u32..(1 << self.states.len()) {
            let key = self
                .states
                .iter()
                .enumerate()
                .fold(base, |key, (bit, &state)| key.set_state(state, mask & (1 << bit) != 0));
            collect_lines(&self.theme.inspect(&key, None), &palette.name, &mut read);
        }
        let mut used = HashMap::new();
        collect_lines(&self.theme.inspect(&self.key, None), &palette.name, &mut used);
        let marks = read
            .into_iter()
            .map(|(line, swatch)| {
                (line, LineMark { highlight: used.contains_key(&line), swatch, ..LineMark::default() })
            })
            .collect();
        let legend = vec![
            div().px(px(4.)).bg(self.look.accent(4)).child("used by key").into_any_element(),
            div().child(format!("read by {}", self.control)).into_any_element(),
            div().opacity(0.45).child("not read").into_any_element(),
        ];
        self.file_view("lss-inspector-palette", palette, legend, &marks, LineMark { dim: true, ..LineMark::default() })
    }

    fn missing(&self, text: &'static str) -> AnyElement {
        div().p(px(12.)).text_color(self.look.chrome().muted_text).child(text).into_any_element()
    }

    /// A style file with line numbers; `marks` says how to draw each marked line, `unmarked` the rest.
    fn file_view(
        &self,
        id: &'static str,
        file: &Source,
        legend: Vec<AnyElement>,
        marks: &HashMap<u32, LineMark>,
        unmarked: LineMark,
    ) -> AnyElement {
        let chrome = self.look.chrome();
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
                    .child(
                        div().font_family(pane_type::MONO).text_color(chrome.title_text).child(file.name.to_string()),
                    )
                    .child(div().flex_1())
                    .children(legend),
            )
            .child(
                div()
                    .id(id)
                    .debug_selector(move || id.into())
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .py(px(6.))
                    .font_family(pane_type::MONO)
                    .children(file.text.lines().enumerate().map(|(index, text)| {
                        let number = index as u32 + 1;
                        let mark = marks.get(&number).copied().unwrap_or(unmarked);
                        let color = if text.trim_start().starts_with("//") {
                            chrome.muted_text
                        } else {
                            chrome.body_text
                        };
                        div()
                            .flex()
                            .items_center()
                            .whitespace_nowrap()
                            .when(mark.highlight, |d| d.bg(accent_soft))
                            .child(
                                div()
                                    .w(px(30.))
                                    .flex_shrink_0()
                                    .pr(px(10.))
                                    .text_right()
                                    .text_color(chrome.muted_text)
                                    .child(number.to_string()),
                            )
                            .child(div().w(px(16.)).flex_shrink_0().children(mark.swatch.map(|c| self.swatch(c))))
                            .child(
                                div()
                                    .text_color(color)
                                    .when(mark.strike, |d| d.line_through())
                                    .when(mark.dim, |d| d.opacity(0.45))
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
        self.pick_parts(&[name], cx);
    }

    /// The Part picker's focus handle, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn part_picker_focus(&self, cx: &gpui::App) -> gpui::FocusHandle {
        use gpui::Focusable;
        self.part_tabs.read(cx).focus_handle(cx)
    }

    /// The picked parts, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn picked(&self) -> Vec<usize> {
        self.picked.clone()
    }

    /// Pick exactly these parts in the Part picker, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn pick_parts(&self, names: &[&str], cx: &mut gpui::App) {
        let ids: Vec<SharedString> = names.iter().map(|name| SharedString::from(name.to_string())).collect();
        self.part_tabs.update(cx, |group, cx| {
            group.set_selected_ids(ids.clone(), cx);
            cx.emit(ControlGroupEvent::Change {
                changed_id: SharedString::default(),
                selected: true,
                selected_ids: ids,
            });
        });
    }

    /// A rendered row's value, expression and rule selector, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn row_summary(&self, part: &str, property: &'static str) -> Option<(String, String, Option<String>)> {
        let index = self.part_index(part)?;
        let rows = self.row_heads.rows.read().ok()?;
        let head = rows.get(&(index, property))?;
        // The expression the row opens to; empty for a literal (it would repeat the value).
        let inspection = self.theme.inspect(&self.key, None);
        let row = inspection.parts.get(index)?.rows.iter().find(|r| r.property == property)?;
        let expression = match &row.explanation.winner {
            Some(_) if row.explanation.references.is_empty() => String::new(),
            Some(w) => w.expression.clone(),
            None => row.explanation.source_line(),
        };
        Some((head.value.to_string(), expression, head.selector.as_ref().map(|s| s.to_string())))
    }

    /// Show the Palette view, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn show_palette(&self, cx: &mut gpui::App) {
        choose_tab(&self.view_tabs, "palette", cx);
    }

    /// The bound mode, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn mode(&self) -> String {
        self.theme.mode().to_string()
    }

    /// The palette file's name and the text of each line the current key reads, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn palette_lines_used(&self) -> (String, Vec<String>) {
        let Some(palette) = &self.palette else {
            return (String::new(), Vec::new());
        };
        let mut used = HashMap::new();
        collect_lines(&self.theme.inspect(&self.key, None), &palette.name, &mut used);
        let lines: Vec<&str> = palette.text.lines().collect();
        let mut numbers: Vec<u32> = used.into_keys().collect();
        numbers.sort();
        let text = numbers
            .into_iter()
            .filter_map(|n| lines.get(n as usize - 1).map(|l| l.trim().to_string()))
            .collect();
        (palette.name.to_string(), text)
    }

    /// Show the Source view, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn show_source(&self, cx: &mut gpui::App) {
        choose_tab(&self.view_tabs, "source", cx);
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
            .debug_selector(|| "lss-inspector-tab-row".into())
            .flex_none()
            .h(px(TAB_ROW_HEIGHT))
            .flex()
            .items_end()
            .gap(px(8.))
            .px(px(12.))
            .border_b_1()
            .border_color(chrome.border)
            .child(self.view_tabs.clone())
            .child(div().flex_1())
            .children(self.unused_states_note().map(|note| div().pb(px(8.)).text_color(chrome.muted_text).child(note)))
            .child(div().pb(px(8.)).font_family(pane_type::MONO).text_color(chrome.muted_text).child(key_line(
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
                        .child(
                            div()
                                .debug_selector(|| "lss-inspector-attributes".into())
                                .flex()
                                .items_center()
                                .gap_2()
                                .child("Attributes")
                                .child(
                                    div()
                                        .text_color(chrome.muted_text)
                                        .child(attributes_line(&self.theme, self.control)),
                                ),
                        )
                        .child(div().flex().items_center().gap_2().child("Part").child(self.part_tabs.clone()))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(12.))
                                .child(
                                    div().flex().items_center().gap_2().child("Show").child(self.filter_tabs.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child("Group")
                                        .child(self.grouping_tabs.clone()),
                                ),
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
            View::Palette => div().flex_1().min_h_0().flex().flex_col().child(self.palette()),
        };
        div()
            .id("style-inspector")
            .debug_selector(|| "style-inspector".into())
            .size_full()
            .flex()
            .flex_col()
            .typography_style(pane_type::body(&self.look))
            .text_color(chrome.body_text)
            .child(toolbar)
            .child(body)
    }
}

/// Choose a tab as a person would, for tests.
#[cfg(all(test, feature = "test-support"))]
pub(super) fn choose_tab(tabs: &Entity<xgpui_luma::controls::tabs::Tabs>, id: &str, cx: &mut gpui::App) {
    let id = gpui::SharedString::from(id.to_string());
    tabs.update(cx, |tabs, cx| {
        tabs.set_active(id.clone(), cx);
        cx.emit(xgpui_luma::controls::tabs::TabsEvent::Change { tab_id: id, label: gpui::SharedString::default() });
    });
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
/// Every active state is shown, including ones no rule uses (e.g. `:focused`, set by the template).
pub(super) fn key_line(theme: &Theme, control: ControlId, key: &StyleKey) -> String {
    let config = theme.stylesheet().config();
    let states: String = config
        .state_names()
        .filter(|n| config.state(n).is_some_and(|s| key.active_states().contains(s)))
        .map(|n| format!(":{n}"))
        .collect();
    if theme.mode().is_empty() {
        format!("{control}{states}")
    } else {
        format!("{control}{states} · {}", theme.mode())
    }
}

/// A control's attribute scopes as text (e.g. `size sm · md`), or a note that the file has none.
pub(super) fn attributes_line(theme: &Theme, control: ControlId) -> String {
    let axes = theme.stylesheet().axes(control);
    let config = theme.stylesheet().config();
    if axes.attributes.is_empty() {
        return "none — the file has no scopes".to_string();
    }
    axes.attributes
        .iter()
        .map(|(attr, values)| {
            let values: Vec<&str> = values.iter().map(|v| v.name()).collect();
            format!("{} {}", config.attribute_name(*attr), values.join(" · "))
        })
        .collect::<Vec<_>>()
        .join("   ")
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

/// The categories `Group: Kind` makes tree nodes of, in order.
const CATEGORIES: [(Category, &str); 6] = [
    (Category::Colors, "Colors"),
    (Category::Layout, "Layout"),
    (Category::Typography, "Typography"),
    (Category::Elevation, "Elevation"),
    (Category::Motion, "Motion"),
    (Category::Other, "Other"),
];

/// A picked part's accordion trigger: its block title, property count and "n set by state".
fn section_trigger(head: Option<&SectionHead>) -> Div {
    let row = div().flex_1().min_w_0().flex().items_baseline().gap(px(8.)).whitespace_nowrap();
    let Some(head) = head else {
        return row;
    };
    let c = head.colors;
    row.child(div().flex_shrink_0().font_family(pane_type::MONO).text_color(c.text).child(head.title.clone()))
        .child(div().flex_shrink_0().text_color(c.muted).child(head.count.clone()))
        .child(div().flex_1())
        .children(head.by_state.clone().map(|text| div().flex_shrink_0().text_color(c.accent).child(text)))
}

/// An SDK button that draws `content` across the full width in the inspector's small type, with the
/// lab's hover paint: the property list's rows.
fn list_button<E: IntoElement + 'static>(
    id: String,
    look: &Arc<Look>,
    content: impl Fn() -> E + Send + Sync + 'static,
    on_click: impl Fn(&mut LssInspector, &mut Context<LssInspector>) + 'static,
    cx: &mut Context<LssInspector>,
) -> Entity<Button> {
    let button = xgpui_luma_look_radix::Button::new(id)
        .look(look.as_ref())
        .content_only()
        // No focus-ring inset, so the columns line up with the header.
        .compact()
        .content(move |_, _| content())
        .with_template_modifier(|root, _| root.w_full())
        .on_click(cx, on_click)
        .spawn(cx);
    // As the Workshop sidebar does: modify the SDK's inner button surface so the content starts at the
    // left edge in the inspector's small type, with the hover paint on the whole row.
    let (row_look, small) = (look.clone(), pane_type::body(look));
    let template = xgpui_luma_look_radix::button_template_for::<()>(
        look.as_ref(),
        xgpui_luma_look_radix::ButtonVariant::GhostQuiet,
        xgpui_luma_look_radix::Paint::gray(),
    )
    .with_modifier(move |row, model| {
        // A stretching column: the SDK wraps the content in a box sized to it, and this makes that box
        // (and so the row's columns) span the full width.
        row.w_full()
            .flex_col()
            .items_stretch()
            .justify_center()
            .text_align(gpui::TextAlign::Left)
            .px_0()
            .border_0()
            .typography_style(small)
            .when_some(row_look.interaction_colors().background(false, model.state), |row, color| row.bg(color))
    });
    button.update(cx, |button, cx| button.set_template(Arc::new(template), cx));
    button
}

/// One property row's columns: property, kind (when it fits), value, expression, rule.
/// A property row on one line: the name, then (right-aligned) its value, the winning rule's selector
/// when a state or scope set it, and the rule's line. The kind shows in the value (`36px`, a swatch),
/// and the expression opens under the row.
fn row_columns(property: &'static str, head: Option<&RowHead>) -> Div {
    let row = div()
        .w_full()
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .gap(px(COLUMN_GAP))
        .font_family(pane_type::MONO)
        .whitespace_nowrap();
    let name = div().debug_selector(move || format!("lss-inspector-name-{property}")).flex_1().min_w_0().truncate();
    let Some(head) = head else {
        return row.child(name.child(property));
    };
    let c = head.colors;
    row.child(name.text_color(c.text).child(property))
        .child(
            div()
                .debug_selector(move || format!("lss-inspector-value-{property}"))
                .flex_none()
                .max_w(px(VALUE_MAX_WIDTH))
                .flex()
                .items_center()
                .gap(px(6.))
                .text_color(c.text)
                .children(head.swatch.map(|color| {
                    div().size(px(11.)).flex_shrink_0().rounded(px(2.)).border_1().border_color(c.border).bg(color)
                }))
                .child(div().min_w_0().truncate().child(head.value.clone())),
        )
        .child(
            div()
                .debug_selector(move || format!("lss-inspector-rule-{property}"))
                .flex_none()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(6.))
                .children(head.selector.clone().map(|selector| {
                    div().px(px(5.)).rounded(px(4.)).bg(c.accent_soft).text_color(c.accent).child(selector)
                }))
                .child(div().min_w(px(LINE_WIDTH)).text_right().text_color(c.muted).child(head.line.clone())),
        )
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

/// The lines of `file` that an inspection's values read (through any chain of references), each with
/// the color defined there, if it is one.
fn collect_lines(inspection: &Inspection, file: &str, lines: &mut HashMap<u32, Option<Hsla>>) {
    fn walk(references: &[Reference], file: &str, lines: &mut HashMap<u32, Option<Hsla>>) {
        for reference in references {
            if let Some(at) = reference.defined_at.as_ref().filter(|at| *at.source == *file) {
                let color = match &reference.value {
                    Some(Value::Color(c)) => Some(Hsla::from(*c)),
                    _ => None,
                };
                lines.insert(at.line, color);
            }
            walk(&reference.children, file, lines);
        }
    }
    for row in inspection.parts.iter().flat_map(|p| &p.rows) {
        walk(&row.explanation.references, file, lines);
    }
}
