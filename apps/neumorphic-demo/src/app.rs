use xgpui_luma::controls::toggle::ToggleEventExt;
use xgpui_luma::controls::switch::SwitchEventExt;
use gpui::{Context, Render, Window, div, hsla, prelude::*, px};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::radio_group::{self, RadioGroup, RadioGroupItem};
use xgpui_luma::controls::slider::{self, Slider, SliderOrientation};
use xgpui_luma::controls::switch::{self, Switch, SwitchEvent};
use xgpui_luma::controls::toggle::{self, Toggle, ToggleEvent};
use lucide_svg_static::Icon as LucideIcon;

use crate::background::deck_background;
use crate::components::{DialSize, dial, fake_analyzer_display};
use crate::power_toggle_template::neumorphic_power_toggle_template;
use crate::radio_group_template::neumorphic_radio_group_template;
use crate::slider_template::neumorphic_slider_template;
use crate::switch_template::neumorphic_switch_template;

pub struct NeumorphicDemoApp {
    level_dial: Slider,
    width_dial: Slider,
    drive_dial: Slider,
    output_dial: Slider,
    bias_dial: Slider,
    tone_dial: Slider,
    mix_dial: Slider,
    low_slider: Slider,
    mid_slider: Slider,
    high_slider: Slider,
    air_slider: Slider,
    speed_group: RadioGroup<RadioGroupItem>,
    left_power: Toggle,
    middle_power: Toggle,
    right_power: Toggle,
    left_switch: Switch,
    middle_switch: Switch,
    right_switch: Switch,
}

impl NeumorphicDemoApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let switch_template = neumorphic_switch_template();
        let slider_template = neumorphic_slider_template();
        let power_template = neumorphic_power_toggle_template();
        let left_power = toggle::new("panel-left-power")
            .with_data(false)
            .template(power_template.clone())
            .content(|_, _| {
                xgpui_luma::infra::icon::lucide_icon(LucideIcon::Power, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0)
            })
            .spawn(cx);
        let middle_power = toggle::new("panel-middle-power")
            .with_data(false)
            .template(power_template.clone())
            .content(|_, _| {
                xgpui_luma::infra::icon::lucide_icon(LucideIcon::Power, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0)
            })
            .spawn(cx);
        let right_power = toggle::new("panel-right-power")
            .with_data(false)
            .template(power_template)
            .content(|_, _| {
                xgpui_luma::infra::icon::lucide_icon(LucideIcon::Power, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0)
            })
            .spawn(cx);
        let left_switch = switch::new("panel-left-adv")
            .with_data(true)
            .template(switch_template.clone())
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let middle_switch = switch::new("panel-middle-adv")
            .with_data(false)
            .template(switch_template.clone())
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let right_switch = switch::new("panel-right-adv")
            .with_data(true)
            .template(switch_template)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let low_slider = slider::new("panel-mid-low")
            .orientation(SliderOrientation::Vertical)
            .range(0..100)
            .step(1.0)
            .value(88.0)
            .template(slider_template.clone())
            .spawn(cx);
        let mid_slider = slider::new("panel-mid-mid")
            .orientation(SliderOrientation::Vertical)
            .range(0..100)
            .step(1.0)
            .value(62.0)
            .template(slider_template.clone())
            .spawn(cx);
        let high_slider = slider::new("panel-mid-high")
            .orientation(SliderOrientation::Vertical)
            .range(0..100)
            .step(1.0)
            .value(18.0)
            .template(slider_template.clone())
            .spawn(cx);
        let air_slider = slider::new("panel-mid-air")
            .orientation(SliderOrientation::Vertical)
            .range(0..100)
            .step(1.0)
            .value(38.0)
            .template(slider_template)
            .spawn(cx);
        let speed_group = radio_group::horizontal("panel-mid-speed")
            .template(neumorphic_radio_group_template())
            .items(speed_items())
            .selected("fast")
            .spawn(cx);

        left_power.clone().on_event(cx, |this, event: &ToggleEvent, cx| {
            this.handle_power_event(PanelSlot::Left, event, cx);
        });
        middle_power.clone().on_event(cx, |this, event: &ToggleEvent, cx| {
            this.handle_power_event(PanelSlot::Middle, event, cx);
        });
        right_power.clone().on_event(cx, |this, event: &ToggleEvent, cx| {
            this.handle_power_event(PanelSlot::Right, event, cx);
        });
        left_switch.clone().on_event(cx, |this, event: &SwitchEvent, cx| {
            this.handle_switch_event(PanelSlot::Left, event, cx);
        });
        middle_switch.clone().on_event(cx, |this, event: &SwitchEvent, cx| {
            this.handle_switch_event(PanelSlot::Middle, event, cx);
        });
        right_switch.clone().on_event(cx, |this, event: &SwitchEvent, cx| {
            this.handle_switch_event(PanelSlot::Right, event, cx);
        });

        Self {
            level_dial: dial("level", "Level", 0.33, DialSize::Large).spawn(cx),
            width_dial: dial("width", "Width", 0.65, DialSize::Medium).spawn(cx),
            drive_dial: dial("drive", "Drive", 0.20, DialSize::Medium).spawn(cx),
            output_dial: dial("output", "Output", 0.26, DialSize::Medium).spawn(cx),
            bias_dial: dial("bias", "Bias", 0.16, DialSize::Small).spawn(cx),
            tone_dial: dial("tone", "Tone", 0.30, DialSize::Small).spawn(cx),
            mix_dial: dial("mix", "Mix", 0.42, DialSize::Small).spawn(cx),
            low_slider,
            mid_slider,
            high_slider,
            air_slider,
            speed_group,
            left_power,
            middle_power,
            right_power,
            left_switch,
            middle_switch,
            right_switch,
        }
    }

    fn handle_power_event(&mut self, _slot: PanelSlot, event: &ToggleEvent, cx: &mut Context<Self>) {
        if !matches!(event, ToggleEvent::Change { .. }) {
            return;
        }

        cx.notify();
    }

    fn handle_switch_event(&mut self, _slot: PanelSlot, event: &SwitchEvent, cx: &mut Context<Self>) {
        if !matches!(event, SwitchEvent::Change { .. }) {
            return;
        }

        cx.notify();
    }
}

