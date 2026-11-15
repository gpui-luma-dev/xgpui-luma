//! Custom button exposition — gallery mod_button pane: templates, content, reactive state.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;

use xgpui_luma::controls::button::{Button, ButtonEvent, HasPresenter};
use crate::theme::Look;
use xgpui_luma_look_radix as radix;

use lucide_svg_static::Icon as LucideIcon;

use super::radix_event_stream::RadixEventStream as ControlEventStream;

#[derive(Clone, Default)]
struct CounterState {
    count: usize,
}

pub struct CustomButtonControlExposition {
    look: Arc<Look>,
    modified_button: Entity<Button>,
    custom_button: Entity<Button>,
    reactive_button: Entity<Button<CounterState>>,
    standard_icon_only: Entity<Button>,
    standard_text_icon: Entity<Button>,
    standard_icon_text: Entity<Button>,
    prominent_icon_only: Entity<Button>,
    prominent_text_icon: Entity<Button>,
    prominent_icon_text: Entity<Button>,
    event_stream: Entity<ControlEventStream>,
    modified_clicks: usize,
    custom_clicks: usize,
}

impl CustomButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let modified_button = radix::Button::new("controls-doc-custom-modified")
            .look(&look)
            .soft()
            .radius(radix::Radius::Full)
            .label("New Button")
            .spawn(cx);
        let custom_button = radix::Button::new("controls-doc-custom-content")
            .look(&look)
            .soft()
            .radius(radix::Radius::Full)
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(xgpui_luma::infra::icon::lucide_icon(LucideIcon::Check, model.look.foreground, 16.))
                    .child("Custom Layout")
            })
            .spawn(cx);
        let reactive_button = radix::Button::new("controls-doc-custom-reactive")
            .look(&look)
            .soft()
            .radius(radix::Radius::Full)
            .typed(CounterState { count: 0 })
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(model.data.count.to_string())
                    .child("Reactive Counter")
            })
            .spawn(cx);

        let standard_icon_only = radix::Button::new("controls-doc-custom-std-icon-only")
            .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
            .icon(LucideIcon::FaceGrinning)
            .look(look.as_ref())
            .soft()
            .spawn(cx);
        let standard_text_icon = radix::Button::new("controls-doc-custom-std-text-icon")
            .look(look.as_ref())
            .soft()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::FaceGrinning, model.look.foreground))
            })
            .spawn(cx);
        let standard_icon_text = radix::Button::new("controls-doc-custom-std-icon-text")
            .look(look.as_ref())
            .soft()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::FaceGrinning, model.look.foreground))
                    .child("Label")
            })
            .spawn(cx);

        let prominent_icon_only = radix::Button::new("controls-doc-custom-prom-icon-only")
            .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
            .icon(LucideIcon::FaceGrinning)
            .look(look.as_ref())
            .solid()
            .spawn(cx);
        let prominent_text_icon = radix::Button::new("controls-doc-custom-prom-text-icon")
            .look(look.as_ref())
            .solid()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::FaceGrinning, model.look.foreground))
            })
            .spawn(cx);
        let prominent_icon_text = radix::Button::new("controls-doc-custom-prom-icon-text")
            .look(look.as_ref())
            .solid()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::FaceGrinning, model.look.foreground))
                    .child("Label")
            })
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-custom-button-event-log",
                "Click customized buttons; ButtonEvent::Click appears below.",
            )
        });

        let this = Self {
            look,
            modified_button: modified_button.clone(),
            custom_button: custom_button.clone(),
            reactive_button: reactive_button.clone(),
            standard_icon_only,
            standard_text_icon,
            standard_icon_text,
            prominent_icon_only,
            prominent_text_icon,
            prominent_icon_text,
            event_stream: event_stream.clone(),
            modified_clicks: 0,
            custom_clicks: 0,
        };

        modified_button.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &ButtonEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };

                append_button_event(&event_stream, event, cx);
                if !event.is_click() {
                    return;
                }
                this.modified_clicks += 1;
                let label = format!("New Button {}", this.modified_clicks);
                this.modified_button.update(cx, |button, cx| button.set_label(label, cx));
            }
        });
        custom_button.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &ButtonEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };

                append_button_event(&event_stream, event, cx);
                if !event.is_click() {
                    return;
                }
                this.custom_clicks += 1;
                let clicks = this.custom_clicks;
                this.custom_button.update(cx, |button, cx| {
                    button.set_presenter(
                        Arc::new(move |model, _| {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(xgpui_luma::infra::icon::lucide_icon(
                                    LucideIcon::Check,
                                    model.look.foreground,
                                    16.0,
                                ))
                                .child(format!("Customized {clicks}"))
                                .into_any_element()
                        }),
                        cx,
                    );
                });
            }
        });
        reactive_button.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &ButtonEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };

                append_button_event(&event_stream, event, cx);
                if !event.is_click() {
                    return;
                }
                this.reactive_button.update(cx, |button, cx| {
                    let mut data = button.data().clone();
                    data.count += 1;
                    button.set_data(data, cx);
                });
            }
        });

        this
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for button in [
            &self.modified_button,
            &self.custom_button,
            &self.standard_icon_only,
            &self.standard_text_icon,
            &self.standard_icon_text,
            &self.prominent_icon_only,
            &self.prominent_text_icon,
            &self.prominent_icon_text,
        ] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.reactive_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for CustomButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        use super::sample_layout::{card, page};
        page(
            "workshop-custom-button-scroll",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.))
                        .child(card(
                            &self.look,
                            "command-custom-text",
                            "Text button",
                            "Click to update the label.",
                            320.,
                            self.modified_button.clone(),
                        ))
                        .child(card(
                            &self.look,
                            "command-custom-content",
                            "Custom content",
                            "Icon and text in a Radix button.",
                            320.,
                            self.custom_button.clone(),
                        ))
                        .child(card(
                            &self.look,
                            "command-custom-counter",
                            "Reactive counter",
                            "Click to increment the count.",
                            320.,
                            self.reactive_button.clone(),
                        )),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.))
                        .child(card(
                            &self.look,
                            "command-custom-soft",
                            "Soft layouts",
                            "Icon-only and text layouts.",
                            420.,
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(12.))
                                .child(self.standard_icon_only.clone())
                                .child(self.standard_text_icon.clone())
                                .child(self.standard_icon_text.clone()),
                        ))
                        .child(card(
                            &self.look,
                            "command-custom-solid",
                            "Solid layouts",
                            "Icon-only and text layouts.",
                            420.,
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(12.))
                                .child(self.prominent_icon_only.clone())
                                .child(self.prominent_text_icon.clone())
                                .child(self.prominent_icon_text.clone()),
                        )),
                )
                .child(self.event_stream.clone()),
        )
    }
}

