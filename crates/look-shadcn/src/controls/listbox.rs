//! List box — input surface tokens; accent row hover and keyboard focus.
//!
//! | Part           | Token              |
//! |----------------|--------------------|
//! | List bg        | `background`       |
//! | Disabled list  | `muted`            |
//! | Border         | `input`            |
//! | Divider        | `border`           |
//! | Row hover      | `accent` (layer)   |
//! | Focused row    | `accent`           |
//! | Disabled label | `muted-foreground` |

use gpui::{Div, ElementId, Stateful};
use xgpui_luma::controls::listbox::ListBoxItemState;

impl crate::ShadcnLook {
    /// Styled, content-free row surface. Attach SDK `ListBoxBinding` for input;
    /// the host supplies arbitrary content and all layout dimensions.
    /// The listbox chrome from `.lss` (`style/listbox.lss`).
    pub fn styled_listbox(&self) -> xgpui_luma::controls::listbox::StyledListBox {
        let look = self.clone();
        xgpui_luma::controls::listbox::StyledListBox::new(xgpui_luma::style::StyleBinding::new(
            "listbox",
            xgpui_luma::style::ThemeSource::Live(std::sync::Arc::new(move || look.style_theme())),
        ))
    }

    pub fn listbox_row(&self, id: impl Into<ElementId>, state: ListBoxItemState, focus_visible: bool) -> Stateful<Div> {
        self.styled_listbox().paint_row(id, state, focus_visible)
    }
}
