//! Basic Choice controls using SDK builders, SDK defaults, and retained local state.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{accordion, pager, radio_button, radio_group};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy, Debug)]
pub(super) enum ChoiceKind {
    Accordion,
    Pager,
    RadioGroup,
}
impl ChoiceKind {
    fn title(self) -> &'static str {
        match self {
            Self::Accordion => "Accordion",
            Self::Pager => "Pager",
            Self::RadioGroup => "Radio Group",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Accordion => "Expand or collapse a section.",
            Self::Pager => "Move between pages and choose a page size.",
            Self::RadioGroup => "Choose one option from a group.",
        }
    }
}

enum Control {
    Accordion(accordion::Accordion),
    Pager(pager::Pager),
    RadioGroup(radio_group::RadioGroup<radio_group::RadioGroupItem>),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Accordion(v) => v.clone().into(),
            Self::Pager(v) => v.clone().into(),
            Self::RadioGroup(v) => v.clone().into(),
        }
    }
}

pub(super) struct ChoiceSample {
    kind: ChoiceKind,
    look: Arc<Look>, // Surrounding chrome only; never passed to the SDK samples.
    control: Control,
    state: String,
    events: Entity<ControlEventStream>,
}
/// The sample's on/off state, for the page's style inspector.
impl ChoiceSample {
    pub(super) fn new(kind: ChoiceKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            ChoiceKind::Accordion => Control::Accordion(
                accordion::new("workshop-sdk-accordion")
                    .items([
                        accordion::AccordionItem::new(
                            "details",
                            accordion::AccordionTrigger::new("Details"),
                            accordion::AccordionContent::custom(|_, _| {
                                div().child("This content belongs to the first section.").into_any_element()
                            }),
                        )
                        .expanded(true),
                        accordion::AccordionItem::new(
                            "more",
                            accordion::AccordionTrigger::new("More information"),
                            accordion::AccordionContent::custom(|_, _| {
                                div().child("Each section retains its expanded state.").into_any_element()
                            }),
                        ),
                    ])
                    .collapsible(true)
                    .spawn(cx),
            ),
            ChoiceKind::Pager => Control::Pager(
                pager::new("workshop-sdk-pager")
                    .page_count(12)
                    .page_size(10)
                    .page_size_options([10, 25, 50])
                    .with_template_modifier(|root, _| root.debug_selector(|| "workshop-sdk-pager".into()))
                    .spawn(cx),
            ),
            ChoiceKind::RadioGroup => Control::RadioGroup(
                radio_group::new("workshop-sdk-radio-group")
                    .template(radio_group::radio_group_buttons_template(
                        radio_button::default_radio_button_template(),
                        radio_group::RadioGroupLayout::Vertical,
                    ))
                    .items([
                        radio_group::RadioGroupItem::new("small").label("Small"),
                        radio_group::RadioGroupItem::new("medium").label("Medium"),
                        radio_group::RadioGroupItem::new("large").label("Large"),
                    ])
                    .selected("medium")
                    .spawn(cx),
            ),
        };
        // Observe the real SDK event streams; native controls own their state transitions.
        match &control {
            Control::Accordion(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &accordion::AccordionEvent, cx| {
                    if let accordion::AccordionEvent::ExpandedChanged { item_id, expanded } = event {
                        this.state = format!("{item_id}: {}", if *expanded { "expanded" } else { "collapsed" });
                    }
                    this.record(event, cx);
                })
                .detach();
            }
            Control::Pager(v) => {
                cx.subscribe(v, |this: &mut Self, pager, event: &pager::PagerEvent, cx| {
                    let pager = pager.read(cx);
                    this.state = format!(
                        "Page {} of {} · {} per page",
                        pager.current_page() + 1,
                        pager.page_count(),
                        pager.page_size()
                    );
                    this.record(event, cx);
                })
                .detach();
            }
            Control::RadioGroup(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &radio_group::RadioGroupEvent, cx| {
                    if let radio_group::RadioGroupEvent::Change { value } = event {
                        this.state = format!("Selected: {}", value.as_deref().unwrap_or("none"));
                    }
                    this.record(event, cx);
                })
                .detach();
            }
        }
        let state = match kind {
            ChoiceKind::Accordion => "details: expanded",
            ChoiceKind::Pager => "Page 1 of 12 · 10 per page",
            ChoiceKind::RadioGroup => "Selected: medium",
        }
        .to_owned();
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-choice-events",
                "Interact with the sample to see its SDK events.",
            )
        });
        Self { kind, look, control, state, events }
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        let line = format!("{event:?}");
        self.events.update(cx, |events, cx| events.append_line(&line, cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for ChoiceSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        page(
            "workshop-choice-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-choice-example",
                    self.kind.title(),
                    self.kind.description(),
                    640.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(self.control.view()).child(
                        div()
                            .debug_selector(|| "workshop-choice-state".into())
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(self.state.clone()),
                    ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext, VisualTestContext};
    use crate::workshop::{Workshop, Sample};

    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<ChoiceSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| match &workshop.read(cx).samples[id] {
            Sample::Choice(v) => v.clone(),
            _ => panic!("missing Choice sample"),
        })
    }
    #[test]
    fn choice_accordion_expansion_and_radio_group_selection_follow_sdk_keyboard_behavior() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "accordion", cx);
        let accordion = cx.update(|_, cx| match &sample.read(cx).control {
            Control::Accordion(v) => v.clone(),
            _ => unreachable!(),
        });
        cx.update(|window, cx| accordion.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("home enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!accordion.read(cx).is_expanded(&"details".into())));
        cx.simulate_keystrokes("end enter");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(accordion.read(cx).is_expanded(&"more".into()));
            assert_eq!(sample.read(cx).state, "more: expanded");
        });
        let radio = select(&workshop, "radio-group", cx);
        let group = cx.update(|_, cx| match &radio.read(cx).control {
            Control::RadioGroup(v) => v.clone(),
            _ => unreachable!(),
        });
        cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(group.read(cx).selected_id(cx).as_deref(), Some("large"));
            assert_eq!(radio.read(cx).state, "Selected: large");
        });
        select(&workshop, "accordion", cx);
        cx.update(|_, cx| assert!(accordion.read(cx).is_expanded(&"more".into())));
    }

    #[test]
    fn choice_pager_click_updates_native_state_and_readout() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "pager", cx);
        let bounds = cx.debug_bounds("workshop-sdk-pager").unwrap();
        cx.simulate_click(gpui::point(bounds.right() - px(12.), bounds.center().y), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| {
            let v = sample.read(cx);
            let Control::Pager(pager) = &v.control else {
                unreachable!()
            };
            assert_eq!(pager.read(cx).current_page(), 11);
            assert_eq!(v.state, "Page 12 of 12 · 10 per page");
        });
        select(&workshop, "accordion", cx);
        select(&workshop, "pager", cx);
        cx.update(|_, cx| assert_eq!(sample.read(cx).state, "Page 12 of 12 · 10 per page"));
    }
}
