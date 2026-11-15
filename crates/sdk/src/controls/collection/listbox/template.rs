//! Look-owned chrome for shared listbox composition.

use gpui::{App, Div, ElementId, Hsla, Stateful};
use crate::controls::tooltip::TooltipLook;
use super::{ListBoxItemState, ListBoxLayout, ListBoxMotion};

/// Surface state supplied to a listbox template. Geometry is authored by the caller.
pub struct ListBoxRenderModel {
    pub id: &'static str,
    pub label: &'static str,
    pub layout: ListBoxLayout,
    pub focused: bool,
}

/// Appearance state for one mounted row; the SDK attaches its input bindings.
pub struct ListBoxRowRenderModel {
    /// Stable keyed identity, independent of the visible index.
    pub id: ElementId,
    pub state: ListBoxItemState,
    pub focus_visible: bool,
    /// Content determines row height; avoid filling an unspecified parent height.
    pub content_sized: bool,
}

/// Resolved drag appearance. No look-specific paint or geometry is defaulted by the renderer.
pub struct ListBoxDragLook {
    pub marker_color: Hsla,
    pub marker_width: f32,
    /// Optional outline while an accepted drag is over the whole surface.
    pub surface_color: Option<Hsla>,
    pub preview: TooltipLook,
}

/// Content-free chrome, using the same render-model/template contract as other
/// SDK collections. Templates supply appearance; the builder owns behavior.
pub trait ListBoxTemplate: Send + Sync {
    /// Build an empty outer surface; the SDK adds the viewport and root input.
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Stateful<Div>;
    /// Build content-free row chrome with the stable `model.id`.
    fn render_row(&self, model: &ListBoxRowRenderModel, cx: &mut App) -> Stateful<Div>;
    /// Resolve insertion paint, surface outline, and floating preview chrome.
    fn drag_look(&self) -> ListBoxDragLook;
    /// Look defaults are captured by the retained SDK motion configuration.
    fn motion_defaults(&self) -> ListBoxMotion;
}
