//! Exposition of the lab's `ColorStop` and `ColorStopList` controls, including a
//! custom stop template defined here (outside the controls) to show that row
//! layout is presentation only.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use xgpui_luma::infra::ElementExt;

use crate::color_stop::template::{editor_panel, label, position_label, swatch};
use crate::color_stop::{
    ColorStop, ColorStopBuilder, ColorStopEvent, ColorStopRenderModel, ColorStopTemplate, ColorStopTemplateHandlers,
    make_color_stop_template,
};
use crate::color_stop_list::{ColorStopList, ColorStopListBuilder, ColorStopListEvent, HeaderActions};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

use super::color_exposition_common::{render_demo_card, render_demo_section};
use super::event_stream::ControlEventStream;
use super::template::render_composition_exposition;

/// Matches the Gradients inspector panel so the card previews the real fit.
const CARD_WIDTH: f32 = 320.0;

pub struct ColorStopListExposition {
    look: Arc<Look>,
    gradient: Entity<ColorStopList>,
    palette: Entity<ColorStopList>,
    numbered: Entity<ColorStopList>,
    background: Entity<ColorStop>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorStopListExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        xgpui_luma::theme::observe_theme_revision(cx, |this, cx| this.sync_look(this.look.clone(), cx)).detach();

        let srgb = |r: u8, g: u8, b: u8| ColorValue::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
        let gradient = ColorStopListBuilder::new("composition-color-stops-gradient")
            .look(look.clone())
            .positioned_stops([
                (srgb(0x07, 0x5a, 0x82), 0.0),
                (srgb(0xa7, 0x3b, 0x93), 0.5),
                (srgb(0xea, 0x81, 0x00), 1.0),
            ])
            .count_limits(2, 8)
            .header_actions(HeaderActions::ALL)
            .spawn(cx);
        let palette = ColorStopListBuilder::new("composition-color-stops-palette")
            .look(look.clone())
            .colors([srgb(0xf2, 0x6d, 0x5b), srgb(0xf6, 0xc3, 0x4a), srgb(0x5b, 0xc0, 0xbe), srgb(0x4a, 0x6f, 0xd1)])
            .count_limits(1, 8)
            .header_actions(HeaderActions { reverse: true, distribute: false, add: true })
            .spawn(cx);
        // Same control, different presentation: only the stop template changes.
        let numbered = ColorStopListBuilder::new("composition-color-stops-numbered")
            .look(look.clone())
            .positioned_stops([
                (srgb(0x07, 0x5a, 0x82), 0.0),
                (srgb(0xa7, 0x3b, 0x93), 0.5),
                (srgb(0xea, 0x81, 0x00), 1.0),
            ])
            .count_limits(2, 8)
            .header_actions(HeaderActions { reverse: false, distribute: false, add: true })
            .stop_template(numbered_stop_template())
            .spawn(cx);
        // A single ColorStop without a list (e.g. a mesh background color).
        let background = ColorStopBuilder::new("composition-color-stop-background", srgb(0x16, 0x18, 0x1d))
            .look(look.clone())
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "composition-color-stops-event-log",
                "Edit either list; ColorStopListEvent variants appear below.",
            )
        });

        let mut subscriptions: Vec<Subscription> =
            [("Gradient", &gradient), ("Palette", &palette), ("Numbered", &numbered)]
                .into_iter()
                .map(|(source, list)| {
                    let event_stream = event_stream.clone();
                    cx.subscribe(list, move |_, _, event: &ColorStopListEvent, cx| {
                        let line = format!("{source}: {}", format_event(event));
                        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                    })
                })
                .collect();
        let background_events = event_stream.clone();
        subscriptions.push(cx.subscribe(&background, move |_, _, event: &ColorStopEvent, cx| {
            let line = format!("Background: {}", format_stop_event(event));
            background_events.update(cx, |stream, cx| stream.append_line(&line, cx));
        }));

        Self { look, gradient, palette, numbered, background, event_stream, _subscriptions: subscriptions }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for list in [&self.gradient, &self.palette, &self.numbered] {
            list.update(cx, |list, cx| list.sync_look(look.clone(), cx));
        }
        self.background.update(cx, |stop, cx| stop.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ColorStopListExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let look = self.look.as_ref();
        let preview = div()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(28.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap(px(28.0))
                    .child(render_demo_section(
                        look,
                        "Gradient stops",
                        "Positioned, sorted, 2–8 stops. Click a swatch to edit its color; drag rows to move colors between stops.",
                        render_demo_card(look, CARD_WIDTH, self.gradient.clone()),
                    ))
                    .child(render_demo_section(
                        look,
                        "Palette",
                        "Unpositioned colors, as used by mesh points. Drag rows to reorder.",
                        render_demo_card(look, CARD_WIDTH, self.palette.clone()),
                    ))
                    .child(render_demo_section(
                        look,
                        "Custom stop template",
                        "The same list with a numbered row layout supplied by this page.",
                        render_demo_card(look, CARD_WIDTH, self.numbered.clone()),
                    ))
                    .child(render_demo_section(
                        look,
                        "Single ColorStop",
                        "One stop without a list. Click the swatch to edit.",
                        render_demo_card(look, CARD_WIDTH, self.background.clone()),
                    )),
            )
            .child(self.event_stream.clone());

        render_composition_exposition("color-stop-list", preview.into_any_element())
    }
}

