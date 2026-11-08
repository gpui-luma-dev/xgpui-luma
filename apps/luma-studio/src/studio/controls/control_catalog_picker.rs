use std::sync::Arc;

use gpui::{AnyElement, Context, FontFeatures, FontWeight, MouseButton, Window, div, prelude::*, px};
use xgpui_luma::controls::floating_menu::FloatingMenuLook;
use xgpui_luma::theme::LumaTextStyle;

use crate::studio::components::catalog::{
    ComponentCatalogEntry, ComponentCatalogGroup, controls_exposition_id, groups_for_column,
};

const COLUMN_COUNT: usize = 4;
const CATEGORY_ICON_SIZE: f32 = 14.0;
const LEAF_LEFT_INDENT: f32 = 7.0;
const COLUMN_GAP_EXTRA: f32 = 5.0;
const CONTAINER_INNER_PADDING: f32 = 15.0;

fn column_gap(menu_look: &FloatingMenuLook) -> f32 {
    menu_look.item_gap * 3.5 + COLUMN_GAP_EXTRA
}

pub(crate) fn render_control_catalog_picker<M: 'static>(
    menu_look: &FloatingMenuLook,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    let category_style = menu_look.item_typography;
    let item_style = menu_look.item_typography;

    div()
        .id("controls-catalog-picker")
        .flex_shrink_0()
        .occlude()
        .p(px(CONTAINER_INNER_PADDING))
        .bg(menu_look.background)
        .border_1()
        .border_color(menu_look.border)
        .rounded(px(menu_look.radius))
        .shadow(menu_look.shadow.clone())
        .child(div().flex_shrink_0().flex().items_start().gap(px(column_gap(menu_look))).children(
            (0..COLUMN_COUNT).map(|column| {
                render_column(
                    menu_look,
                    column,
                    selected_exposition_id,
                    interactive,
                    category_style,
                    item_style,
                    cx,
                    on_select.clone(),
                )
            }),
        ))
        .into_any_element()
}

fn render_column<M: 'static>(
    menu_look: &FloatingMenuLook,
    column: usize,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    category_style: LumaTextStyle,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(menu_look.item_gap * 1.5))
        .children(groups_for_column(column).map(|group| {
            render_category_section(
                menu_look,
                group,
                selected_exposition_id,
                interactive,
                category_style,
                item_style,
                cx,
                on_select.clone(),
            )
        }))
        .into_any_element()
}

fn render_category_section<M: 'static>(
    menu_look: &FloatingMenuLook,
    group: &ComponentCatalogGroup,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    category_style: LumaTextStyle,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(menu_look.item_gap * 0.25))
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(menu_look.item_gap))
                .whitespace_nowrap()
                .text_color(menu_look.foreground)
                .child(render_category_icon(group.icon, menu_look.foreground))
                .child(
                    div()
                        .text_size(px(category_style.size))
                        .line_height(px(category_style.line_height))
                        .font_weight(FontWeight::SEMIBOLD)
                        .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1), ("c2sc".into(), 1)])))
                        .child(group.label),
                ),
        )
        .children(group.entries.iter().map(|entry| {
            render_catalog_item(
                menu_look,
                entry,
                selected_exposition_id,
                interactive,
                item_style,
                cx,
                on_select.clone(),
            )
        }))
        .into_any_element()
}

fn render_catalog_item<M: 'static>(
    menu_look: &FloatingMenuLook,
    entry: &ComponentCatalogEntry,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    let gallery_id = entry.id;
    let exposition_id = controls_exposition_id(gallery_id);
    let selectable = interactive && exposition_id.is_some();
    let active = exposition_id.is_some_and(|id| selected_exposition_id == Some(id));

    div()
        .id(format!("controls-catalog-item-{gallery_id}"))
        .flex_shrink_0()
        .whitespace_nowrap()
        .pl(px(menu_look.item_padding_x + LEAF_LEFT_INDENT))
        .pr(px(menu_look.item_padding_x))
        .min_h(px(menu_look.item_height * 0.85))
        .flex()
        .items_center()
        .rounded(px(menu_look.item_radius))
        .text_size(px(item_style.size))
        .line_height(px(item_style.line_height))
        .text_color(if active {
            menu_look.item_hover_foreground
        } else {
            menu_look.foreground
        })
        .font_weight(if active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .when(active, |item| item.bg(menu_look.item_hover_background))
        .when(selectable, |item| item.cursor_pointer())
        .when(selectable, |item| {
            item.hover(move |style| {
                style.bg(menu_look.item_hover_background).text_color(menu_look.item_hover_foreground)
            })
        })
        .when(selectable, |item| {
            let exposition_id = exposition_id.expect("selectable catalog item must map to exposition");
            item.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_select(this, exposition_id, window, cx);
                }),
            )
        })
        .child(entry.label)
        .into_any_element()
}

fn render_category_icon(icon: lucide_svg_static::Icon, color: gpui::Hsla) -> AnyElement {
    div()
        .flex_shrink_0()
        .child(xgpui_luma::infra::icon::lucide_icon(icon, color, CATEGORY_ICON_SIZE))
        .into_any_element()
}
