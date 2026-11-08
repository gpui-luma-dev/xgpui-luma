//! Custom button exposition — gallery mod_button pane: templates, content, reactive state.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, Window, div, prelude::*, px, rgb};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button_family::default_button_family_theme;
use xgpui_luma::controls::button::{Button, ButtonEvent, ButtonTemplate, DefaultButtonTemplate, HasPresenter};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

#[derive(Clone, Default)]
struct CounterState {
    count: usize,
}

pub struct CustomButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
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
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("custom-button").expect("custom-button catalog entry");

        let modified_button =
            Button::new("controls-doc-custom-modified").template(modified_button_template::<()>()).spawn(cx);
        let custom_button = Button::new("controls-doc-custom-content")
            .content(|model, _| {
                let color = if model.state.hovered {
                    rgb(0x4f46e5)
                } else {
                    rgb(0x3730a3)
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(xgpui_luma::infra::icon::lucide_icon(
                        LucideIcon::Check,
                        gpui::hsla(0.0, 0.0, 1.0, 1.0),
                        16.0,
                    ))
                    .child("Custom Layout")
                    .text_color(color)
            })
            .template(modified_button_template::<()>())
            .spawn(cx);
        let reactive_button = Button::new("controls-doc-custom-reactive")
            .typed(CounterState { count: 0 })
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .bg(rgb(0x4f46e5))
                            .text_color(rgb(0xffffff))
                            .px(px(6.0))
                            .rounded(px(4.0))
                            .child(model.data.count.to_string()),
                    )
                    .child("Reactive Counter")
            })
            .template(modified_button_template::<CounterState>())
            .spawn(cx);

        let standard_icon_only =
            shadcn::Button::icon_button("controls-doc-custom-std-icon-only", LucideIcon::FaceGrinning)
                .look(look.as_ref())
                .secondary()
                .spawn(cx);
        let standard_text_icon = shadcn::Button::new("controls-doc-custom-std-text-icon")
            .look(look.as_ref())
            .secondary()
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::FaceGrinning))
            })
            .spawn(cx);
        let standard_icon_text = shadcn::Button::new("controls-doc-custom-std-icon-text")
            .look(look.as_ref())
            .secondary()
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::FaceGrinning))
                    .child("Label")
            })
            .spawn(cx);

        let prominent_icon_only =
            shadcn::Button::icon_button("controls-doc-custom-prom-icon-only", LucideIcon::FaceGrinning)
                .look(look.as_ref())
                .primary()
                .spawn(cx);
        let prominent_text_icon = shadcn::Button::new("controls-doc-custom-prom-text-icon")
            .look(look.as_ref())
            .primary()
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::FaceGrinning))
            })
            .spawn(cx);
        let prominent_icon_text = shadcn::Button::new("controls-doc-custom-prom-icon-text")
            .look(look.as_ref())
            .primary()
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::FaceGrinning))
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
            entry,
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
                            let color = if model.state.hovered {
                                rgb(0x4f46e5)
                            } else {
                                rgb(0x3730a3)
                            };
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(xgpui_luma::infra::icon::lucide_icon(
                                    LucideIcon::Check,
                                    gpui::hsla(0.0, 0.0, 1.0, 1.0),
                                    16.0,
                                ))
                                .child(format!("Customized {clicks}"))
                                .text_color(color)
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

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for CustomButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_nowrap()
                        .items_stretch()
                        .gap(px(16.0))
                        .child(render_example_card(
                            "New Button (Text)",
                            "Button::new with a custom template.",
                            self.modified_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "Custom Content",
                            "Using .content() for free-form layouts.",
                            self.custom_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "Reactive Button",
                            "State-driven data (CounterState).",
                            self.reactive_button.clone(),
                            chrome,
                        )),
                )
                .child(demo_section(
                    "Standard factory layouts",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(12.0))
                        .child(self.standard_icon_only.clone())
                        .child(self.standard_text_icon.clone())
                        .child(self.standard_icon_text.clone()),
                ))
                .child(demo_section(
                    "Prominent factory layouts",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(12.0))
                        .child(self.prominent_icon_only.clone())
                        .child(self.prominent_text_icon.clone())
                        .child(self.prominent_icon_text.clone()),
                ))
                .child(self.event_stream.clone());

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                None,
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn demo_section(title: &'static str, title_color: gpui::Hsla, content: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(4.0))
        .child(section_label(title, title_color))
        .child(content)
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
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

fn render_lucide_icon(icon: LucideIcon) -> AnyElement {
    div()
        .text_size(px(16.0))
        .child(xgpui_luma::infra::icon::lucide_icon(icon, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0))
        .into_any_element()
}

fn render_example_card(
    title: &'static str,
    description: &'static str,
    control: impl IntoElement,
    chrome: xgpui_luma::theme::LumaChrome,
) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .min_h(px(168.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .p(px(16.0))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div().text_size(px(12.0)).line_height(px(17.0)).text_color(chrome.muted_text).child(description),
                ),
        )
        .child(control)
        .into_any_element()
}

fn modified_button_template<D: Clone + 'static>() -> Arc<dyn ButtonTemplate<D>> {
    let mut template = DefaultButtonTemplate::new(default_button_family_theme());

    template = template
        .with_modifier(|el, model| {
            let (background, border, foreground) = if model.state.disabled {
                (rgb(0xf3f4f6), rgb(0xe5e7eb), rgb(0x9ca3af))
            } else if model.state.pressed {
                (rgb(0x4338ca), rgb(0x312e81), rgb(0xffffff))
            } else if model.state.hovered {
                (rgb(0x6366f1), rgb(0x4f46e5), rgb(0xffffff))
            } else if model.state.focused {
                (rgb(0xeef2ff), rgb(0x4f46e5), rgb(0x3730a3))
            } else {
                (rgb(0xf5f3ff), rgb(0xc4b5fd), rgb(0x5b21b6))
            };

            model.radius_override.set(Some(999.0));

            el.bg(background).border_color(border).text_color(foreground).rounded(px(999.0)).px(px(18.0))
        })
        .with_modifier(|el, model| {
            if model.state.pressed {
                el.py(px(7.0))
            } else {
                el.py(px(8.0))
            }
        });

    Arc::new(template)
}
