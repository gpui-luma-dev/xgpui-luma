//! Listbox chrome read from `.lss` (issue #285): `listbox.lss`. Looks paint the surface from
//! [`StyledListBox::surface`] and rows with [`StyledListBox::paint_row`]; item content stays the host's.

use crate::theme::InteractionState;
use crate::style::StyleStates;
use crate::style::PartRead;
use gpui::{Div, ElementId, Stateful, div, prelude::*, px};
use xgpui_luma_style::{PartId, Resolved};

use super::ListBoxItemState;
use crate::style::StyleBinding;

/// The listbox's `.lss` values for one binding (the `listbox` control).
#[derive(Clone)]
pub struct StyledListBox {
    binding: StyleBinding,
}

impl StyledListBox {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("listbox") }
    }

    /// The theme this reads.
    pub(crate) fn theme(&self) -> crate::style::lss::Theme {
        self.binding.theme()
    }

    /// The surface's resolved style, enabled or not, with or without focus (#433): read its root part.
    pub fn resolve_surface(&self, enabled: bool, focused: bool) -> std::sync::Arc<Resolved> {
        self.binding.resolve_states(
            StyleStates::new(InteractionState { focused, disabled: !enabled, ..Default::default() }),
            1.0,
        )
    }

    /// The surface's fill, border and corners on `surface`, with or without focus (#433).
    pub fn paint_surface(&self, surface: Stateful<Div>, focused: bool) -> Stateful<Div> {
        let resolved = self.resolve_surface(true, focused);
        let root = resolved.root();
        surface
            .bg(root.color("background"))
            .border(px(root.number("border_width")))
            .border_color(root.color("border"))
            .rounded(px(root.number("radius")))
    }

    /// A row's resolved style in `state` (#433): read its `row` part. `focused` is the active row while focus is
    /// visible.
    pub fn resolve_row(
        &self,
        state: ListBoxItemState,
        focused: bool,
        hover: bool,
        pressed: bool,
    ) -> std::sync::Arc<Resolved> {
        let interaction =
            InteractionState { hovered: hover, pressed, focused, disabled: !state.enabled, ..Default::default() };
        self.binding.resolve_states(StyleStates::new(interaction).selected(state.selected), 1.0)
    }

    /// The row surface for `state`, with the hover and pressed fills when enabled. The SDK binding attaches
    /// selection and keyboard input.
    pub fn paint_row(&self, id: impl Into<ElementId>, state: ListBoxItemState, focus_visible: bool) -> Stateful<Div> {
        let focused = state.active && focus_visible;
        let (base, hover, pressed) = (
            self.resolve_row(state, focused, false, false),
            self.resolve_row(state, focused, true, false),
            self.resolve_row(state, focused, false, true),
        );
        let part = PartId::new("row");
        let (base, hover, pressed) = (base.part(part), hover.part(part), pressed.part(part));
        let font_family = base.font_family_name();
        let (hover_fill, hover_text) = (hover.color("background"), hover.color("foreground"));
        let (pressed_fill, pressed_text) = (pressed.color("background"), pressed.color("foreground"));
        let row = div()
            .id(id)
            .bg(base.color("background"))
            .text_color(base.color("foreground"))
            .text_size(px(base.font_size().unwrap_or(14.0)))
            .line_height(px(base.line_height().unwrap_or(20.0)))
            .font_weight(base.font_weight())
            .border(px(base.number("border_width")))
            .border_color(base.color("border"))
            .rounded(px(base.number("radius")))
            .px(px(base.number("padding_x")))
            .when(!font_family.is_empty(), |row| row.font_family(font_family));
        row.when(state.enabled, |row| {
            row.cursor_pointer()
                .hover(move |style| style.bg(hover_fill).text_color(hover_text))
                .active(move |style| style.bg(pressed_fill).text_color(pressed_text))
        })
    }
}

#[cfg(test)]
mod tests {}
