//! Titled section block shared by the documentation-style screens.

use gpui::{AnyElement, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::vstack;

pub fn section(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    fg: Hsla,
    muted: Hsla,
    border: Hsla,
    content: AnyElement,
) -> AnyElement {
    vstack! {
        gap=10;
        vstack! {
            gap=6;
            div().text_lg().font_weight(FontWeight::SEMIBOLD).text_color(fg).child(title.into()),
            div().text_xs().text_color(muted).child(description.into()),
            div().w_full().h(px(1.0)).bg(border),
        }
        .w_full(),
        div().w_full().flex().justify_center().child(content),
    }
    .w_full()
    .into_any_element()
}
