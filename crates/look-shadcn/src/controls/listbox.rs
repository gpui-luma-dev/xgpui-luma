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

use crate::provenance::ResolvedColor;

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

#[derive(Clone, Debug)]
pub struct ListBoxListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

impl ListBoxListColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListBoxRowColorTable {
    pub label_color: ResolvedColor,
    pub background: ResolvedColor,
}

impl ListBoxRowColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), background: ResolvedColor::transparent() }
    }
}
