//! Switch control exposition — live primary/secondary previews and event stream.

use xgpui_luma::controls::switch::SwitchEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::switch::{Switch, SwitchEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::SwitchMotionReport;
use super::switch_theme_inspector::SwitchThemeInspector;
use super::template::render_control_exposition_card;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SwitchVariant {
    Primary,
    Secondary,
}

impl SwitchVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

pub struct SwitchControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_switch: Switch,
    primary_switch: Switch,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SwitchExpositionLeftPane>,
    theme_inspector: Entity<SwitchThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct SwitchExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_switch: Switch,
    primary_switch: Switch,
    event_stream: Entity<ControlEventStream>,
}

impl SwitchExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for switch in [&self.secondary_switch, &self.primary_switch] {
            switch.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SwitchExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_start()
                        .gap(px(12.0))
                        .child(self.primary_switch.clone())
                        .child(self.secondary_switch.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-switch-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl SwitchControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("switch").expect("switch catalog entry");
        let secondary_switch = shadcn::Switch::new("controls-doc-switch-secondary")
            .look(look.as_ref())
            .secondary()
            .with_data(true)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let primary_switch = shadcn::Switch::new("controls-doc-switch-primary")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-switch-event-log",
                "Toggle the preview switches; all emitted SwitchEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| SwitchExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_switch: secondary_switch.clone(),
            primary_switch: primary_switch.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-switch-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::SWITCH_INSPECTOR_SPEC,
            super::inspector::SwitchInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(SwitchMotionReport::new([
                    ("Primary", primary_switch.clone()),
                    ("Secondary", secondary_switch.clone()),
                ])),
                cx,
            );
        });
        bind_switch_events(
            &secondary_switch,
            SwitchVariant::Secondary,
            "controls-doc-switch-secondary",
            event_stream.clone(),
            cx,
        );
        bind_switch_events(
            &primary_switch,
            SwitchVariant::Primary,
            "controls-doc-switch-primary",
            event_stream.clone(),
            cx,
        );

        Self {
            look,
            entry,
            secondary_switch,
            primary_switch,
            event_stream,
            left_pane,
            theme_inspector,
            inspector_split,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for switch in [&self.secondary_switch, &self.primary_switch] {
            switch.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for SwitchControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-switch-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn bind_switch_events(
    switch: &Switch,
    variant: SwitchVariant,
    switch_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<SwitchControlExposition>,
) {
    switch.clone().on_event(cx, move |_, event: &SwitchEvent, cx| {
        let line = format!("{} - {} (\"{switch_id}\")", format_switch_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    });
}

fn format_switch_event(event: &SwitchEvent) -> String {
    match event {
        SwitchEvent::Change { on } => format!("SwitchEvent::Change {{ on: {on} }}"),
        SwitchEvent::FocusChanged { focused } => format!("SwitchEvent::FocusChanged {{ focused: {focused} }}"),
        SwitchEvent::EnabledChanged { enabled } => format!("SwitchEvent::EnabledChanged {{ enabled: {enabled} }}"),
        SwitchEvent::HoverChanged { hovered } => format!("SwitchEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "SwitchEvent::(unknown)".to_string(),
    }
}
