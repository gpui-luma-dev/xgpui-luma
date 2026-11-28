use std::sync::Arc;

use crate::style::PartRead;
use gpui::{AnyElement, App, SharedString, Window, div, prelude::*};

use super::model::{TableColumnCellTemplate, TableRowRenderModel};

pub type TableColumnRenderModel<'a, T> = TableRowRenderModel<'a, T>;

pub fn default_text_column_template<T, F, S>(value_fn: F) -> TableColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().truncate().child(value.into()).into_any_element()
    })
}

pub fn default_muted_column_template<T, F, S>(value_fn: F) -> TableColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div()
            .opacity(model.style.root().number("column_muted_opacity"))
            .truncate()
            .child(value.into())
            .into_any_element()
    })
}

pub fn default_emphasis_column_template<T, F, S>(value_fn: F) -> TableColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div()
            .font_weight(gpui::FontWeight(model.style.root().number("column_emphasis_weight").max(100.0)))
            .truncate()
            .child(value.into())
            .into_any_element()
    })
}

pub fn default_numeric_column_template<T, F, S>(value_fn: F) -> TableColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().w_full().flex().justify_end().truncate().child(value.into()).into_any_element()
    })
}

pub fn column_template_with_modifier<T, F>(base: TableColumnCellTemplate<T>, modifier: F) -> TableColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(AnyElement, &TableColumnRenderModel<'_, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(move |model, window, cx| {
        let cell = base(model, window, cx);
        modifier(cell, model, window, cx)
    })
}
