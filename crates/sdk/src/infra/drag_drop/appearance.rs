//! Theme-owned compact preview and valid/invalid insertion feedback.
use gpui::{Context, Hsla, IntoElement, Pixels, Point, Render, SharedString, Window, div, prelude::*, px};
use crate::theme::{LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct DragDropLook {
    pub geometry: crate::theme::stylesheet::SharedGeometry,
    pub background: Hsla,
    pub foreground: Hsla,
    pub valid_marker: Hsla,
    pub invalid_marker: Hsla,
    pub radius: f32,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}
impl DragDropLook {
    pub fn from_tokens(tokens: &ThemeTokens) -> Self {
        let palette = tokens.stylesheet.recipes.palette("drag_drop", &tokens.palette);
        Self {
            geometry: tokens.stylesheet.shared_geometry(),
            background: palette.surface.floating.background,
            foreground: palette.surface.floating.foreground,
            valid_marker: palette.focus.ring,
            invalid_marker: palette.form.input.invalid_border,
            radius: tokens.metrics.radius(),
            typography: tokens.typography.text.label,
            font_family: tokens.typography.font.sans.family.clone().into(),
        }
    }
}
pub(crate) struct DragPreview {
    pub label: SharedString,
    pub offset: Point<Pixels>,
    pub look: DragDropLook,
    pub selector: &'static str,
}
impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selector = self.selector;
        div().relative().child(
            div()
                .debug_selector(move || selector.to_owned())
                .relative()
                .left(self.offset.x + px(self.look.geometry.drag_offset))
                .top(self.offset.y + px(self.look.geometry.drag_offset))
                .w(px(self.look.geometry.drag_preview_width))
                .px(px(self.look.geometry.drag_padding_x))
                .py(px(self.look.geometry.drag_padding_y))
                .rounded(px(self.look.radius))
                .bg(self.look.background)
                .text_color(self.look.foreground)
                .border(px(self.look.geometry.drag_border_width))
                .border_color(self.look.valid_marker)
                .font_family(self.look.font_family.clone())
                .text_size(px(self.look.typography.size))
                .line_height(px(self.look.typography.line_height))
                .font_weight(self.look.typography.weight)
                .truncate()
                .child(self.label.clone()),
        )
    }
}

/// Match the Table's admission-aware marker and native drag cursor feedback.
pub(crate) fn bind_feedback<S: Clone + 'static, K: Clone + 'static>(
    surface: gpui::Stateful<gpui::Div>,
    look: DragDropLook,
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
        .drag_over::<super::KeyedDrag<S, K>>(move |style, drag, window, cx| {
            if !drag.accepts(scope) {
                return style;
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
            style.border_color(if accepted {
                look.valid_marker
            } else {
                look.invalid_marker
            })
        })
}
