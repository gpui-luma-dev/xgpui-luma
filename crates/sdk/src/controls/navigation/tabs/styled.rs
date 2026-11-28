//! Tabs read from `.lss` (issue #285): `tabs.lss`. Looks keep their own drag preview, baseline and content
//! padding, and delegate the list and tabs to [`StyledTabs`].

use std::sync::Arc;

use crate::style::StyleStates;
use gpui::SharedString;
use xgpui_luma_style::Resolved;

use super::theme::TabsTheme;
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// The tab list's style for `binding` (the `tabs` control), enabled or not (#433): the template draws from its
/// `list` part.
pub fn resolve_tabs_list(binding: &StyleBinding, enabled: bool) -> Arc<Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
}

/// A tab's style for `binding`, `active` or not, in `state` (#433): the template draws from its root and `item`
/// part.
pub fn resolve_tab(binding: &StyleBinding, active: bool, state: InteractionState) -> Arc<Resolved> {
    binding.resolve_states(StyleStates::new(state).selected(active), 1.0)
}

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

    /// The binding the tabs resolve.
    pub fn binding(&self) -> StyleBinding {
        self.binding.clone()
    }

    /// The tabs' font family.
    pub fn font_family(&self) -> SharedString {
        use crate::style::PartRead;
        resolve_tabs_list(&self.binding, true)
            .part(xgpui_luma_style::PartId::new("item"))
            .font_family_name()
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
    fn style(&self) -> StyleBinding {
        self.0.binding()
    }
}
