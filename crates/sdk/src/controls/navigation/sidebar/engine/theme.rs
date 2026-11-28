#![allow(deprecated)]

use std::sync::{Arc, OnceLock};

use gpui::Hsla;
use xgpui_luma_style::PartId;

use super::styled::resolve_sidebar_row;
use crate::style::{PartRead, StyleBinding};
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[deprecated(
    note = "the sidebar template draws from `SidebarTheme::style()`; resolve it with `resolve_sidebar_row` (#433)"
)]
#[derive(Clone, Copy, Debug)]
pub struct SidebarSectionLook {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[deprecated(
    note = "the sidebar template draws from `SidebarTheme::style()`; resolve it with `resolve_sidebar_row` (#433)"
)]
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

impl SidebarSectionLook {
    /// The look `resolved` (a sidebar's resolved style, see [`resolve_sidebar_row`]) describes for its section
    /// labels: the compatibility path for code that still reads a `SidebarSectionLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let section = resolved.part(PartId::new("section"));
        Self {
            label_color: section.color("foreground"),
            typography: section.typography(12.5, 18.0),
            height: section.number("height"),
        }
    }
}

impl SidebarItemLook {
    /// The look `resolved` (a row's resolved style, see [`resolve_sidebar_row`]) describes: the compatibility path
    /// for code that still reads a `SidebarItemLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        let foreground = item.color("foreground");
        Self {
            background: item.painted("background"),
            focus_border: item.painted("focus_border"),
            foreground,
            icon_color: foreground,
            typography: item.typography(12.5, 18.0),
            radius: item.number("radius"),
            height: item.number("height"),
            padding_x: item.number("padding_x"),
            gap: item.number("gap"),
            icon_size: root.number("item_icon_size"),
            opacity: item.opacity(),
        }
    }
}

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

    /// Appearance of native drag previews and insertion feedback.
    #[deprecated(note = "templates draw from the resolved style; use `drag_style` (#433)")]
    #[allow(deprecated)]
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_resolved(&self.drag_style())
    }
    /// The rows' text color at rest.
    fn foreground(&self) -> Hsla {
        resolve_sidebar_row(&self.style(), false, InteractionState::default())
            .part(PartId::new("item"))
            .color("foreground")
    }
    /// The section labels, built from [`Self::style`] for code that still reads a [`SidebarSectionLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_sidebar_row` (#433)")]
    fn resolve_section(&self) -> SidebarSectionLook {
        SidebarSectionLook::from_resolved(&resolve_sidebar_row(&self.style(), false, InteractionState::default()))
    }
    /// A branch row (never the current page), built from [`Self::style`] for code that still reads a
    /// [`SidebarItemLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_sidebar_row` (#433)")]
    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        SidebarItemLook::from_resolved(&resolve_sidebar_row(&self.style(), false, state))
    }
    /// A row, built from [`Self::style`] for code that still reads a [`SidebarItemLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_sidebar_row` (#433)")]
    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        SidebarItemLook::from_resolved(&resolve_sidebar_row(&self.style(), selected, state))
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

pub fn default_sidebar_theme() -> Arc<dyn SidebarTheme> {
    static THEME: OnceLock<Arc<dyn SidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSidebarTheme::sdk())).clone()
}
