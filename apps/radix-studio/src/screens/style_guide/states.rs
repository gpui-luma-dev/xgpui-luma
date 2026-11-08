//! Interaction-state columns shared by the style guide matrices.

use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use xgpui_luma::theme::InteractionState;

use crate::assets::{icon_named, react_icon};

const HEADER_ICON_SIZE: f32 = 15.0;

#[derive(Clone, Copy)]
pub struct StateSample {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub state: InteractionState,
}

pub fn samples() -> [StateSample; 5] {
    [
        StateSample { id: "default", label: "Default", icon: "home", state: InteractionState::default() },
        StateSample {
            id: "hover",
            label: "Hover",
            icon: "cursor-arrow",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        StateSample {
            id: "focused",
            label: "Focused",
            icon: "border-dashed",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        StateSample {
            id: "pressed",
            label: "Pressed",
            icon: "arrow-down",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        StateSample {
            id: "disabled",
            label: "Disabled",
            icon: "circle-backslash",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ]
}

pub fn header_cell(sample: &StateSample, muted: Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .children(icon_named(sample.icon).map(|icon| react_icon(icon, muted, HEADER_ICON_SIZE)))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(sample.label),
        )
        .into_any_element()
}
