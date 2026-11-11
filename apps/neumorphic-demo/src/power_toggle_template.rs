use std::sync::{Arc, OnceLock};

use gpui::{App, Div, FontWeight, Stateful, Window, div, hsla, point, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, button_content_context};
use xgpui_luma::controls::button_family::ButtonFamilyLook;
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma::controls::toggle::ToggleData;

pub fn neumorphic_power_toggle_template() -> Arc<dyn ButtonTemplate<ToggleData>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<ToggleData>>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(NeumorphicPowerToggleTemplate)).clone()
}

struct NeumorphicPowerToggleTemplate;

impl ButtonTemplate<ToggleData> for NeumorphicPowerToggleTemplate {
    fn render(&self, model: &ButtonRenderModel<ToggleData>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let is_on = model.data.selected;
        let state = model.state;

        let background = if state.disabled {
            hsla(220.0 / 360.0, 0.08, 0.88, 0.72)
        } else if is_on {
            hsla(220.0 / 360.0, 0.08, 0.82, 0.98)
        } else {
            hsla(0.0, 0.0, 0.98, 0.98)
        };

        let border = if is_on {
            hsla(220.0 / 360.0, 0.10, 0.58, 0.56)
        } else {
            hsla(220.0 / 360.0, 0.08, 0.74, 0.58)
        };

        let icon_color = if state.disabled {
            hsla(220.0 / 360.0, 0.08, 0.46, 0.64)
        } else if is_on {
            hsla(220.0 / 360.0, 0.10, 0.24, 0.96)
        } else {
            hsla(220.0 / 360.0, 0.10, 0.30, 0.94)
        };

        let shell_shadow = if state.disabled || is_on {
            Vec::new()
        } else {
            vec![
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.12, 0.16, 0.16),
                    offset: point(px(0.0), px(2.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.20, 0.08, 0.26),
                    offset: point(px(1.0), px(6.0)),
                    blur_radius: px(9.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
            ]
        };

        let inner_shadow = if state.disabled || !is_on {
            Vec::new()
        } else {
            vec![gpui::BoxShadow {
                color: hsla(220.0 / 360.0, 0.10, 0.34, 0.22),
                offset: point(px(0.0), px(1.0)),
                blur_radius: px(2.0),
                spread_radius: px(0.0),
                inset: false,
            }]
        };

        let highlight = if is_on {
            hsla(0.0, 0.0, 1.0, 0.08)
        } else {
            hsla(0.0, 0.0, 1.0, 0.18)
        };

        let content_model = button_content_context(
            model,
            ButtonFamilyLook {
                focus_border: gpui::transparent_black(),
                background,
                foreground: icon_color,
                muted_foreground: icon_color,
                border: Some(border),
                typography: LumaTextStyle { size: 14.0, line_height: 16.0, weight: FontWeight::NORMAL },
                font_family: "System UI".into(),
                radius: 14.0,
                padding_x: 3.0,
                padding_y: 3.0,
                gap: 0.0,
                height: 42.0,
                icon_size: 18.0,
                shadow: None,
            },
        );

        div()
            .id(model.id.clone())
            .relative()
            .w(px(42.0))
            .h(px(42.0))
            .p(px(3.0))
            .bg(hsla(220.0 / 360.0, 0.08, if is_on { 0.84 } else { 0.90 }, if state.disabled { 0.68 } else { 0.78 }))
            .rounded(px(14.0))
            .shadow(shell_shadow)
            .child(
                div()
                    .relative()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(background)
                    .border_1()
                    .border_color(border)
                    .rounded(px(11.0))
                    .shadow(inner_shadow)
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .left(px(0.0))
                            .right(px(0.0))
                            .h(px(11.0))
                            .bg(highlight)
                            .rounded_tl(px(11.0))
                            .rounded_tr(px(11.0)),
                    )
                    .child(div().text_color(icon_color).child((model.content)(&content_model, cx))),
            )
            .when(state.disabled, |div| div.opacity(0.64))
            .when(!state.disabled, |div| div.cursor_pointer())
    }
}
