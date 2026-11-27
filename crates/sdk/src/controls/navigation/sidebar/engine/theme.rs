use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct SidebarSectionLook {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SidebarItemLook {
    pub background: Option<Hsla>,
    pub focus_border: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub typography: LumaTextStyle,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub icon_size: f32,
    /// The row's opacity: the `item` part's resolved `opacity`, which a look sets under `:disabled`.
    pub opacity: f32,
}

pub trait SidebarTheme: Send + Sync {
    /// Shared title/subtitle typography and section OpenType treatment.
    fn typography(&self) -> crate::theme::LumaTypography {
        crate::theme::LumaTypography::default()
    }
    fn section_font_features(&self) -> gpui::FontFeatures {
        crate::theme::stylesheet::CommonStylesheet::sdk_default().font_features.resolve("sidebar_section")
    }

    /// Shared shell and row decoration used by the stock template: the SDK's `sidebar.lss` unless a theme
    /// reads its own.
    fn geometry(&self) -> crate::theme::stylesheet::SidebarGeometry {
        crate::theme::stylesheet::SidebarGeometry::sdk()
    }
    /// Appearance of native drag previews and insertion feedback.
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        let mut drag = crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default());
        let item = self.resolve_item(true, InteractionState { focused: true, ..Default::default() });
        drag.background = item.background.unwrap_or(drag.background);
        drag.foreground = item.foreground;
        drag.valid_marker = item.focus_border.unwrap_or(item.foreground);
        drag.invalid_marker =
            self.resolve_item(false, InteractionState { invalid: true, ..Default::default() }).foreground;
        drag.radius = item.radius;
        drag.typography = item.typography;
        drag
    }
    fn foreground(&self) -> Hsla;
    fn resolve_section(&self) -> SidebarSectionLook;
    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook;
    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_sidebar_theme() -> Arc<dyn SidebarTheme> {
    static THEME: OnceLock<Arc<dyn SidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSidebarTheme::sdk())).clone()
}
