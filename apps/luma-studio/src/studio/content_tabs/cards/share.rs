use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*};
use xgpui_luma::controls::button::Button;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::selector::{Selector, SelectorItem};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::titled_card;

declare_form! {
    pub struct SharePanel {
        controls: {
            link_selector: Entity<Selector> = shadcn::Selector::new("share-link").look(look.as_ref())
                .label("Anyone with the link")
                .items(link_items()),
            copy_button: Entity<Button> = shadcn::Button::new("share-copy").look(look.as_ref()).outline().label("Copy Link").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: shadcn::ShadcnSize,
        },
        fields: {}
    }
}

impl Render for SharePanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let link_selector = self.link_selector.clone();
        let copy_button = self.copy_button.clone();

        titled_card(
            "luma-studio-share-card",
            &self.look,
            380.0,
            "Share this document",
            "Anyone with the link can view this document.",
            move |_, _| {
                vstack! {
                    gap=12;
                    form_field!("Link", chrome; hstack! {
                        gap=8;
                        div().flex_1().min_w_0().child(link_selector.clone()),
                        copy_button.clone(),
                    }),
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn link_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("anyone").label("Anyone with the link"),
        SelectorItem::new("restricted").label("Restricted"),
    ]
}
