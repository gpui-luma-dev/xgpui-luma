use xgpui_luma::controls::accordion::AccordionEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionEvent, AccordionItem, AccordionTrigger};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma::{vstack};
use lucide_svg_static::Icon as LucideIcon;

use super::common::titled_card;

const ACCORDION_CARD_WIDTH: f32 = 380.0;

pub struct AccordionPanel {
    look: Arc<ShadcnLook>,
    accordion: Entity<AccordionControl>,
    last_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl AccordionPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let accordion = shadcn::Accordion::new("studio-accordion-single")
            .look(look.as_ref())
            .single()
            .item(demo_item("general", "General", LucideIcon::Settings, "General settings content."))
            .item(demo_item("billing", "Billing", LucideIcon::CreditCard, "Billing and payment details."))
            .item(demo_item("team", "Team", LucideIcon::Users, "Team member management.").expanded(true))
            .item(demo_item("legacy", "Legacy", LucideIcon::Archive, "Legacy configuration.").enabled(false))
            .spawn(cx);

        let subscriptions = Vec::new();
        accordion.clone().on_event(cx, |panel, event, cx| {
            panel.handle_event(event, cx);
        });

        Self { look, accordion, last_event: "None".into(), _subscriptions: subscriptions }
    }

    fn handle_event(&mut self, event: &AccordionEvent, cx: &mut Context<Self>) {
        self.last_event = match event {
            AccordionEvent::ExpandedChanged { item_id, expanded } => {
                format!("{} {}", item_id, if *expanded { "expanded" } else { "collapsed" }).into()
            }
            _ => self.last_event.clone(),
        };
        cx.notify();
    }
}

impl Render for AccordionPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let accordion = self.accordion.clone();
        let look = self.look.clone();
        let last_event = self.last_event.clone();
        let event_style = self.look.typography_scale(ShadcnTextSize::Sm);

        titled_card(
            "luma-studio-accordion-card",
            &self.look,
            ACCORDION_CARD_WIDTH,
            "Settings",
            "Single-expansion accordion with icons.",
            move |_, cx| {
                vstack! {
                    gap=12;
                    shadcn::Frame::new("studio-accordion-frame")
                        .look(&look)
                        .h_auto()
                        .bg(gpui::transparent_black())
                        .rounded(px(8.0))
                        .overflow_hidden()
                        .child(accordion.clone())
                        .render(cx),
                    div()
                        .typography_style(event_style)
                        .text_color(chrome.muted_text)
                        .child(format!("Last event: {}", last_event)),
                }
                .w_full()
                .overflow_hidden()
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn demo_item(id: &str, label: &str, icon: LucideIcon, body: &str) -> AccordionItem {
    let body = body.to_string();
    AccordionItem::new(
        id.to_string(),
        AccordionTrigger::new(label.to_string()).icon(icon),
        AccordionContent::custom(move |_, _| div().text_sm().child(body.clone()).into_any_element()),
    )
}