fn append_button_event(
    event_stream: &Entity<ControlEventStream>,
    event: &ButtonEvent,
    cx: &mut Context<CustomButtonControlExposition>,
) {
    if let ButtonEvent::Click = event {
        event_stream.update(cx, |stream, cx| stream.append_line("ButtonEvent::Click", cx));
    } else if let ButtonEvent::HoverChanged { hovered } = event {
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&format!("ButtonEvent::HoverChanged {{ hovered: {hovered} }}"), cx);
        });
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla) -> AnyElement {
    div()
        .text_size(px(16.0))
        .child(xgpui_luma::infra::icon::lucide_icon(icon, color, 16.0))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Sample, Workshop};

    #[test]
    fn command_custom_buttons_update_and_retain_counters_across_navigation_and_theme() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |workshop, cx| workshop.select("custom-button", cx));
        cx.run_until_parked();
        let sample = cx.update(|_, cx| match &workshop.read(cx).samples["custom-button"] {
            Sample::CustomButton(sample) => sample.clone(),
            _ => unreachable!(),
        });
        let (modified, custom, reactive) = cx.update(|_, cx| {
            let sample = sample.read(cx);
            (sample.modified_button.clone(), sample.custom_button.clone(), sample.reactive_button.clone())
        });
        modified.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        custom.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|window, cx| reactive.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        workshop.update(cx, |workshop, cx| {
            workshop.select("toolbar", cx);
            workshop.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            workshop.select("custom-button", cx);
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let state = sample.read(cx);
            assert_eq!(state.modified_clicks, 1);
            assert_eq!(state.custom_clicks, 1);
            assert_eq!(reactive.read(cx).data().count, 1);
            assert_eq!(workshop.read(cx).samples["custom-button"].view().entity_id(), sample.entity_id());
        });
    }
}
