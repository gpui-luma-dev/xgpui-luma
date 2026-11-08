//! Segmented presentation for the SDK radio group; selection stays SDK-owned.

use std::sync::Arc;
use gpui::{IntoElement, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroupItemHandlerExt, ControlGroupItemLike};
use xgpui_luma::controls::radio_group::RadioGroupTemplate;
use crate::{Look, ScaleFamily};

/// Horizontal radio-group template with a shared track and selected segment.
/// Item templates supply content only; colors and geometry belong to the look.
pub fn segmented_radio_template<T: ControlGroupItemLike + 'static>(look: &Look) -> RadioGroupTemplate<T> {
    let look = look.clone();
    Arc::new(move |model, handlers, window, cx| {
        let geometry = resolve_geometry(&look);
        let gray = |step| look.resolve_step(ScaleFamily::Gray, step).hsla();
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let fill = if item.state.pressed {
                    6
                } else if item.selected {
                    5
                } else if item.state.hovered {
                    4
                } else {
                    3
                };
                let border = if item.state.focus_visible {
                    look.resolve_step(ScaleFamily::Color, 8).hsla()
                } else {
                    gray(if item.selected { 6 } else { fill })
                };
                let content = match model.item_template {
                    Some(template) => template(item, window, cx),
                    None => div().child(item.item.label().clone()).into_any_element(),
                };
                div()
                    .id(format!("{}-{}", model.id, item.item.id()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .h(px(geometry.height))
                    .px(px(geometry.padding_x))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(border)
                    .bg(gray(fill))
                    .text_size(px(geometry.font_size))
                    .line_height(px(geometry.line_height))
                    .text_color(gray(if item.enabled { 12 } else { 8 }))
                    .when(item.enabled, |row| row.cursor_pointer())
                    .child(content)
                    .control_group_item_handlers(handlers)
            })
            .collect::<Vec<_>>();
        div().id(model.id.clone()).flex().items_center().rounded(px(5.0)).bg(gray(3)).children(rows)
    })
}

pub(crate) fn resolve_geometry(look: &Look) -> xgpui_luma::theme::stylesheet::SegmentedGeometry {
    let control = look.metrics().control.md;
    let typography = xgpui_luma::theme::ThemeTokens::default().typography.text.label;
    look.common_stylesheet().segmented.resolve_geometry(
        "2",
        xgpui_luma::theme::stylesheet::SegmentedGeometry {
            height: control.height,
            padding_x: control.padding_x,
            font_size: typography.size,
            line_height: typography.line_height,
        },
    )
}

/// Native common geometry owned by this control.
pub(crate) fn native_segmented_stylesheet() -> xgpui_luma::theme::stylesheet::SegmentedStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    SegmentedStylesheet {
        sizes: xgpui_luma::theme::ControlSizes::from([(
            "2".into(),
            SegmentedGeometryStylesheet {
                height: Some(32.0),
                padding_x: Some(12.0),
                font_size: Some(14.0),
                line_height: Some(20.0),
            },
        )]),
        ..Default::default()
    }
}
