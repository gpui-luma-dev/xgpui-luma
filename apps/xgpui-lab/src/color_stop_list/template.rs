//! Presentation for [`super::ColorStopList`]: arranges the header actions and
//! the row list. Rows themselves are [`crate::color_stop::ColorStop`] controls,
//! styled by their own template (see `ColorStopListBuilder::stop_template`).

use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, IntoElement, SharedString, Window, div, prelude::*};

use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

// Template data: not every template reads every field.
#[allow(dead_code)]
pub(crate) struct ColorStopListRenderModel<'a> {
    pub id: &'a SharedString,
    pub look: &'a Look,
    pub count: usize,
    pub positioned: bool,
    /// Header actions (reverse, distribute, add) that are enabled for this list.
    pub actions: Vec<AnyElement>,
    /// The SDK ListBox holding one `ColorStop` per row.
    pub list: AnyElement,
}

pub(crate) trait ColorStopListTemplate: Send + Sync {
    fn render(&self, model: ColorStopListRenderModel<'_>, window: &mut Window, cx: &mut App) -> AnyElement;
}

struct FnColorStopListTemplate<F>(F);

impl<F, E> ColorStopListTemplate for FnColorStopListTemplate<F>
where
    F: for<'a> Fn(ColorStopListRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync,
    E: IntoElement,
{
    fn render(&self, model: ColorStopListRenderModel<'_>, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.0)(model, window, cx).into_any_element()
    }
}

/// Adapt a closure or function to a [`ColorStopListTemplate`].
#[allow(dead_code)]
pub(crate) fn make_color_stop_list_template<F, E>(template: F) -> Arc<dyn ColorStopListTemplate>
where
    F: for<'a> Fn(ColorStopListRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(FnColorStopListTemplate(template))
}

pub(crate) fn default_color_stop_list_template() -> Arc<dyn ColorStopListTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ColorStopListTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultColorStopListTemplate)).clone()
}

/// "N colors" and actions on one line above the rows.
pub(crate) struct DefaultColorStopListTemplate;

impl ColorStopListTemplate for DefaultColorStopListTemplate {
    fn render(&self, model: ColorStopListRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> AnyElement {
        let chrome = model.look.chrome();
        let header = (!model.actions.is_empty()).then(|| {
            div()
                .w_full()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .flex_1()
                        .typography_style(model.look.typography_scale(TextSize::Xs))
                        .text_color(chrome.muted_text)
                        .child(format!("{} colors", model.count)),
                )
                .children(model.actions)
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .text_color(chrome.body_text)
            .children(header)
            .child(model.list)
            .into_any_element()
    }
}
