//! Look-independent content templates for arbitrary ListBox items.

use gpui::{AnyElement, App, IntoElement};

/// Data and state supplied to an item content template. Input handlers and
/// themed interaction surfaces remain outside the content template.
pub struct ListBoxItemRenderModel<'a, T> {
    /// Domain item in the current snapshot.
    pub item: &'a T,
    /// Whether this item is selected.
    pub selected: bool,
    /// Whether this is the active navigation item.
    pub active: bool,
    /// Whether the item accepts input.
    pub enabled: bool,
}

/// Reusable content callback. Templates may borrow local data and need not be
/// Send/Sync. Elements and retained GPUI handlers must own the data they retain.
pub type ListBoxItemTemplate<'a, T> = Box<dyn for<'b> Fn(&ListBoxItemRenderModel<'b, T>, &mut App) -> AnyElement + 'a>;

/// Adapt a named function or closure to a reusable template value. Render-time
/// builders may also accept the function/closure directly without this adapter.
///
/// ```
/// use gpui::{div, prelude::*};
/// use xgpui_luma::controls::listbox::{ListBoxItemRenderModel, make_listbox_item_template};
/// let template = make_listbox_item_template(
///     |model: &ListBoxItemRenderModel<'_, String>, _cx| div().child(model.item.clone())
/// );
/// ```
pub fn make_listbox_item_template<'a, T, F, E>(template: F) -> ListBoxItemTemplate<'a, T>
where
    F: for<'b> Fn(&ListBoxItemRenderModel<'b, T>, &mut App) -> E + 'a,
    E: IntoElement + 'static,
{
    Box::new(move |model, cx| template(model, cx).into_any_element())
}
