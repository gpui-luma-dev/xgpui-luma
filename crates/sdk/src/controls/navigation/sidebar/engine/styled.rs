//! Sidebar rows and section labels read from `.lss` (issue #285): `sidebar.lss`. Looks keep the shell
//! layout, typography roles, section font features and drag preview, and delegate the rows to
//! [`StyledSidebar`].

use std::sync::Arc;

use crate::style::StyleStates;
use gpui::Hsla;
use xgpui_luma_style::Resolved;

#[allow(deprecated)]
use super::theme::{SidebarItemLook, SidebarSectionLook};
use super::theme::SidebarTheme;
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// A sidebar row's style for `binding` (the `sidebar` control), `selected` (the current page) or not, in `state`
/// (#433): the template draws rows from its root and `item` part, and section labels from its `section` part
/// at rest.
pub fn resolve_sidebar_row(binding: &StyleBinding, selected: bool, state: InteractionState) -> Arc<Resolved> {
    binding.resolve_states(StyleStates::new(state).selected(selected), 1.0)
}

/// The sidebar's `.lss` values for one binding (the `sidebar` control).
#[derive(Clone)]
pub struct StyledSidebar {
    binding: StyleBinding,
}

impl StyledSidebar {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("sidebar") }
    }

    /// The binding the sidebar resolves.
    pub fn binding(&self) -> StyleBinding {
        self.binding.clone()
    }

    /// A section label.
    #[deprecated(note = "the template draws from the resolved style; use `resolve_sidebar_row` (#433)")]
    #[allow(deprecated)]
    pub fn section(&self) -> SidebarSectionLook {
        SidebarSectionLook::from_resolved(&resolve_sidebar_row(&self.binding, false, InteractionState::default()))
    }

    /// The rows' text color at rest.
    pub fn foreground(&self) -> Hsla {
        use crate::style::PartRead;
        resolve_sidebar_row(&self.binding, false, InteractionState::default())
            .part(xgpui_luma_style::PartId::new("item"))
            .color("foreground")
    }

    /// A row, selected (the current page) or not, in `state`.
    #[deprecated(note = "the template draws from the resolved style; use `resolve_sidebar_row` (#433)")]
    #[allow(deprecated)]
    pub fn item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        SidebarItemLook::from_resolved(&resolve_sidebar_row(&self.binding, selected, state))
    }

    /// A branch row, which is never the current page.
    #[deprecated(note = "the template draws from the resolved style; use `resolve_sidebar_row` (#433)")]
    #[allow(deprecated)]
    pub fn branch(&self, state: InteractionState) -> SidebarItemLook {
        self.item(false, state)
    }
}

/// The SDK's sidebar theme: `sidebar.lss` for the rows, with the SDK's layout and drag preview.
pub struct StyledSidebarTheme(StyledSidebar);

impl StyledSidebarTheme {
    pub fn sdk() -> Self {
        Self(StyledSidebar::sdk())
    }
}

impl SidebarTheme for StyledSidebarTheme {
    fn style(&self) -> StyleBinding {
        self.0.binding()
    }
}
