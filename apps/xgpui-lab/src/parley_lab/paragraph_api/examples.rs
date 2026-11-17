//! Remaining xgpui text-lab bidi examples (#332). Only the specimen text uses
//! raw GPUI elements; interactive targets and tooltip lifecycle belong to Luma.
use gpui::{Context, Div, Entity, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::{
    controls::{button::Button, tooltip::Tooltip},
    prelude::HasPresenter,
    infra::attachments::TooltipEntityExt,
};
use xgpui_luma_look_radix::Look;
use xgpui_parley::typeset;
use crate::screen::{TextRole, text};

pub(super) const WRAPPED: &str = "שורה ראשונה של טקסט עברי ארוך שחייב לעבור לשורה הבאה כדי להיכנס לתיבה";
pub(super) const WRAP_WIDTH: gpui::Pixels = px(220.0);
const LONG: [&str; 2] = [
    "כותרת ארוכה מאוד שצריכה להיחתך בסוף השורה",
    "A long Latin title that has to be truncated at the end",
];

pub(super) fn truncation(look: &Look) -> Div {
    div().flex().flex_col().gap_2()
        .child(text(look, TextRole::Caption, "Hebrew keeps its first words with the ellipsis on the left; Latin keeps its start with the ellipsis on the right."))
        .children(LONG.into_iter().enumerate().map(|(index, value)| {
            div().id(SharedString::from(format!("rtl-truncation-{index}")))
                .role(gpui::Role::Label).aria_label(value)
                .w(px(260.0)).font(super::font()).text_size(super::FONT_SIZE)
                .truncate().child(value)
        }))
}

/// A shared specimen renderer for both the inline comparison and Luma tooltips.
fn specimen(typeset_text: bool) -> gpui::AnyElement {
    if typeset_text {
        typeset(WRAPPED).into_any_element()
    } else {
        WRAPPED.into_any_element()
    }
}

pub(super) fn tooltip_buttons<M: 'static>(look: &Look, cx: &mut Context<M>) -> Vec<Entity<Button>> {
    [false, true]
        .into_iter()
        .map(|typeset_text| {
            let kind = if typeset_text { "typeset" } else { "plain" };
            xgpui_luma_look_radix::Button::new(format!("rtl-tooltip-{kind}"))
                .look(look)
                .outline()
                .label(format!("{kind} RTL tooltip"))
                .spawn(cx)
                .tooltip(
                    Tooltip::new(WRAPPED).template(move |model| {
                        div()
                            .id(SharedString::from(format!("rtl-tooltip-content-{kind}")))
                            .role(gpui::Role::Label)
                            .aria_label(WRAPPED)
                            .w(WRAP_WIDTH)
                            .p_2()
                            .rounded_md()
                            .bg(model.background)
                            .text_color(model.foreground)
                            .font(super::font())
                            .text_size(px(15.0))
                            .child(specimen(typeset_text))
                            .into_any_element()
                    }),
                    cx,
                )
        })
        .collect()
}

fn comparison(look: &Look, label: &str, id: &'static str, clamp: bool) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap_3()
        .child(div().w(px(140.0)).child(text(look, TextRole::Label, label.to_owned())))
        .children([false, true].into_iter().map(|typeset_text| {
            let kind = if typeset_text { "typeset" } else { "plain" };
            div()
                .id(SharedString::from(format!("rtl-{id}-{kind}")))
                .role(gpui::Role::Label)
                .aria_label(format!("{label}, {kind}: {WRAPPED}"))
                .debug_selector(move || format!("rtl-{id}-{kind}"))
                .w(WRAP_WIDTH)
                .flex_none()
                .p_1()
                .font(super::font())
                .text_size(px(15.0))
                .when(clamp, |frame| frame.line_clamp(2).text_ellipsis())
                .child(specimen(typeset_text))
        }))
}

pub(super) fn wrapped(look: &Look, active: bool, buttons: &[Entity<Button>]) -> Div {
    div().flex().flex_col().gap_3()
        .child(text(look, TextRole::Caption, if active {
            "Plain GPUI wraps in painting order; typeset() wraps in logical order with the first words at the top right. Compare wrapping, two-line clamping and tooltips (#332)."
        } else {
            "No Parley typeset engine is installed: both columns use plain GPUI text and wrap in painting order. Run with --text-system=parley to compare logical wrapping."
        }))
        .child(comparison(look, "Wrapped", "wrap", false))
        .child(comparison(look, "Two-line clamp", "clamp", true))
        .child(div().flex().flex_wrap().gap_3().children(buttons.iter().cloned()))
}
