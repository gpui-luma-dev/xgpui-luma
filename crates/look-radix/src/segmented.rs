//! Segmented presentation for the SDK radio group; selection stays SDK-owned.

use std::sync::Arc;
use gpui::{IntoElement, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroupItemHandlerExt, ControlGroupItemLike};
use xgpui_luma::controls::radio_group::RadioGroupTemplate;
use xgpui_luma::controls::segmented::{SegmentState, StyledSegmented, segment_border};
use xgpui_luma::style::PartRead;
use xgpui_luma::style::lss::PartId;
use crate::Look;

/// Horizontal radio-group template with a shared track and selected segment.
/// Item templates supply content only; colors and geometry belong to the look.
pub fn segmented_radio_template<T: ControlGroupItemLike + 'static>(look: &Look) -> RadioGroupTemplate<T> {
    let look = look.clone();
    Arc::new(move |model, handlers, window, cx| {
        let styled = styled_segmented(&look);
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let state = SegmentState {
                    selected: item.selected,
                    hovered: item.state.hovered,
                    pressed: item.state.pressed,
                    focus_visible: item.state.focus_visible,
                    disabled: !item.enabled,
                };
                let resolved = styled.resolve_segment(state);
                let segment = resolved.part(PartId::new("item"));
                let typography = segment.typography(14.0, 20.0);
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
                    .h(px(segment.number("height")))
                    .px(px(segment.number("padding_x")))
                    .rounded(px(segment.number("radius")))
                    .border_1()
                    .border_color(segment_border(&resolved, state))
                    .bg(segment.color("background"))
                    .text_size(px(typography.size))
                    .line_height(px(typography.line_height))
                    .text_color(segment.color("foreground"))
                    .when(item.enabled, |row| row.cursor_pointer())
                    .child(content)
                    .control_group_item_handlers(handlers)
            })
            .collect::<Vec<_>>();
        let track_style = styled.resolve_track();
        let track = track_style.root();
        div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .rounded(px(track.number("radius")))
            .bg(track.color("background"))
            .children(rows)
    })
}

/// The segmented choice from `.lss` (`style/segmented.lss`).
pub fn styled_segmented(look: &Look) -> StyledSegmented {
    StyledSegmented::new(xgpui_luma::style::StyleBinding::new("segmented", crate::textfield::live_source(look)))
}
