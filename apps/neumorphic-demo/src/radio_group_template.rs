use std::cell::Cell;
use std::sync::{Arc, OnceLock};

use gpui::{App, FontWeight, MouseButton, Stateful, Window, div, hsla, px, prelude::*};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::infra::ElementExt;
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use xgpui_luma::controls::button_family::ButtonFamilyLook;
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma::controls::control_group::{ControlGroupItemLike, ControlGroupRenderModel, ControlGroupTemplateHandlers};
use xgpui_luma::controls::radio_group::RadioGroupTemplate;

pub fn neumorphic_radio_group_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    let button_template = segmented_button_template();

    Arc::new(move |model, handlers, window, cx| {
        let rows = render_segmented_rows(model, handlers, &button_template, window, cx);

        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .p(px(4.0))
            .gap(px(2.0))
            .bg(hsla(220.0 / 360.0, 0.09, 0.86, 0.98))
            .border_1()
            .border_color(hsla(220.0 / 360.0, 0.08, 0.78, 0.58))
            .rounded(px(21.0))
            .children(rows);

        if !model.enabled {
            root = root.opacity(0.56);
        }

        root
    })
}

fn segmented_button_template() -> Arc<dyn ButtonTemplate<bool>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<bool>>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(NeumorphicSegmentedButtonTemplate)).clone()
}

struct NeumorphicSegmentedButtonTemplate;

impl ButtonTemplate<bool> for NeumorphicSegmentedButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, _cx: &mut App) -> Stateful<gpui::Div> {
        let selected = model.data;
        let state = model.state;

        let background = if selected {
            if state.pressed {
                hsla(220.0 / 360.0, 0.08, 0.95, 0.98)
            } else {
                hsla(0.0, 0.0, 0.995, 0.98)
            }
        } else {
            hsla(0.0, 0.0, 1.0, 0.0)
        };

        let border = if selected {
            hsla(220.0 / 360.0, 0.08, 0.74, 0.52)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0)
        };

        let text_color = if selected {
            hsla(220.0 / 360.0, 0.10, 0.18, 0.96)
        } else if state.hovered {
            hsla(220.0 / 360.0, 0.08, 0.42, 0.94)
        } else {
            hsla(220.0 / 360.0, 0.08, 0.54, 0.94)
        };

        let thumb_shadow = if selected && !state.disabled {
            vec![
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.14, 0.18, 0.20),
                    offset: gpui::point(px(0.0), px(2.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.24, 0.08, 0.38),
                    offset: gpui::point(px(1.0), px(6.0)),
                    blur_radius: px(10.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.26, 0.06, 0.26),
                    offset: gpui::point(px(2.0), px(10.0)),
                    blur_radius: px(14.0),
                    spread_radius: px(-1.0),
                    inset: false,
                },
            ]
        } else {
            Vec::new()
        };

        let content_model = button_content_context(
            model,
            ButtonFamilyLook {
                focus_border: gpui::transparent_black(),
                background,
                foreground: text_color,
                muted_foreground: text_color,
                border: Some(border),
                typography: LumaTextStyle { size: 11.0, line_height: 12.0, weight: FontWeight::MEDIUM },
                font_family: "System UI".into(),
                radius: 17.0,
                padding_x: 0.0,
                padding_y: 0.0,
                gap: 0.0,
                height: 34.0,
                icon_size: 14.0,
                shadow: None,
            },
        );

        div()
            .id(model.id.clone())
            .relative()
            .w(px(66.0))
            .h(px(34.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(background)
            .border_1()
            .border_color(border)
            .rounded(px(17.0))
            .shadow(thumb_shadow)
            .text_size(px(11.0))
            .line_height(px(12.0))
            .font_weight(if selected {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::MEDIUM
            })
            .text_color(text_color)
            .child((model.content)(&content_model, _cx))
    }
}

fn render_segmented_rows<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<gpui::AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    let ControlGroupTemplateHandlers {
        item_bounds,
        item_hovers,
        item_mouse_downs,
        item_mouse_ups,
        item_mouse_up_outs,
        item_clicks,
    } = handlers;

    let mut item_bounds = item_bounds.into_iter();
    let mut item_hovers = item_hovers.into_iter();
    let mut item_mouse_downs = item_mouse_downs.into_iter();
    let mut item_mouse_ups = item_mouse_ups.into_iter();
    let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
    let mut item_clicks = item_clicks.into_iter();
    let mut rows = Vec::with_capacity(model.items.len());

    for item in &model.items {
        let Some(item_bounds) = item_bounds.next() else {
            break;
        };
        let Some(item_hover) = item_hovers.next() else {
            break;
        };
        let Some(item_mouse_down) = item_mouse_downs.next() else {
            break;
        };
        let Some(item_mouse_up) = item_mouse_ups.next() else {
            break;
        };
        let Some(item_mouse_up_out) = item_mouse_up_outs.next() else {
            break;
        };
        let Some(item_click) = item_clicks.next() else {
            break;
        };

        let render_model = ButtonRenderModel {
            id: format!("{}-{}", model.id, item.item.id()).into(),
            data: item.selected,
            content: Arc::new({
                let label = item.item.label().clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            elevation: true,
            compact: false,
            look: None,
            ..Default::default()
        };

        let mut button = button_template
            .render(&render_model, window, cx)
            .on_prepaint(move |bounds, window, cx| {
                item_bounds(&bounds, window, cx);
            })
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

        if item.enabled {
            button = button.cursor_pointer();
        } else {
            button = button.opacity(0.56);
        }

        rows.push(button.into_any_element());
    }

    rows
}
