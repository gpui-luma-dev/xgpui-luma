use std::sync::{Arc, OnceLock};

use gpui::Hsla;
use xgpui_luma_style::PartId;

use super::styled::resolve_sidebar_row;
use crate::style::{PartRead, StyleBinding};
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

pub trait SidebarTheme: Send + Sync {
    /// The sidebar's style (#433): the binding the template resolves for its rows and section labels (see
    /// [`resolve_sidebar_row`]) and draws from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("sidebar")
    }

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
    /// Appearance of native drag previews and insertion feedback (#433): the SDK's `drag_preview` with the
    /// sidebar's colors, corner and text set over it.
    fn drag_style(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let base = crate::infra::drag_drop::resolve_drag_preview(&crate::infra::drag_drop::sdk_drag_preview());
        let base = base.root();
        let row = resolve_sidebar_row(&self.style(), true, InteractionState { focused: true, ..Default::default() });
        let item = row.part(PartId::new("item"));
        let foreground = item.color("foreground");
        let invalid =
            resolve_sidebar_row(&self.style(), false, InteractionState { invalid: true, ..Default::default() });
        crate::infra::drag_drop::drag_preview_over(
            item.painted("background").unwrap_or(base.color("background")),
            foreground,
            item.painted("focus_border").unwrap_or(foreground),
            invalid.part(PartId::new("item")).color("foreground"),
            item.number("radius"),
            item.typography(12.5, 18.0),
            None,
        )
    }

    /// The rows' text color at rest.
    fn foreground(&self) -> Hsla {
        resolve_sidebar_row(&self.style(), false, InteractionState::default())
            .part(PartId::new("item"))
            .color("foreground")
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_sidebar_theme() -> Arc<dyn SidebarTheme> {
    static THEME: OnceLock<Arc<dyn SidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSidebarTheme::sdk())).clone()
}
