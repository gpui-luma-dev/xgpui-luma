//! Generic style inspector (issue #285): every part and property of one control, with where each
//! value came from. Driven only by `Inspection` from the style crate, so it works for any control
//! styled with `.lss` and needs no per-control code. Read-only for now.

use std::sync::Arc;

use gpui::{AnyElement, Context, FontWeight, Hsla, Render, Window, div, prelude::*, px};
use xgpui_luma::style::lss::{
    AttrId, AttrValue, Category, ControlId, Inspection, PartInspection, Row, StateId, StyleKey, Theme, Value,
    display_value,
};

use xgpui_luma::prelude::HasPresenter;

use super::box_model;
use crate::theme::{ColorVizLookExt, Look, MONO_FAMILY, TextSize, TypographyExt};

pub(super) struct StyleInspector {
    look: Arc<Look>,
    theme: Theme,
    control: ControlId,
    key: StyleKey,
    states: Vec<(StateId, xgpui_luma::controls::switch::Switch)>,
    /// One Radix toggle group per attribute, with the attribute's name as its label.
    attributes: Vec<(
        AttrId,
        String,
        xgpui_luma::controls::control_group::ControlGroup<xgpui_luma::controls::control_group::ControlGroupItem>,
    )>,
}

impl StyleInspector {
    pub(super) fn new(look: Arc<Look>, theme: Theme, control: ControlId, cx: &mut Context<Self>) -> Self {
        let sheet = theme.stylesheet().clone();
        let config = sheet.config().clone();
        let axes = sheet.axes(control);
        let states = axes
            .states
            .iter()
            .map(|&state| {
                let name = config.state_name(state).to_string();
                let switch = xgpui_luma_look_radix::Switch::new(format!("style-inspector-state-{name}"))
                    .look(look.as_ref())
                    .label(name.clone())
                    .checked(false)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        this.key = this.key.set_state(state, on);
                        cx.notify();
                    })
                    .spawn(cx);
                (state, switch)
            })
            .collect();
        let attributes = axes
            .attributes
            .iter()
            .map(|(attr, values)| {
                let attr = *attr;
                let name = config.attribute_name(attr).to_string();
                let mut options: Vec<String> = vec!["base".into()];
                options.extend(values.iter().map(|v| v.name().to_string()));
                let values = values.clone();
                let group = xgpui_luma_look_radix::control_group_builder(
                    look.as_ref(),
                    format!("style-inspector-attr-{name}"),
                    look.size(),
                )
                .horizontal()
                .single_required()
                .items(options.iter().enumerate().map(|(index, option)| {
                    xgpui_luma::controls::control_group::ControlGroupItem::new(index.to_string()).label(option.clone())
                }))
                .selected_ids(["0"])
                .spawn(cx);
                cx.subscribe(
                    &group,
                    move |this: &mut Self, _, event: &xgpui_luma::controls::control_group::ControlGroupEvent, cx| {
                        if let xgpui_luma::controls::control_group::ControlGroupEvent::Change { selected_ids, .. } =
                            event
                            && let Some(index) = selected_ids.first().and_then(|id| id.parse::<usize>().ok())
                        {
                            let value: Option<AttrValue> = index.checked_sub(1).and_then(|i| values.get(i).copied());
                            this.key = this.key.attr_opt(attr, value);
                            cx.notify();
                        }
                    },
                )
                .detach();
                (attr, name, group)
            })
            .collect();
        Self { look, theme, control, key: StyleKey::new(control), states, attributes }
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    /// Follow a live sample: set a state by name (e.g. the sample checkbox's `checked`). A state this
    /// control does not declare is ignored.
    pub(super) fn set_state(&mut self, name: &str, on: bool, cx: &mut Context<Self>) {
        let Some(state) = self.theme.stylesheet().config().state(name) else {
            return;
        };
        let Some((_, switch)) = self.states.iter().find(|(s, _)| *s == state) else {
            return;
        };
        if *switch.read(cx).data() != on {
            switch.update(cx, |switch, cx| switch.set_data(on, cx));
        }
        self.key = self.key.set_state(state, on);
        cx.notify();
    }

    /// Whether a state is on in the inspected key, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn state_on(&self, name: &str) -> bool {
        self.theme
            .stylesheet()
            .config()
            .state(name)
            .is_some_and(|state| self.key.active_states().contains(state))
    }

    fn part(&self, part: &PartInspection) -> AnyElement {
        let chrome = self.look.chrome();
        let categories = [
            (Category::Colors, "Colors"),
            (Category::Layout, "Layout"),
            (Category::Typography, "Typography"),
            (Category::Elevation, "Elevation"),
            (Category::Motion, "Motion"),
            (Category::Other, "Other"),
        ];
        let sections = categories.into_iter().filter_map(|(category, title)| {
            let rows: Vec<AnyElement> =
                part.rows.iter().filter(|r| r.category == category).map(|r| self.row(part, r)).collect();
            // Layout always opens with the box-model diagram, as in Studio's inspector.
            let diagram = (category == Category::Layout).then(|| {
                let id = format!("style-inspector-box-{}", part.part.name());
                div().pb(px(6.)).child(box_model::diagram(id.into(), &part.box_model, self.look.mode()))
            });
            (diagram.is_some() || !rows.is_empty()).then(|| {
                div()
                    .flex()
                    .flex_col()
                    .child(div().pt(px(6.)).pb(px(2.)).text_color(chrome.muted_text).child(title))
                    .children(diagram)
                    .children(rows)
            })
        });
        div()
            .id(gpui::SharedString::from(format!("style-inspector-part-{}", part.part.name())))
            .debug_selector({
                let name = part.part.name();
                move || format!("style-inspector-part-{name}")
            })
            .w_full()
            .flex()
            .flex_col()
            .gap(px(2.))
            .pt(px(10.))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_color(chrome.title_text)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(part.part.name().to_string()),
                    )
                    .children(part.doc.clone().map(|d| div().text_color(chrome.muted_text).child(d))),
            )
            .children(sections)
            .into_any_element()
    }

    fn row(&self, part: &PartInspection, row: &Row) -> AnyElement {
        let chrome = self.look.chrome();
        let e = &row.explanation;
        let swatch = match &e.value {
            Some(Value::Color(c)) => Some(Hsla::from(*c)),
            _ => None,
        };
        let notes = e.notes().map(|n| format!("  · {n}")).unwrap_or_default();
        let source = format!("{}{notes}", e.source_line());
        let references = e.reference_lines();
        let id = format!("style-inspector-row-{}-{}", part.part.name(), row.property);
        div()
            .debug_selector(move || id.clone())
            .w_full()
            .flex()
            .flex_col()
            .py(px(3.))
            .border_b_1()
            .border_color(chrome.border)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .child(
                        div().w(px(150.)).flex_shrink_0().text_color(chrome.title_text).child(row.property.to_string()),
                    )
                    .child(
                        div()
                            .w(px(150.))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .font_family(MONO_FAMILY)
                            .children(swatch.map(|c| {
                                div()
                                    .size(px(12.))
                                    .flex_shrink_0()
                                    .rounded(px(2.))
                                    .border_1()
                                    .border_color(chrome.border)
                                    .bg(c)
                            }))
                            .child(display_value(e.value.as_ref(), row.kind)),
                    )
                    .child(
                        div().flex_1().min_w_0().font_family(MONO_FAMILY).text_color(chrome.muted_text).child(source),
                    ),
            )
            .children(row.doc.clone().map(|d| div().pl(px(162.)).text_color(chrome.muted_text).child(d)))
            .children(
                references
                    .into_iter()
                    .map(|line| div().pl(px(162.)).font_family(MONO_FAMILY).text_color(chrome.muted_text).child(line)),
            )
            .into_any_element()
    }
}

