//! The SDK's default listbox chrome (#397). A listbox is a container of anything: the host supplies each item's
//! content and lays it out; this draws only the surface and each row's state chrome from `listbox.lss`
//! ([`StyledListBox`]): fill, border and focus ring for selected, hover, pressed, focused and disabled, and the text
//! style text content inherits. The focus ring is the drop marker, and the theme's tooltip the drag preview. Looks can still supply their
//! own [`ListBoxTemplate`]; this is what a listbox draws without one ([`super::ListBoxBuilder::build_default`]).

use crate::style::PartRead;
use gpui::{App, Div, Stateful, div, prelude::*};

use super::{ListBoxDragLook, ListBoxMotion, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxTemplate, StyledListBox};
use crate::controls::tooltip::resolve_tooltip;
use crate::style::{StyleBinding, ThemeSource};

/// Listbox chrome drawn from a theme's `listbox.lss`.
#[derive(Clone)]
pub struct ThemedListBoxTemplate {
    styled: StyledListBox,
    /// The same theme's `tooltip`, for the drag preview.
    tooltip: StyleBinding,
}

impl ThemedListBoxTemplate {
    /// Chrome from `styled`'s theme.
    pub fn new(styled: StyledListBox) -> Self {
        let tooltip = StyleBinding::new("tooltip", ThemeSource::Fixed(styled.theme()));
        Self { styled, tooltip }
    }

    /// Chrome from the SDK's `listbox.lss` alone.
    pub fn sdk() -> Self {
        Self::new(StyledListBox::sdk())
    }

    /// The `.lss` values this template draws.
    pub fn styled(&self) -> &StyledListBox {
        &self.styled
    }
}

impl ListBoxTemplate for ThemedListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, _: &mut App) -> Stateful<Div> {
        self.styled
            .paint_surface(div().id(model.id), model.focused)
            .overflow_hidden()
            .p_1()
            .w_full()
            .min_w_0()
    }

    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Stateful<Div> {
        self.styled
            .paint_row(model.id.clone(), model.state, model.focus_visible)
            // No layout of its own: the content is the host's, laid out as the host likes.
            .w_full()
            .when(!model.content_sized, |row| row.h_full())
    }

    fn drag_look(&self) -> ListBoxDragLook {
        ListBoxDragLook {
            marker_color: self.styled.resolve_surface(true, true).root().color("border"),
            marker_width: 2.,
            surface_color: None,
            preview: resolve_tooltip(&self.tooltip),
        }
    }

    fn motion_defaults(&self) -> ListBoxMotion {
        ListBoxMotion::default()
    }
}
