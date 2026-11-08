use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::hstack;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
}

pub fn card(
    id: impl Into<SharedString>,
    look: &Arc<ShadcnLook>,
    _width: f32,
    content: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .w_full()
        .max_w_full()
        .child(shadcn::Card::new(id).look(look.as_ref()).child_render(content).render(window, cx))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub fn titled_card(
    id: impl Into<SharedString>,
    look: &Arc<ShadcnLook>,
    _width: f32,
    title: &'static str,
    subtitle: &'static str,
    content: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .w_full()
        .max_w_full()
        .child(
            shadcn::Card::new(id)
                .look(look.as_ref())
                .title(title)
                .description(subtitle)
                .child_render(content)
                .render(window, cx),
        )
        .into_any_element()
}

pub fn or_divider(label: &'static str, border: Hsla, text: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().flex_1().h(px(1.0)).bg(border))
        .child(div().text_xs().line_height(px(12.0)).text_color(text).child(label))
        .child(div().flex_1().h(px(1.0)).bg(border))
}

pub fn avatar(initials: &'static str, size: AvatarSize) -> impl IntoElement {
    let size_px = match size {
        AvatarSize::Sm => 32.0,
        AvatarSize::Md => 36.0,
    };

    div()
        .size(px(size_px))
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::SEMIBOLD)
        .bg_cn(ShadcnToken::Muted)
        .text_cn(ShadcnToken::MutedForeground)
        .font_cn(ShadcnFont::Sans)
        .rounded_full()
        .text_size(px(size_px * 0.38))
        .child(initials)
}

pub fn message_bubble(text: &'static str, align_end: bool, bg: Hsla, fg: Hsla) -> impl IntoElement {
    let bubble = div()
        .max_w(px(220.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(10.0))
        .bg(bg)
        .text_sm()
        .line_height(px(16.0))
        .text_color(fg)
        .child(text);

    if align_end {
        hstack! { justify=end; bubble }
    } else {
        div().child(bubble)
    }
}
