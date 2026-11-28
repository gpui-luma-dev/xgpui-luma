#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};
use xgpui_luma_style::PartId;

use super::styled::{resolve_tab, resolve_tabs_list};
use crate::style::{PartRead, StyleBinding};
use crate::theme::{InteractionState, LumaTextStyle};

#[deprecated(note = "the tabs template draws from `TabsTheme::style()`; resolve it with `resolve_tabs_list` (#433)")]
#[derive(Clone, Copy, Debug)]
pub struct TabsListLook {
    /// List border thickness in pixels.
    pub border_width: f32,
    pub background: Option<Hsla>,
    pub border: Option<Hsla>,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[deprecated(note = "the tabs template draws from `TabsTheme::item_style()`; resolve it with `resolve_tab` (#433)")]
#[derive(Clone, Copy, Debug)]
pub struct TabsItemLook {
    /// Accessory glyph size and gap in pixels.
    pub accessory_size: f32,
    pub accessory_gap: f32,
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
    /// Selected-chip fill. `None` is transparent (line / underline tabs).
    pub background: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub height: f32,
    pub indicator_height: f32,
}

impl TabsListLook {
    /// The look `resolved` (a tab list's resolved style, see [`resolve_tabs_list`]) describes: the compatibility
    /// path for code that still reads a `TabsListLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let list = resolved.part(PartId::new("list"));
        Self {
            border_width: list.number("border_width"),
            background: list.painted("background"),
            border: list.painted("border"),
            radius: list.number("radius"),
            padding: list.number("padding"),
            gap: list.number("gap"),
        }
    }
}

impl TabsItemLook {
    /// The look `resolved` (a tab's resolved style, see [`resolve_tab`]) describes, `active` or not: the
    /// compatibility path for code that still reads a `TabsItemLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved, active: bool) -> Self {
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        Self {
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
}

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

    /// Appearance of native drag previews and insertion feedback.
    #[deprecated(note = "templates draw from the resolved style; use `drag_style` (#433)")]
    #[allow(deprecated)]
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_resolved(&self.drag_style())
    }

    /// The tab list, built from [`Self::style`] for code that still reads a [`TabsListLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_tabs_list` (#433)")]
    fn resolve_list(&self, enabled: bool) -> TabsListLook {
        TabsListLook::from_resolved(&resolve_tabs_list(&self.style(), enabled))
    }

    /// A tab, built from [`Self::item_style`] for code that still reads a [`TabsItemLook`].
    #[deprecated(note = "the template draws from `item_style()`; resolve it with `resolve_tab` (#433)")]
    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        TabsItemLook::from_resolved(&resolve_tab(&self.item_style(active, state), active, state), active)
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
