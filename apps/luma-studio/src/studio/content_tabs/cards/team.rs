use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextRole, ShadcnTextSize};
use xgpui_luma::{declare_form, hstack, vstack};

use super::common::{avatar, AvatarSize, card};

const TEAM_CARD_WIDTH: f32 = 380.0;

declare_form! {
    pub struct TeamPanel {
        controls: {
            sofia_menu: Entity<PopupMenu> = shadcn::PopupMenu::new("team-sofia").look(look.as_ref())
                .label("Owner")
                .size(shadcn::ShadcnSize::Sm)
                .without_elevation()
                .items(role_menu_items())
                => PopupMenuEvent |this, event, cx| {
                    let PopupMenuEvent::Select { label, .. } = event else {
                        return;
                    };
                    this.sofia_menu.update(cx, |menu, cx| menu.set_label(label.clone(), cx));
                },
            jackson_menu: Entity<PopupMenu> = shadcn::PopupMenu::new("team-jackson").look(look.as_ref())
                .label("Developer")
                .size(shadcn::ShadcnSize::Sm)
                .without_elevation()
                .items(role_menu_items())
                => PopupMenuEvent |this, event, cx| {
                    let PopupMenuEvent::Select { label, .. } = event else {
                        return;
                    };
                    this.jackson_menu.update(cx, |menu, cx| menu.set_label(label.clone(), cx));
                },
            isabella_menu: Entity<PopupMenu> = shadcn::PopupMenu::new("team-isabella").look(look.as_ref())
                .label("Billing")
                .size(shadcn::ShadcnSize::Sm)
                .without_elevation()
                .items(role_menu_items())
                => PopupMenuEvent |this, event, cx| {
                    let PopupMenuEvent::Select { label, .. } = event else {
                        return;
                    };
                    this.isabella_menu.update(cx, |menu, cx| menu.set_label(label.clone(), cx));
                },
        },
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {}
    }
}

impl Render for TeamPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let title_style = self.look.typography_role(ShadcnTextRole::H4);
            let body_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let caption_style = self.look.typography_scale(ShadcnTextSize::Xs);
            let sofia_menu = self.sofia_menu.clone();
            let jackson_menu = self.jackson_menu.clone();
            let isabella_menu = self.isabella_menu.clone();

            card(
                "luma-studio-team-card",
                &self.look,
                TEAM_CARD_WIDTH,
                move |_, _| {
                    vstack! {
                        gap=12;
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div().typography_style(title_style).text_color(chrome.title_text).child("Team Members"),
                            )
                            .child(
                                div().typography_style(body_style).text_color(chrome.muted_text).child("Invite your team members to collaborate."),
                            ),
                        member_row("SD", "Sofia Davis", "m@example.com", &sofia_menu, chrome, body_style, caption_style),
                        member_row("JL", "Jackson Lee", "m@example.com", &jackson_menu, chrome, body_style, caption_style),
                        member_row("IN", "Isabella Nguyen", "m@example.com", &isabella_menu, chrome, body_style, caption_style),
                    }
                    .w_full()
                    .overflow_hidden()
                    .into_any_element()
                },
                window,
                _cx,
            )
        })
    }
}

fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
    menu: &Entity<PopupMenu>,
    chrome: xgpui_luma::theme::LumaChrome,
    body_style: xgpui_luma::theme::LumaTextStyle,
    caption_style: xgpui_luma::theme::LumaTextStyle,
) -> impl IntoElement {
    hstack! {
        gap=10 align=center;
        avatar(initials, AvatarSize::Sm),
        vstack! {
            gap=2;
            div()
                .typography_style(body_style)
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .overflow_hidden()
                .child(name),
            div()
                .typography_style(caption_style)
                .text_color(chrome.muted_text)
                .overflow_hidden()
                .child(email),
        }
        .flex_1()
        .min_w_0()
        .overflow_hidden(),
        div().flex_none().child(menu.clone()),
    }
    .py(px(6.0))
    .overflow_hidden()
}

fn role_menu_items() -> Vec<MenuItem> {
    vec![MenuItem::new("owner").label("Owner"), MenuItem::new("billing").label("Billing")]
}
