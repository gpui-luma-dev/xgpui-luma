use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

pub struct ComboBoxItemRenderModel<'a, T> {
    pub combobox_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

pub type ComboBoxItemTemplate<T> =
    Arc<dyn for<'a> Fn(&ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub type ComboBoxItemTemplateModifier<T> =
    Box<dyn for<'a> Fn(AnyElement, &ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_combobox_item_template<T, F, E>(template: F) -> ComboBoxItemTemplate<T>
where
    F: for<'a> Fn(&ComboBoxItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

pub(super) fn item_template_with_modifier<T: 'static, F>(
    template: ComboBoxItemTemplate<T>,
    modifier: F,
) -> ComboBoxItemTemplate<T>
where
    F: for<'a> Fn(AnyElement, &ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(move |model, cx| {
        let element = template(model, cx);
        modifier(element, model, cx)
    })
}
