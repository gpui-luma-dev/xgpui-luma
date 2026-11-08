use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ControlIcon};
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use xgpui_luma::{declare_form, hstack, vstack};
use lucide_svg_static::Icon as LucideIcon;

use super::common::{avatar, AvatarSize, card, message_bubble};

declare_form! {
    pub struct ChatPanel {
        controls: {
            message_field: TextField = shadcn::TextField::new("chat-message").look(look.as_ref())
                .placeholder("Type your message…")
                .full_width(true),
            send_button: Entity<Button> = shadcn::Button::icon_button("chat-send", ControlIcon::Lucide(LucideIcon::ArrowUp)).look(look.as_ref()).primary()
                .size(size)
                .round(true),
            plus_button: Entity<Button> = shadcn::Button::icon_button("chat-add", ControlIcon::Lucide(LucideIcon::Plus)).look(look.as_ref()).secondary()
                .size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ShadcnSize,
        },
        fields: {}
    }
}

impl Render for ChatPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let incoming_bg = chrome.panel_background;
            let primary = self.look.mode_tokens().palette.primary;
            let outgoing_bg = primary.background;
            let outgoing_fg = primary.foreground;
            let plus_button = self.plus_button.clone();
            let message_field = self.message_field.clone();
            let send_button = self.send_button.clone();
            let name_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let email_style = self.look.typography_scale(ShadcnTextSize::Xs);

            card(
                "luma-studio-chat-card",
                &self.look,
                360.0,
                move |_, _| {
                    vstack! {
                        gap=12;
                        hstack! {
                            justify=between align=center;
                            hstack! {
                                gap=10 align=center;
                                avatar("SD", AvatarSize::Md),
                                vstack! {
                                    gap=2;
                                    div()
                                        .typography_style(name_style)
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(chrome.title_text)
                                        .child("Sofia Davis"),
                                    div()
                                        .typography_style(email_style)
                                        .text_color(chrome.muted_text)
                                        .child("m@example.com"),
                                },
                            },
                            plus_button.clone(),
                        },
                        vstack! {
                            gap=8;
                            message_bubble("Hi, how can I help you today?", false, incoming_bg, chrome.body_text),
                            message_bubble("Hey, I'm having trouble with my account.", true, outgoing_bg, outgoing_fg),
                            message_bubble("What seems to be the problem?", false, incoming_bg, chrome.body_text),
                            message_bubble("I can't log in.", true, outgoing_bg, outgoing_fg),
                        }
                        .min_h(px(180.0)),
                        hstack! {
                            gap=8 align=center;
                            div().flex_1().child(message_field.clone()),
                            send_button.clone(),
                        },
                    }
                    .into_any_element()
                },
                window,
                _cx,
            )
        })
    }
}
