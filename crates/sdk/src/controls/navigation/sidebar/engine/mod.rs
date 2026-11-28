//! Flush presentation engine formerly known as `navigation_sidebar`.
//!
//! Kept as an internal implementation detail of [`super::SidebarControl`].
//! Theme/template types are re-exported from [`super`] for look-crate binding.

#![allow(dead_code)]
#![allow(unused_imports)]

mod styled;
pub use styled::{StyledSidebar, StyledSidebarTheme, resolve_sidebar_row};
mod control;
mod model;
mod template;
mod theme;

pub(crate) use control::{SidebarPanelEngine, SidebarPanelEngineEvent};
pub(crate) use model::{
    NavHostedContent, NavNode, NavPresenter, SidebarPanelEngineBuilder, SidebarPanelEngineModel, entity_presenter,
};
pub use template::{
    SidebarPanelTemplate, SidebarPanelTemplateHandlers, SidebarPanelTemplateModifier, ThemedSidebarPanelTemplate,
    default_sidebar_panel_template,
};
pub(crate) use template::modified_sidebar_panel_template;

pub use theme::{SidebarTheme, default_sidebar_theme};

// Public presentation contracts; the retained engine remains internal.
pub use model::{NavNodeKind, NavNodeState, SidebarPanelEngineRenderModel, RenderedNavNode, RenderedRailSubmenu};
