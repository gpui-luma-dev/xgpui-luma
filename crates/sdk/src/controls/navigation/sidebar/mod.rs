mod drag;
pub use drag::{SidebarDragDrop, SidebarDragEvent, SidebarDropLocation, SidebarDropProposal, SidebarItemDragBinding};
mod control;
/// Internal flush-list presentation engine (former `navigation_sidebar`).
mod engine;
mod model;
pub(crate) mod theme;

pub use control::{SidebarControl, SidebarEvent};
pub use model::{
    SidebarBuilder, SidebarContentBuilder, SidebarControlBuilder, SidebarControlModel, SidebarFooterBuilder,
    SidebarGroupBuilder, SidebarHeaderBuilder, SidebarMenuBuilder, SidebarMenuItemBuilder, SidebarMenuItemModel,
    SidebarMenuModel, SidebarMenuSubBuilder, SidebarPanelModel,
};
pub use theme::{SidebarMetricScale, SidebarPresentation};

// Look-bound flush theme / template surface (presentation engine).

pub use engine::{
    SidebarPanelTemplate, SidebarPanelTemplateHandlers, SidebarPanelTemplateModifier, SidebarTheme, StyledSidebar,
    StyledSidebarTheme, ThemedSidebarPanelTemplate, default_sidebar_panel_template, default_sidebar_theme,
    resolve_sidebar_row,
};

use gpui::SharedString;

pub fn new(id: impl Into<SharedString>) -> SidebarControlBuilder {
    SidebarControlBuilder::new(id)
}

pub fn sidebar(id: impl Into<SharedString>) -> SidebarBuilder {
    SidebarBuilder::new(id)
}

pub fn sidebar_header() -> SidebarHeaderBuilder {
    SidebarHeaderBuilder::new()
}

pub fn sidebar_content() -> SidebarContentBuilder {
    SidebarContentBuilder::new()
}

pub fn sidebar_group() -> SidebarGroupBuilder {
    SidebarGroupBuilder::new()
}

pub fn sidebar_menu(id: impl Into<SharedString>) -> SidebarMenuBuilder {
    SidebarMenuBuilder::new(id)
}

pub fn sidebar_menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> SidebarMenuItemBuilder {
    SidebarMenuItemBuilder::new(id, label)
}

pub fn sidebar_menu_sub() -> SidebarMenuSubBuilder {
    SidebarMenuSubBuilder::new()
}

pub fn sidebar_footer() -> SidebarFooterBuilder {
    SidebarFooterBuilder::new()
}

mod callback;
pub use callback::SidebarEventExt;

#[cfg(all(test, feature = "test-support"))]
mod drag_tests;

pub use model::SidebarMotion;

/// Presentation-only models for custom sidebar templates and forced-state previews.
/// They do not expose the retained navigation engine or its behavior.
pub use engine::{
    SidebarPanelEngineRenderModel as SidebarPanelRenderModel, RenderedNavNode as SidebarRenderedNode,
    NavNodeState as SidebarNodeState, NavNodeKind as SidebarNodeKind, RenderedRailSubmenu as SidebarRailSubmenu,
};
