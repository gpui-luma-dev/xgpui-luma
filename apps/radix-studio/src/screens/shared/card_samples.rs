//! Local profile content shared by the home and style-guide Card samples.
use gpui::{Div, FontWeight, IntoElement, div, prelude::*, px};
use xgpui_luma_look_radix::{Avatar, AvatarSize, Card, CardSize, CardVariant, Look, Tone};

pub fn profile(look: &Look, variant: CardVariant, size: CardSize) -> Div {
    let (avatar_size, gap) = match size {
        CardSize::One => (AvatarSize::Three, 8.0),
        CardSize::Two => (AvatarSize::Four, 12.0),
        CardSize::Three => (AvatarSize::Five, 16.0),
    };
    Card::new(look)
        .variant(variant)
        .size(size)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(gap))
                .child(Avatar::new(look, "EA").image("assets/avatars/portrait.jpg").size(avatar_size))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .text_size(px(14.0))
                        .line_height(px(22.0))
                        .child(div().font_weight(FontWeight::MEDIUM).child("Emily Adams"))
                        .child(div().text_color(Tone::Gray.step(look, 11)).truncate().child("emily.adams@example.com")),
                ),
        )
        .into_element()
}

pub fn home(look: &Look) -> Div {
    div().flex().flex_col().gap(px(16.0)).w_full().children(
        [CardVariant::Classic, CardVariant::Surface].map(|variant| profile(look, variant, CardSize::Two).w_full()),
    )
}