fn format_event(event: &ColorStopListEvent) -> String {
    let hex = |color: ColorValue| {
        gpui_bridge::to_rgba(color, GamutMapping::CssLocalMinde)
            .map(|c| {
                let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                format!("#{:02X}{:02X}{:02X} a={:.2}", ch(c.r), ch(c.g), ch(c.b), c.a)
            })
            .unwrap_or_else(|_| "invalid".into())
    };
    match event {
        ColorStopListEvent::SelectionChanged(key) => format!("SelectionChanged({key:?})"),
        ColorStopListEvent::Activated(key) => format!("Activated({key})"),
        ColorStopListEvent::ColorChanged { key, color } => format!("ColorChanged {{ key: {key}, {} }}", hex(*color)),
        ColorStopListEvent::Added(key) => format!("Added({key})"),
        ColorStopListEvent::Removed(key) => format!("Removed({key})"),
        ColorStopListEvent::Reordered => "Reordered".into(),
        ColorStopListEvent::Reversed => "Reversed".into(),
        ColorStopListEvent::Distributed => "Distributed".into(),
    }
}

fn format_stop_event(event: &ColorStopEvent) -> String {
    match event {
        ColorStopEvent::ColorChanged(color) => format!("ColorChanged({})", crate::color_stop::stop_label(*color)),
        ColorStopEvent::RemoveRequested => "RemoveRequested".into(),
        ColorStopEvent::ExpandedChanged(expanded) => format!("ExpandedChanged({expanded})"),
    }
}

/// Number · swatch · hex · position · remove, editor below. Uses the stop's
/// `index` from its list; everything interactive comes from the control.
fn numbered_stop_template() -> Arc<dyn ColorStopTemplate> {
    const NUMBER_WIDTH: f32 = 18.0;
    make_color_stop_template(
        |model: ColorStopRenderModel<'_>, handlers: ColorStopTemplateHandlers, _: &mut Window, cx: &mut App| {
            let chrome = model.look.chrome();
            let number: SharedString = model.index.map(|index| (index + 1).to_string()).unwrap_or_default().into();
            let line = div()
                .w_full()
                .h(px(40.0))
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_shrink_0()
                        .w(px(NUMBER_WIDTH))
                        .typography_style(model.look.typography_scale(TextSize::Base))
                        .text_color(chrome.muted_text)
                        .child(number),
                )
                .child(swatch(&model, handlers.swatch_click, handlers.swatch_hover, cx))
                .child(label(&model))
                .children(position_label(&model))
                .children(model.remove_button);
            let editor: Option<AnyElement> =
                model.editor.map(|editor| editor_panel(model.look, editor).into_any_element());
            div()
                .relative()
                .w_full()
                .flex()
                .flex_col()
                .child(line)
                .children(editor)
                .on_prepaint(move |bounds, window, cx| (handlers.bounds)(bounds, window, cx))
        },
    )
}
