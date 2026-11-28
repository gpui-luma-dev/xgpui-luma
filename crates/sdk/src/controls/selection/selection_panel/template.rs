use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::Arc;

use gpui::{App, Div, SharedString, Stateful, div, prelude::*};

pub use super::item_template::{
    SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler,
};
use super::item_template::{
    SelectionPanelItemRowHandlers, SelectionPanelItemRowModel, SelectionPanelItemTemplate,
    render_selection_panel_item_row,
};
use super::model::SelectionPanelItemLike;
use crate::infra::state::ControlFocusState;
use crate::infra::icon::SelectionStatusIcons;

pub struct SelectionPanelTemplateHandlers {
    pub item_hovers: Vec<SelectionPanelHoverHandler>,
    pub item_mouse_downs: Vec<SelectionPanelMouseDownHandler>,
    pub item_mouse_ups: Vec<SelectionPanelMouseUpHandler>,
    pub item_mouse_up_outs: Vec<SelectionPanelMouseUpHandler>,
    pub item_clicks: Vec<SelectionPanelClickHandler>,
}

pub type SelectionPanelModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectionPanelRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct SelectionPanelRenderModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub items: &'a [T],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub hovered_visible_index: Option<usize>,
    pub pressed_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: Option<&'a SelectionPanelItemTemplate<T>>,
    /// The panel's style (a `selection_panel` binding, which extends `floating_menu`, #433): the template
    /// resolves it and draws from its parts.
    pub menu: crate::style::StyleBinding,
    pub show_selection_marker: bool,
    pub icons: &'a SelectionStatusIcons,
    pub show_panel_chrome: bool,
}

pub trait SelectionPanelTemplate<T>: Send + Sync
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl<T> RenderModel for dyn SelectionPanelTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
{
    type Model<'a> = SelectionPanelRenderModel<'a, T>;
}

impl<T> SelectionPanelTemplate<T> for ModifiedTemplate<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply(root, model)
    }
}

pub(super) fn template_with_modifier<T, F>(
    template: Arc<dyn SelectionPanelTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectionPanelRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

pub struct DefaultSelectionPanelTemplate;

impl<T> SelectionPanelTemplate<T> for DefaultSelectionPanelTemplate
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectionPanelTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;
        let style = crate::controls::floating_menu::MenuStyle::resolve(&model.menu);

        let mut root = div().id((model.panel_id.clone(), 0usize)).relative().flex().flex_col().w_full();

        if model.show_panel_chrome {
            root = style.pane(root);
        }

        let mut mouse_downs = item_mouse_downs.into_iter();
        let mut mouse_ups = item_mouse_ups.into_iter();
        let mut mouse_up_outs = item_mouse_up_outs.into_iter();
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let row_enabled = model.enabled && item.is_enabled();
            let selected = model.selected_source_index == Some(source_index);
            let active = model.active_visible_index == Some(visible_index);
            let hovered = model.hovered_visible_index == Some(visible_index);
            let pressed = model.pressed_visible_index == Some(visible_index);

            let row = render_selection_panel_item_row(
                SelectionPanelItemRowModel {
                    panel_id: model.panel_id,
                    control_id: model.control_id,
                    item,
                    source_index,
                    visible_index,
                    selected,
                    active,
                    hovered,
                    pressed,
                    focused: model.focus.focused,
                    focus_visible: model.focus.focus_visible,
                    enabled: row_enabled,
                    sibling_count: model.visible_indices.len(),
                    item_template: model.item_template,
                    style: &style,
                    show_selection_marker: model.show_selection_marker,
                    icons: model.icons.clone(),
                },
                SelectionPanelItemRowHandlers {
                    hover,
                    mouse_down: mouse_downs.next(),
                    mouse_up: mouse_ups.next(),
                    mouse_up_out: mouse_up_outs.next(),
                    click: clicks.next(),
                },
                cx,
            );

            root = root.child(row);
        }

        root
    }
}

pub fn default_selection_panel_template<T>() -> Arc<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
{
    Arc::new(DefaultSelectionPanelTemplate)
}

#[allow(clippy::too_many_arguments)]
pub fn render_selection_panel<T>(
    template: Arc<dyn SelectionPanelTemplate<T>>,
    model: &SelectionPanelRenderModel<'_, T>,
    item_hovers: Vec<SelectionPanelHoverHandler>,
    item_mouse_downs: Vec<SelectionPanelMouseDownHandler>,
    item_mouse_ups: Vec<SelectionPanelMouseUpHandler>,
    item_mouse_up_outs: Vec<SelectionPanelMouseUpHandler>,
    item_clicks: Vec<SelectionPanelClickHandler>,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectionPanelItemLike + 'static,
{
    template.render(
        model,
        SelectionPanelTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        },
        cx,
    )
}
