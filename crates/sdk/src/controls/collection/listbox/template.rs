//! Look-owned chrome for the shared listbox renderer.

use gpui::{App, Div, ElementId, Hsla, Stateful};
use crate::controls::tooltip::TooltipLook;
use super::{ListBoxAxis, ListBoxItemState, ListBoxLayout};

/// Per-frame surface and flow geometry. Colors and typography belong to the template.
pub struct ListBoxRenderModel {
    pub id: &'static str,
    pub label: &'static str,
    pub layout: ListBoxLayout,
    pub axis: ListBoxAxis,
    pub item_width: f32,
    pub content_sized: bool,
    pub focused: bool,
}

/// Appearance state for one mounted row. The SDK attaches input after rendering chrome.
pub struct ListBoxRowRenderModel {
    pub id: ElementId,
    pub state: ListBoxItemState,
    pub focus_visible: bool,
}

/// Content-free surface and row templates, following the SDK's render-model
/// contract. The shared renderer owns interaction and collection composition;
/// implementations supply only look-specific chrome and drag-preview styling.
pub trait ListBoxTemplate: Send + Sync {
    /// Build the empty surface; the SDK adds its viewport and root bindings.
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Stateful<Div>;
    /// Build content-free chrome using the stable `model.id`.
    fn render_row(&self, model: &ListBoxRowRenderModel, cx: &mut App) -> Stateful<Div>;
    /// Paint for keyed insertion markers.
    fn drop_color(&self) -> Hsla;
    /// Optional outline on the whole surface while an accepted drag is over it.
    fn surface_drop_color(&self) -> Option<Hsla>;
    /// Look-specific insertion marker thickness in logical pixels.
    fn drop_marker_width(&self) -> f32;
    /// Tooltip chrome for the persistent control's floating drag label.
    fn drag_preview_style(&self) -> TooltipLook;
}