impl Render for NeumorphicDemoApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(deck_background())
            .flex()
            .items_center()
            .justify_center()
            .gap(px(12.0))
            .child(
                panel_shell(PanelPosition::Left).child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_between()
                        .pt(px(30.0))
                        .pb(px(34.0))
                        .child(power_toggle_row(&self.left_power))
                        .child(
                            div()
                                .flex_1()
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(44.0))
                                .child(self.level_dial.clone())
                                .child(self.width_dial.clone()),
                        )
                        .child(adv_row(&self.left_switch)),
                ),
            )
            .child(
                panel_shell(PanelPosition::Middle).child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .items_center()
                        .pt(px(30.0))
                        .pb(px(34.0))
                        .child(power_toggle_row(&self.middle_power))
                        .child(
                            div()
                                .flex_1()
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_end()
                                .gap(px(24.0))
                                .pb(px(28.0))
                                .pt(px(26.0))
                                .child(fake_analyzer_display())
                                .child(
                                    div()
                                        .w_full()
                                        .flex()
                                        .items_start()
                                        .justify_center()
                                        .gap(px(8.0))
                                        .child(slider_column("LOW", &self.low_slider))
                                        .child(slider_column("MID", &self.mid_slider))
                                        .child(slider_column("HIGH", &self.high_slider))
                                        .child(slider_column("AIR", &self.air_slider)),
                                )
                                .child(self.speed_group.clone()),
                        )
                        .child(adv_row(&self.middle_switch)),
                ),
            )
            .child(
                panel_shell(PanelPosition::Right).child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_between()
                        .pt(px(30.0))
                        .pb(px(34.0))
                        .child(power_toggle_row(&self.right_power))
                        .child(
                            div()
                                .flex_1()
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(26.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_start()
                                        .justify_center()
                                        .gap(px(12.0))
                                        .child(self.drive_dial.clone())
                                        .child(self.output_dial.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_start()
                                        .justify_center()
                                        .gap(px(16.0))
                                        .child(self.bias_dial.clone())
                                        .child(self.tone_dial.clone()),
                                )
                                .child(self.mix_dial.clone()),
                        )
                        .child(adv_row(&self.right_switch)),
                ),
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanelSlot {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanelPosition {
    Left,
    Middle,
    Right,
}

fn panel_shell(position: PanelPosition) -> gpui::Div {
    let outer_radius = px(44.0);
    let inner_radius = px(23.0);
    let lower_outer_radius = px(31.0);
    let lower_inner_radius = px(18.0);

    let panel = div().w(px(322.0)).h(px(676.0)).bg(hsla(0.0, 0.0, 1.0, 0.58)).border_1().border_color(hsla(
        220.0 / 360.0,
        0.08,
        0.85,
        0.55,
    ));

    match position {
        PanelPosition::Left => panel
            .rounded_tl(outer_radius)
            .rounded_tr(inner_radius)
            .rounded_br(lower_inner_radius)
            .rounded_bl(lower_outer_radius),
        PanelPosition::Middle => panel
            .rounded_tl(inner_radius)
            .rounded_tr(inner_radius)
            .rounded_br(lower_inner_radius)
            .rounded_bl(lower_inner_radius),
        PanelPosition::Right => panel
            .rounded_tl(inner_radius)
            .rounded_tr(outer_radius)
            .rounded_br(lower_outer_radius)
            .rounded_bl(lower_inner_radius),
    }
}

fn adv_row(switch: &Switch) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .child(
            div()
                .text_size(px(17.0))
                .line_height(px(20.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .font_features(gpui::FontFeatures(std::sync::Arc::new(vec![("smcp".into(), 1)])))
                .text_color(hsla(220.0 / 360.0, 0.08, 0.30, 0.92))
                .child("ADV"),
        )
        .child(switch.clone())
}

fn power_toggle_row(toggle: &Toggle) -> gpui::Div {
    div().w_full().flex().justify_center().child(toggle.clone())
}

fn slider_column(label: &'static str, slider: &Slider) -> gpui::Div {
    div().w(px(64.0)).flex().flex_col().items_center().gap(px(6.0)).child(slider.clone()).child(
        div()
            .text_size(px(12.0))
            .line_height(px(18.0))
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(hsla(220.0 / 360.0, 0.08, 0.28, 0.96))
            .child(label),
    )
}

fn speed_items() -> Vec<RadioGroupItem> {
    vec![
        RadioGroupItem::new("fast").label("FAST"),
        RadioGroupItem::new("slow").label("SLOW"),
        RadioGroupItem::new("auto").label("AUTO"),
    ]
}