impl Render for StyleInspector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let small = self.look.typography_scale(TextSize::Xs);
        let inspection: Inspection = self.theme.inspect(&self.key, None);
        let config = self.theme.stylesheet().config().clone();
        let active: Vec<&str> = self
            .states
            .iter()
            .filter(|(s, _)| self.key.active_states().contains(*s))
            .map(|(s, _)| config.state_name(*s))
            .collect();
        let key_line = if active.is_empty() {
            "base".to_string()
        } else {
            active.iter().map(|s| format!(":{s}")).collect()
        };
        div()
            .id("style-inspector")
            .debug_selector(|| "style-inspector".into())
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.))
            .typography_style(small)
            .text_color(chrome.body_text)
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_color(chrome.title_text)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("Style · {}", self.control)),
                    )
                    .children(inspection.doc.clone().map(|d| div().text_color(chrome.muted_text).child(d))),
            )
            .child(div().flex().flex_wrap().gap(px(10.)).children(self.states.iter().map(|(_, s)| s.clone())))
            .children(
                self.attributes.iter().map(|(_, name, group)| {
                    div().flex().items_center().gap_2().child(name.clone()).child(group.clone())
                }),
            )
            .child(div().font_family(MONO_FAMILY).text_color(chrome.muted_text).child(format!("key  {key_line}")))
            .children(inspection.parts.iter().map(|p| self.part(p)))
    }
}
