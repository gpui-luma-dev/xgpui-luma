//! Checkbox control exposition — live primary/secondary previews and event stream.

use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::checkbox_theme_inspector::CheckboxThemeInspector;
use super::inspector::StateMotionReport;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckboxVariant {
    Primary,
    Secondary,
    IndicatorOnly,
}

impl CheckboxVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::IndicatorOnly => "Indicator Only",
        }
    }
}

pub struct CheckboxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_checkbox: Checkbox,
    primary_checkbox: Checkbox,
    indicator_only_checked: Checkbox,
    indicator_only_unchecked: Checkbox,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<CheckboxExpositionLeftPane>,
    theme_inspector: Entity<CheckboxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct CheckboxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_checkbox: Checkbox,
    primary_checkbox: Checkbox,
    indicator_only_checked: Checkbox,
    indicator_only_unchecked: Checkbox,
    event_stream: Entity<ControlEventStream>,
}

impl CheckboxExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for checkbox in [
            &self.secondary_checkbox,
            &self.primary_checkbox,
            &self.indicator_only_checked,
            &self.indicator_only_unchecked,
        ] {
            checkbox.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for CheckboxExpositionLeftPane {
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
                        .child(self.secondary_checkbox.clone())
                        .child(self.primary_checkbox.clone()),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(self.look.chrome().title_text)
                                .child("Indicator-only Checkbox"),
                        )
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .gap(px(12.0))
                                .child(self.indicator_only_checked.clone())
                                .child(self.indicator_only_unchecked.clone()),
                        ),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-checkbox-left-pane")
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

impl CheckboxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("checkbox").expect("checkbox catalog entry");
        let secondary_checkbox = shadcn::Checkbox::new("controls-doc-checkbox-secondary")
            .look(look.as_ref())
            .secondary()
            .with_data(true)
            .content(|_, _| div().child("Secondary").into_any_element())
            .spawn(cx);
        let primary_checkbox = shadcn::Checkbox::new("controls-doc-checkbox-primary")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .content(|_, _| div().child("Primary").into_any_element())
            .spawn(cx);
        let indicator_only_checked = shadcn::Checkbox::new("controls-doc-checkbox-indicator-only-checked")
            .look(look.as_ref())
            .primary()
            .with_data(true)
            .indicator_only()
            .compact()
            .spawn(cx);
        let indicator_only_unchecked = shadcn::Checkbox::new("controls-doc-checkbox-indicator-only-unchecked")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .indicator_only()
            .compact()
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-checkbox-event-log",
                "Toggle the preview checkboxes; all emitted CheckboxEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| CheckboxExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_checkbox: secondary_checkbox.clone(),
            primary_checkbox: primary_checkbox.clone(),
            indicator_only_checked: indicator_only_checked.clone(),
            indicator_only_unchecked: indicator_only_unchecked.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-checkbox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::CHECKBOX_INSPECTOR_SPEC,
            super::inspector::CheckboxInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(StateMotionReport::new(
                    "State",
                    [
                        ("Secondary", secondary_checkbox.clone()),
                        ("Primary", primary_checkbox.clone()),
                        ("Indicator only, checked", indicator_only_checked.clone()),
                        ("Indicator only", indicator_only_unchecked.clone()),
                    ],
                    |checkbox| checkbox.motion_enabled(),
                    |checkbox| checkbox.motion_profile().state.clone(),
                    |look| look.motion_defaults().checkbox.state.clone(),
                )),
                cx,
            );
        });
        bind_checkbox_events(
            &secondary_checkbox,
            CheckboxVariant::Secondary,
            "controls-doc-checkbox-secondary",
            event_stream.clone(),
            cx,
        );
        bind_checkbox_events(
            &indicator_only_checked,
            CheckboxVariant::IndicatorOnly,
            "controls-doc-checkbox-indicator-only-checked",
            event_stream.clone(),
            cx,
        );
        bind_checkbox_events(
            &indicator_only_unchecked,
            CheckboxVariant::IndicatorOnly,
            "controls-doc-checkbox-indicator-only-unchecked",
            event_stream.clone(),
            cx,
        );
        bind_checkbox_events(
            &primary_checkbox,
            CheckboxVariant::Primary,
            "controls-doc-checkbox-primary",
            event_stream.clone(),
            cx,
        );

        Self {
            look,
            entry,
            secondary_checkbox,
            primary_checkbox,
            indicator_only_checked,
            indicator_only_unchecked,
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
        for checkbox in [
            &self.secondary_checkbox,
            &self.primary_checkbox,
            &self.indicator_only_checked,
            &self.indicator_only_unchecked,
        ] {
            checkbox.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for CheckboxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-checkbox-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn bind_checkbox_events(
    checkbox: &Checkbox,
    variant: CheckboxVariant,
    checkbox_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<CheckboxControlExposition>,
) {
    checkbox.clone().on_event(cx, move |_, event: &CheckboxEvent, cx| {
        let line = format!("{} - {} (\"{checkbox_id}\")", format_checkbox_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    });
}

fn format_checkbox_event(event: &CheckboxEvent) -> String {
    match event {
        CheckboxEvent::Change { checked } => format!("CheckboxEvent::Change {{ checked: {checked} }}"),
        CheckboxEvent::FocusChanged { focused } => format!("CheckboxEvent::FocusChanged {{ focused: {focused} }}"),
        CheckboxEvent::EnabledChanged { enabled } => format!("CheckboxEvent::EnabledChanged {{ enabled: {enabled} }}"),
        CheckboxEvent::HoverChanged { hovered } => format!("CheckboxEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "CheckboxEvent::(unknown)".to_string(),
    }
}
