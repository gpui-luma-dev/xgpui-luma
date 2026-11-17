use crate::infra::locale::{LayoutDirection, LogicalStyled};
use std::sync::Arc;
use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use super::TooltipRenderModel;

/// Presentation-only renderer; lifecycle and positioning are managed by the SDK.
pub type TooltipTemplate = Arc<dyn Fn(&TooltipRenderModel) -> AnyElement + Send + Sync>;

pub fn bubble(model: &TooltipRenderModel) -> AnyElement {
    div()
        .block_mouse_except_scroll()
        .flex()
        .flex_row_logical(model.direction)
        .when(model.direction == LayoutDirection::Rtl, |root| root.text_right())
        .items_center()
        .gap(px(model.shortcut_gap))
        .max_w(px(model.max_width))
        .px(px(model.padding))
        .py(px(model.padding_y))
        .rounded(px(model.radius))
        .shadow(model.shadow.clone())
        .bg(model.background)
        .text_color(model.foreground)
        .text_size(px(model.text_size))
        .line_height(px(model.line_height))
        .child(div().min_w_0().child(model.text.clone()))
        .when_some(model.shortcut.clone(), |root, shortcut| {
            root.child(
                div()
                    .flex_shrink_0()
                    .opacity(model.shortcut_opacity)
                    .text_size(px(model.shortcut_font_size))
                    .child(shortcut),
            )
        })
        .into_any_element()
}
