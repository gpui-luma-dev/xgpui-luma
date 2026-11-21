//! SDK display elements drawn from `.lss`: the badge and the card.
use std::sync::Arc;
use gpui::{Context, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{badge::Badge, card::Card};
use crate::theme::Look;
use super::sample_layout::{card, page};

pub(super) struct DisplaySample {
    look: Arc<Look>,
}

impl DisplaySample {
    pub(super) fn new(look: Arc<Look>, _cx: &mut Context<Self>) -> Self {
        Self { look }
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for DisplaySample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        page(
            "workshop-display-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-badge-example",
                    "Badge",
                    "SDK Badge: a short non-interactive label drawn from badge.lss.",
                    640.,
                    div()
                        .debug_selector(|| "workshop-sdk-badges".into())
                        .flex()
                        .gap(px(8.))
                        .child(Badge::new("New"))
                        .child(Badge::new("Beta"))
                        .child(Badge::new("12")),
                ))
                .child(card(
                    &self.look,
                    "workshop-card-example",
                    "Card",
                    "SDK Card: a passive surface with a title, a description and body content, drawn from card.lss.",
                    640.,
                    div().w(px(360.)).debug_selector(|| "workshop-sdk-card".into()).child(
                        Card::new()
                            .title("Project")
                            .description("Updated two hours ago")
                            .child(div().child("Cards group related content.")),
                    ),
                )),
        )
    }
}
