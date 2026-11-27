use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, SharedString, Stateful,
    Window, div, prelude::*, px,
};

use crate::infra::icon::lucide_icon;
use crate::infra::icon::{SelectionStatusIcons, render_icon_source};
use super::model::SelectionPanelItemLike;
use super::theme::SelectionPanelLook;

#[derive(Clone, Debug)]
pub struct SelectionPanelItemRenderModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub sibling_count: usize,
}

pub type SelectionPanelItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selection_panel_item_template<T, F, E>(template: F) -> SelectionPanelItemTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

pub type SelectionPanelItemTemplateModifier<T> = Box<
    dyn for<'a> Fn(AnyElement, &SelectionPanelItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
>;

pub(super) fn item_template_with_modifier<T, F>(
    template: SelectionPanelItemTemplate<T>,
    modifier: F,
) -> SelectionPanelItemTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(AnyElement, &SelectionPanelItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(move |model, cx| {
        let element = template(model, cx);
        modifier(element, model, cx)
    })
}

pub type SelectionPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectionPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectionPanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectionPanelMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub(crate) struct SelectionPanelItemRowHandlers {
    pub hover: SelectionPanelHoverHandler,
    pub mouse_down: Option<SelectionPanelMouseDownHandler>,
    pub mouse_up: Option<SelectionPanelMouseUpHandler>,
    pub mouse_up_out: Option<SelectionPanelMouseUpHandler>,
    pub click: Option<SelectionPanelClickHandler>,
}

pub(crate) struct SelectionPanelItemRowModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub sibling_count: usize,
    pub item_template: Option<&'a SelectionPanelItemTemplate<T>>,
    pub look: &'a SelectionPanelLook,
    pub show_selection_marker: bool,
    pub icons: SelectionStatusIcons,
}

pub(crate) fn render_selection_panel_item_row<T>(
    model: SelectionPanelItemRowModel<'_, T>,
    handlers: SelectionPanelItemRowHandlers,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectionPanelItemLike + 'static,
{
    let SelectionPanelItemRowHandlers { hover, mouse_down, mouse_up, mouse_up_out, click } = handlers;

    let color = if model.enabled {
        model.look.foreground
    } else {
        model.look.item_disabled_foreground
    };

    let content = if let Some(item_template) = model.item_template {
        item_template(
            &SelectionPanelItemRenderModel {
                panel_id: model.panel_id,
                control_id: model.control_id,
                item: model.item,
                source_index: model.source_index,
                visible_index: model.visible_index,
                selected: model.selected,
                active: model.active,
                hovered: model.hovered,
                pressed: model.pressed,
                focused: model.focused,
                focus_visible: model.focus_visible,
                enabled: model.enabled,
                sibling_count: model.sibling_count,
            },
            cx,
        )
    } else {
        div().flex_1().min_w(px(0.0)).truncate().child(model.item.label().clone()).into_any_element()
    };

    let mut row = div()
        .id((model.panel_id.clone(), model.visible_index))
        .flex()
        .items_center()
        .gap(px(model.look.item_gap))
        .min_h(px(model.look.item_height))
        .px(px(model.look.item_padding_x))
        .rounded(px(model.look.item_radius))
        .text_color(color)
        .text_size(px(model.look.item_typography.size))
        .line_height(px(model.look.item_typography.line_height))
        .font_weight(model.look.item_typography.weight)
        .when_some(model.item.icon(), |row, icon| {
            if let Some(icon) = icon.lucide() {
                row.child(lucide_icon(icon, color, model.look.item_icon_size))
            } else if let Some(path) = icon.svg_path() {
                row.child(gpui::svg().external_path(path.clone()).size(px(model.look.item_icon_size)).text_color(color))
            } else {
                row
            }
        })
        .child(div().flex_1().child(content));

    if model.show_selection_marker {
        if model.selected {
            row = row.child(render_icon_source(&model.icons.selected, color, model.look.item_icon_size));
        } else {
            row = row.child(div().size(px(model.look.item_icon_size)));
        }
    }

    if model.enabled {
        row = row.cursor_pointer().on_hover(hover).hover({
            let hover_background = model.look.item_hover_background;
            let hover_foreground = model.look.item_hover_foreground;
            move |style| style.bg(hover_background).text_color(hover_foreground)
        });

        if model.active || model.pressed {
            row = row.bg(model.look.item_hover_background).text_color(model.look.item_hover_foreground);
        }

        if let Some(mouse_down) = mouse_down {
            row = row.on_mouse_down(MouseButton::Left, mouse_down);
        }
        if let Some(mouse_up) = mouse_up {
            row = row.on_mouse_up(MouseButton::Left, mouse_up);
        }
        if let Some(mouse_up_out) = mouse_up_out {
            row = row.on_mouse_up_out(MouseButton::Left, mouse_up_out);
        }
        if let Some(click) = click {
            row = row.on_click(click);
        }
    } else {
        row = crate::style::fade(row, model.look.item_disabled_opacity);
    }

    row
}
