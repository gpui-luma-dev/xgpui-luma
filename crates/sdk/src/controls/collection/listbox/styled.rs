//! Listbox chrome read from `.lss` (issue #285): `listbox.lss`. Looks paint the surface from
//! [`StyledListBox::surface`] and rows with [`StyledListBox::paint_row`]; item content stays the host's.

use crate::style::PartRead;
use gpui::{Div, ElementId, Hsla, Stateful, div, prelude::*, px};
use xgpui_luma_style::{PartId, Resolved};

use super::ListBoxItemState;
use crate::style::StyleBinding;

/// The listbox surface's colors and frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListBoxSurfaceLook {
    pub background: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub radius: f32,
}

/// One row's colors, focus border, layout and text.
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

    /// The surface, enabled or not, with or without focus.
    pub fn surface(&self, enabled: bool, focused: bool) -> ListBoxSurfaceLook {
        let resolved = self.binding.resolve(&[("focused", focused), ("disabled", !enabled)], 1.0);
        let root = resolved.root();
        ListBoxSurfaceLook {
            background: root.color("background"),
            border: root.color("border"),
            border_width: root.number("border_width"),
            radius: root.number("radius"),
        }
    }

    fn resolve_row(
        &self,
        state: ListBoxItemState,
        focused: bool,
        hover: bool,
        pressed: bool,
    ) -> std::sync::Arc<Resolved> {
        let states = [
            ("selected", state.selected),
            ("hover", hover),
            ("pressed", pressed),
            ("focused", focused),
            ("disabled", !state.enabled),
        ];
        self.binding.resolve(&states, 1.0)
    }

    /// A row in `state`; `focused` is the active row while focus is visible.
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
        let base = self.row(state, focused, false, false);
        let hover = self.row(state, focused, true, false);
        let pressed = self.row(state, focused, false, true);
        let row = div()
            .id(id)
            .bg(base.background)
            .text_color(base.foreground)
            .text_size(px(base.font_size))
            .line_height(px(base.line_height))
            .font_weight(base.font_weight)
            .border(px(base.border_width))
            .border_color(base.border)
            .rounded(px(base.radius))
            .px(px(base.padding_x))
            .when(!base.font_family.is_empty(), |row| row.font_family(base.font_family.clone()));
        row.when(state.enabled, |row| {
            row.cursor_pointer()
                .hover(|style| style.bg(hover.background).text_color(hover.foreground))
                .active(|style| style.bg(pressed.background).text_color(pressed.foreground))
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
