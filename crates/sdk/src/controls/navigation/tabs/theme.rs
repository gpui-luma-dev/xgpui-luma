use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};
use xgpui_luma_style::PartId;

use super::styled::{resolve_tab, resolve_tabs_list};
use crate::style::{PartRead, StyleBinding};
use crate::theme::InteractionState;

/// Full-list rule painted behind the active indicator without consuming layout space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsBaseline {
    pub color: Hsla,
    pub height: f32,
    pub inset_x: f32,
    /// Distance above the bottom of the list, in logical pixels.
    pub bottom: f32,
}

pub trait TabsTheme: Send + Sync {
    /// The tabs' style (#433): the binding the template resolves for the list (see [`resolve_tabs_list`]) and, by
    /// default, for each tab.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("tabs")
    }

    /// A tab's style, `active` or not, in `state` (see [`resolve_tab`]): [`Self::style`] unless a theme sets values
    /// per state (with [`StyleBinding::overrides`]).
    fn item_style(&self, _active: bool, _state: InteractionState) -> StyleBinding {
        self.style()
    }

    /// Appearance of native drag previews and insertion feedback (#433): the SDK's `drag_preview` with the tabs'
    /// colors, corner and text set over it.
    fn drag_style(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let base = crate::infra::drag_drop::resolve_drag_preview(&crate::infra::drag_drop::sdk_drag_preview());
        let base = base.root();
        let list_style = resolve_tabs_list(&self.style(), true);
        let list = list_style.part(PartId::new("list"));
        let state = InteractionState::default();
        let active = resolve_tab(&self.item_style(true, state), true, state);
        let item = active.part(PartId::new("item"));
        let invalid = InteractionState { invalid: true, ..Default::default() };
        let label = item.color("foreground");
        crate::infra::drag_drop::drag_preview_over(
            list.painted("background").or(item.painted("background")).unwrap_or(base.color("background")),
            label,
            active.root().painted("indicator").unwrap_or(label),
            resolve_tab(&self.item_style(false, invalid), false, invalid)
                .part(PartId::new("item"))
                .color("foreground"),
            list.number("radius"),
            item.typography(14.0, 20.0),
            Some(self.font_family()),
        )
    }

    /// The tabs' font family: the `item` part's, at rest.
    fn font_family(&self) -> SharedString {
        resolve_tabs_list(&self.style(), true).part(PartId::new("item")).font_family_name()
    }

    /// Optional horizontal/vertical padding for an inset label background.
    /// Horizontal padding is taken from the trigger padding, preserving its width.
    fn content_padding(&self) -> Option<(f32, f32)> {
        None
    }
    /// Independent of the selected-tab indicator. Absent by default.
    fn baseline(&self, _enabled: bool) -> Option<TabsBaseline> {
        None
    }
    /// Override the active indicator's inset; `None` retains the item padding inset.
    fn indicator_inset(&self) -> Option<f32> {
        None
    }
}

pub fn default_tabs_theme() -> Arc<dyn TabsTheme> {
    static THEME: OnceLock<Arc<dyn TabsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTabsTheme::sdk())).clone()
}
