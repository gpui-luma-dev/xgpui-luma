//! Tabs read from `.lss` (issue #285): `tabs.lss`. Looks keep their own drag preview, baseline and content
//! padding, and delegate the list and tabs to [`StyledTabs`].

use crate::style::StyleStates;
use crate::style::PartRead;
use gpui::SharedString;
use xgpui_luma_style::PartId;

use super::theme::{TabsItemLook, TabsListLook, TabsTheme};
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// The tabs' `.lss` values for one binding (the `tabs` control).
#[derive(Clone)]
pub struct StyledTabs {
    binding: StyleBinding,
}

impl StyledTabs {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("tabs") }
    }

    /// The tab list, enabled or not.
    pub fn list(&self, enabled: bool) -> TabsListLook {
        let resolved = self.binding.resolve(&[("disabled", !enabled)], 1.0);
        let list = resolved.part(PartId::new("list"));
        TabsListLook {
            border_width: list.number("border_width"),
            background: list.painted("background"),
            border: list.painted("border"),
            radius: list.number("radius"),
            padding: list.number("padding"),
            gap: list.number("gap"),
        }
    }

    /// A tab, active or not, in `state`.
    pub fn item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        let states = StyleStates::new(state).selected(active);
        let resolved = self.binding.resolve_states(states, 1.0);
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        TabsItemLook {
            accessory_size: root.number("accessory_size"),
            accessory_gap: root.number("accessory_gap"),
            label_color: item.color("foreground"),
            indicator: if active { root.painted("indicator") } else { None },
            background: item.painted("background"),
            label_typography: item.typography(14.0, 20.0),
            radius: item.number("radius"),
            padding_x: item.number("padding_x"),
            height: item.number("height"),
            indicator_height: root.number("indicator_height"),
        }
    }

    /// The tabs' font family.
    pub fn font_family(&self) -> SharedString {
        let resolved = self.binding.resolve(&[], 1.0);
        resolved.part(PartId::new("item")).font_family_name()
    }
}

/// The SDK's tabs theme: `tabs.lss`, with the SDK's drag preview.
pub struct StyledTabsTheme(StyledTabs);

impl StyledTabsTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self(StyledTabs::new(binding))
    }

    pub fn sdk() -> Self {
        Self(StyledTabs::sdk())
    }
}

impl TabsTheme for StyledTabsTheme {
    fn resolve_list(&self, enabled: bool) -> TabsListLook {
        self.0.list(enabled)
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        self.0.item(active, state)
    }

    fn font_family(&self) -> SharedString {
        self.0.font_family()
    }
}
