//! Theme-owned compact preview and valid/invalid insertion feedback, read from `drag_preview.lss`.
//!
//! The preview and the markers draw from the control's resolved style (#433); [`DragDropLook`] is built from it for
//! code that still reads one.
use std::sync::Arc;

use crate::style::{PartRead, StyleBinding};
use crate::theme::LumaTextStyle;
use gpui::{Context, Hsla, IntoElement, Pixels, Point, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma_style::Resolved;

/// The drag preview's style for `binding` (the `drag_preview` control) (#433): the preview is its root, and
/// `valid_marker` and `invalid_marker` are the insertion feedback's colors.
pub fn resolve_drag_preview(binding: &StyleBinding) -> Arc<Resolved> {
    binding.resolve(&[], 1.0)
}

/// The SDK's drag preview binding: `drag_preview.lss` alone.
pub fn sdk_drag_preview() -> StyleBinding {
    StyleBinding::sdk("drag_preview")
}

/// The color of the insertion marker for a drop that is `valid` or not.
pub(crate) fn drag_marker_color(style: &Resolved, valid: bool) -> Hsla {
    style.root().color(if valid { "valid_marker" } else { "invalid_marker" })
}

#[cfg(test)]
mod styled_tests {}

pub(crate) struct DragPreview {
    pub label: SharedString,
    pub offset: Point<Pixels>,
    pub style: Arc<Resolved>,
    pub selector: &'static str,
}
impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selector = self.selector;
        let geometry = crate::theme::stylesheet::SharedGeometry::sdk();
        let root = self.style.root();
        let typography = root.typography(12.5, 18.0);
        div().relative().child(
            div()
                .debug_selector(move || selector.to_owned())
                .relative()
                .left(self.offset.x + px(geometry.drag_offset))
                .top(self.offset.y + px(geometry.drag_offset))
                .w(px(geometry.drag_preview_width))
                .px(px(geometry.drag_padding_x))
                .py(px(geometry.drag_padding_y))
                .rounded(px(root.number("radius")))
                .bg(root.color("background"))
                .text_color(root.color("foreground"))
                .border(px(geometry.drag_border_width))
                .border_color(root.color("valid_marker"))
                .font_family(root.font_family_name())
                .text_size(px(typography.size))
                .line_height(px(typography.line_height))
                .font_weight(typography.weight)
                .truncate()
                .child(self.label.clone()),
        )
    }
}

/// Match the Table's admission-aware marker and native drag cursor feedback.
pub(crate) fn bind_feedback<S: Clone + 'static, K: Clone + 'static>(
    surface: gpui::Stateful<gpui::Div>,
    style: Arc<Resolved>,
    scope: gpui::EntityId,
    valid: impl Fn(&super::KeyedDrag<S, K>, &gpui::App) -> bool + 'static,
) -> gpui::Stateful<gpui::Div> {
    let valid = std::rc::Rc::new(valid);
    let pointer_valid = valid.clone();
    surface
        .border_color(gpui::transparent_black())
        .on_drag_move(move |event: &gpui::DragMoveEvent<super::KeyedDrag<S, K>>, window, cx| {
            if event.bounds.contains(&event.event.position) {
                let accepted = pointer_valid(event.drag(cx), cx);
                cx.set_active_drag_cursor_style(
                    if accepted {
                        gpui::CursorStyle::ClosedHand
                    } else {
                        gpui::CursorStyle::OperationNotAllowed
                    },
                    window,
                );
            }
        })
        .drag_over::<super::KeyedDrag<S, K>>(move |surface_style, drag, window, cx| {
            if !drag.accepts(scope) {
                return surface_style;
            }
            let accepted = valid(drag, cx);
            let cursor = if accepted {
                gpui::CursorStyle::ClosedHand
            } else {
                gpui::CursorStyle::OperationNotAllowed
            };
            let pointer = window.mouse_position();
            let session = drag.clone();
            window.on_next_frame(move |window, cx| {
                if session.accepts(scope)
                    && window.mouse_position() == pointer
                    && cx.has_active_drag()
                    && cx.active_drag_cursor_style() != Some(cursor)
                {
                    cx.set_active_drag_cursor_style(cursor, window);
                }
            });
            surface_style.border_color(drag_marker_color(&style, accepted))
        })
}

/// The drag preview a control draws from its own styles (#433): the SDK's `drag_preview` with the control's
/// colors, corner radius and text set over it.
pub fn drag_preview_over(
    background: Hsla,
    foreground: Hsla,
    valid_marker: Hsla,
    invalid_marker: Hsla,
    radius: f32,
    typography: LumaTextStyle,
    font_family: Option<SharedString>,
) -> Arc<Resolved> {
    use crate::style::lss::{Overrides, PartId, Prop};
    use crate::style::lss_color;
    let root = PartId::ROOT;
    let mut overrides = Overrides::new()
        .set(root, Prop::<xgpui_luma_style::Color>::new("background"), lss_color(background))
        .set(root, Prop::<xgpui_luma_style::Color>::new("foreground"), lss_color(foreground))
        .set(root, Prop::<xgpui_luma_style::Color>::new("valid_marker"), lss_color(valid_marker))
        .set(root, Prop::<xgpui_luma_style::Color>::new("invalid_marker"), lss_color(invalid_marker))
        .set(root, Prop::<f32>::new("radius"), radius)
        .set(root, Prop::<f32>::new("font_size"), typography.size)
        .set(root, Prop::<f32>::new("line_height"), typography.line_height)
        .set(root, Prop::<f32>::new("font_weight"), typography.weight.0);
    if let Some(family) = font_family {
        overrides = overrides.set(root, Prop::<Arc<str>>::new("font_family"), Arc::<str>::from(family.as_ref()));
    }
    resolve_drag_preview(&sdk_drag_preview().with_overrides(&overrides))
}
