//! Listbox chrome read from `.lss` (issue #285): `listbox.lss`. Looks paint the surface from
//! [`StyledListBox::surface`] and rows with [`StyledListBox::paint_row`]; item content stays the host's.

#![allow(deprecated)] // ListBoxSurfaceLook and ListBoxRowLook are this module's compatibility types.

use crate::theme::InteractionState;
use crate::style::StyleStates;
use crate::style::PartRead;
use gpui::{Div, ElementId, Hsla, Stateful, div, prelude::*, px};
use xgpui_luma_style::{PartId, Resolved};

use super::ListBoxItemState;
use crate::style::StyleBinding;

/// The listbox surface's colors and frame, copied out of its style (#433). A compatibility type: templates read
/// [`StyledListBox::resolve_surface`].
#[deprecated(note = "read `StyledListBox::resolve_surface` instead (#433)")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListBoxSurfaceLook {
    pub background: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub radius: f32,
}

/// One row's colors, focus border, layout and text, copied out of its style (#433). A compatibility type:
/// templates read [`StyledListBox::resolve_row`].
#[deprecated(note = "read `StyledListBox::resolve_row` instead (#433)")]
#[derive(Clone, Debug, PartialEq)]
pub struct ListBoxRowLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub radius: f32,
    pub padding_x: f32,
    pub font_family: gpui::SharedString,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: gpui::FontWeight,
}

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

    /// The surface, enabled or not, with or without focus.
    #[deprecated(note = "read `resolve_surface` instead (#433)")]
    pub fn surface(&self, enabled: bool, focused: bool) -> ListBoxSurfaceLook {
        let resolved = self.resolve_surface(enabled, focused);
        let root = resolved.root();
        ListBoxSurfaceLook {
            background: root.color("background"),
            border: root.color("border"),
            border_width: root.number("border_width"),
            radius: root.number("radius"),
        }
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

    /// A row in `state`; `focused` is the active row while focus is visible.
    #[deprecated(note = "read `resolve_row` instead (#433)")]
    pub fn row(&self, state: ListBoxItemState, focused: bool, hover: bool, pressed: bool) -> ListBoxRowLook {
        let resolved = self.resolve_row(state, focused, hover, pressed);
        let row = resolved.part(PartId::new("row"));
        ListBoxRowLook {
            background: row.color("background"),
            foreground: row.color("foreground"),
            border: row.color("border"),
            border_width: row.number("border_width"),
            radius: row.number("radius"),
            padding_x: row.number("padding_x"),
            font_family: row.font_family_name(),
            font_size: row.font_size().unwrap_or(14.0),
            line_height: row.line_height().unwrap_or(20.0),
            font_weight: row.font_weight(),
        }
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
mod tests {
    use super::*;

    #[test]
    fn sdk_rows_follow_selection_and_focus() {
        let lss = StyledListBox::sdk();
        let idle = ListBoxItemState { selected: false, active: false, enabled: true };
        let selected = ListBoxItemState { selected: true, ..idle };
        let disabled = ListBoxItemState { enabled: false, ..idle };
        assert_eq!(lss.row(idle, false, false, false).background.a, 0.0);
        assert!(lss.row(selected, false, false, false).background.a > 0.0);
        assert!(lss.row(idle, false, true, false).background.a > 0.0);
        assert_eq!(lss.row(idle, false, false, false).border.a, 0.0);
        assert!(lss.row(idle, true, false, false).border.a > 0.0);
        assert_ne!(lss.row(disabled, false, false, false).foreground, lss.row(idle, false, false, false).foreground);
        let surface = lss.surface(true, false);
        assert!(surface.border_width > 0.0);
        assert_ne!(lss.surface(true, true).border, surface.border);
    }
}
