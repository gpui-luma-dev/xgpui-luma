use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{InteractionState, LumaTextStyle, ThemeTokens};

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
    /// Appearance of native drag previews and insertion feedback.
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        let mut drag = crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default());
        let list = self.resolve_list(true);
        let item = self.resolve_item(true, Default::default());
        drag.background = list.background.or(item.background).unwrap_or(drag.background);
        drag.foreground = item.label_color;
        drag.valid_marker = item.indicator.unwrap_or(item.label_color);
        drag.invalid_marker =
            self.resolve_item(false, InteractionState { invalid: true, ..Default::default() }).label_color;
        drag.radius = list.radius;
        drag.typography = item.label_typography;
        drag.font_family = self.font_family();
        drag
    }
    fn resolve_list(&self, enabled: bool) -> TabsListLook;
    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook;
    fn font_family(&self) -> SharedString;
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
