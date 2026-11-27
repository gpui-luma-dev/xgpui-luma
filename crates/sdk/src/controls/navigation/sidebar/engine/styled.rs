//! Sidebar rows and section labels read from `.lss` (issue #285): `sidebar.lss`. Looks keep the shell
//! layout, typography roles, section font features and drag preview, and delegate the rows to
//! [`StyledSidebar`].

use crate::style::PartRead;
use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{SidebarItemLook, SidebarSectionLook, SidebarTheme};
use crate::style::StyleBinding;
use crate::theme::{InteractionState, LumaTextStyle};

fn typography(part: PartView<'_>) -> LumaTextStyle {
    part.typography(12.5, 18.0)
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

    /// A section label.
    pub fn section(&self) -> SidebarSectionLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let section = resolved.part(PartId::new("section"));
        SidebarSectionLook {
            label_color: section.color("foreground"),
            typography: typography(section),
            height: section.number("height"),
        }
    }

    /// The rows' text color at rest.
    pub fn foreground(&self) -> Hsla {
        self.binding.resolve(&[], 1.0).part(PartId::new("item")).color("foreground")
    }

    /// A row, selected (the current page) or not, in `state`.
    pub fn item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        let states = [
            ("selected", selected),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
            ("invalid", state.invalid),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        let foreground = item.color("foreground");
        SidebarItemLook {
            background: item.painted("background"),
            focus_border: item.painted("focus_border"),
            foreground,
            icon_color: foreground,
            typography: typography(item),
            radius: item.number("radius"),
            height: item.number("height"),
            padding_x: item.number("padding_x"),
            gap: item.number("gap"),
            icon_size: root.number("item_icon_size"),
            opacity: item.opacity(),
        }
    }

    /// A branch row, which is never the current page.
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
    fn foreground(&self) -> Hsla {
        self.0.foreground()
    }

    fn resolve_section(&self) -> SidebarSectionLook {
        self.0.section()
    }

    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        self.0.branch(state)
    }

    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        self.0.item(selected, state)
    }
}
