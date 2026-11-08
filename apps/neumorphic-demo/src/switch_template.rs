use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, hsla, px, prelude::*};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::switch::SwitchData;

pub fn neumorphic_switch_template() -> Arc<dyn ButtonTemplate<SwitchData>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<SwitchData>>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(NeumorphicSwitchTemplate)).clone()
}

struct NeumorphicSwitchTemplate;

impl ButtonTemplate<SwitchData> for NeumorphicSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<SwitchData>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let progress = model.data.progress.clamp(0.0, 1.0);
        let is_on = model.data.checked;
        let state = model.state;

        let track_background = if state.disabled {
            hsla(220.0 / 360.0, 0.10, 0.84, 0.85)
        } else if is_on {
            hsla(0.0, 0.0, 0.20, if state.pressed { 0.96 } else { 0.92 })
        } else {
            hsla(220.0 / 360.0, 0.08, 0.86, if state.pressed { 0.92 } else { 0.84 })
        };

        let track_border = if is_on {
            hsla(220.0 / 360.0, 0.06, 0.18, 0.22)
        } else {
            hsla(220.0 / 360.0, 0.08, 0.74, 0.72)
        };

        let thumb_shadow = if state.disabled {
            Vec::new()
        } else {
            vec![
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.16, 0.16, 0.22),
                    offset: gpui::point(px(0.0), px(2.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.24, 0.08, 0.42),
                    offset: gpui::point(px(1.0), px(6.0)),
                    blur_radius: px(10.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.26, 0.06, 0.34),
                    offset: gpui::point(px(2.0), px(10.0)),
                    blur_radius: px(15.0),
                    spread_radius: px(-1.0),
                    inset: false,
                },
            ]
        };

        let thumb_left = 3.0 + ((31.0 - 3.0) * progress);

        let thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .top(px(2.0))
            .left(px(thumb_left))
            .size(px(24.0))
            .bg(hsla(0.0, 0.0, 0.985, 0.98))
            .border_1()
            .border_color(hsla(220.0 / 360.0, 0.08, 0.72, 0.55))
            .rounded(px(12.0))
            .shadow(thumb_shadow);

        let track = div()
            .id(model.id.clone())
            .relative()
            .w(px(58.0))
            .h(px(30.0))
            .bg(track_background)
            .border_1()
            .border_color(track_border)
            .rounded(px(11.0))
            .child(thumb);

        if state.disabled {
            track.opacity(0.56)
        } else {
            track.cursor_pointer()
        }
    }
}
