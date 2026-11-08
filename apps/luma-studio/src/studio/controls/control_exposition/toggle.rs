//! Toggle control exposition — gallery-aligned primary/secondary text and round-icon toggles.

use xgpui_luma::controls::toggle::ToggleEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::infra::icon::lucide_icon;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::toggle::{Toggle, ToggleEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::inspector::StateMotionReport;
use super::toggle_theme_inspector::ToggleThemeInspector;

pub struct ToggleControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ToggleExpositionLeftPane>,
    theme_inspector: Entity<ToggleThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ToggleExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_toggle: Toggle,
    primary_toggle: Toggle,
    secondary_round_icon_toggle: Toggle,
    primary_round_icon_toggle: Toggle,
    event_stream: Entity<ControlEventStream>,
    secondary_selected: bool,
    primary_selected: bool,
    secondary_round_icon_selected: bool,
    primary_round_icon_selected: bool,
}

impl ToggleExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for toggle in [
            &self.secondary_toggle,
            &self.primary_toggle,
            &self.secondary_round_icon_toggle,
            &self.primary_round_icon_toggle,
        ] {
            toggle.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ToggleExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.primary_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary selected: {}", self.primary_selected)),
                        )
                        .child(self.secondary_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary selected: {}", self.secondary_selected)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.primary_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary icon selected: {}", self.primary_round_icon_selected)),
                        )
                        .child(self.secondary_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary icon selected: {}", self.secondary_round_icon_selected)),
                        ),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-toggle-left-pane")
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

impl ToggleControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("toggle").expect("toggle catalog entry");

        let secondary_toggle = shadcn::Toggle::new("controls-doc-toggle-secondary")
            .look(look.as_ref())
            .secondary()
            .with_data(true)
            .content(|_, _| div().child("Secondary").into_any_element())
            .spawn(cx);
        let primary_toggle = shadcn::Toggle::new("controls-doc-toggle-primary")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .content(|_, _| div().child("Primary").into_any_element())
            .spawn(cx);
        let secondary_round_icon_toggle = shadcn::Toggle::new("controls-doc-toggle-secondary-round-icon")
            .look(look.as_ref())
            .secondary()
            .with_data(false)
            .round(true)
            .content(|model, _| round_icon_glyph(false, model.look.foreground).into_any_element())
            .spawn(cx);
        let primary_round_icon_toggle = shadcn::Toggle::new("controls-doc-toggle-primary-round-icon")
            .look(look.as_ref())
            .primary()
            .with_data(true)
            .round(true)
            .content(|model, _| round_icon_glyph(true, model.look.foreground).into_any_element())
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toggle-event-log",
                "Toggle the preview controls; ToggleEvent variants appear below.",
            )
        });

        let motion_toggles = [
            ("Primary", primary_toggle.clone()),
            ("Secondary", secondary_toggle.clone()),
            ("Primary icon", primary_round_icon_toggle.clone()),
            ("Secondary icon", secondary_round_icon_toggle.clone()),
        ];
        let left_pane = cx.new(|cx| {
            wire_toggle(cx, &secondary_toggle, "secondary", ToggleTarget::Secondary, event_stream.clone());
            wire_toggle(cx, &primary_toggle, "primary", ToggleTarget::Primary, event_stream.clone());
            wire_toggle(
                cx,
                &secondary_round_icon_toggle,
                "secondary-round-icon",
                ToggleTarget::SecondaryRoundIcon,
                event_stream.clone(),
            );
            wire_toggle(
                cx,
                &primary_round_icon_toggle,
                "primary-round-icon",
                ToggleTarget::PrimaryRoundIcon,
                event_stream.clone(),
            );

            ToggleExpositionLeftPane {
                look: look.clone(),
                entry,
                secondary_toggle,
                primary_toggle,
                secondary_round_icon_toggle,
                primary_round_icon_toggle,
                event_stream,
                secondary_selected: true,
                primary_selected: false,
                secondary_round_icon_selected: false,
                primary_round_icon_selected: true,
            }
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-toggle-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::TOGGLE_INSPECTOR_SPEC,
            super::inspector::ToggleInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(StateMotionReport::new(
                    "State",
                    motion_toggles,
                    |toggle| toggle.motion_enabled(),
                    |toggle| toggle.motion_profile().state.clone(),
                    |look| look.motion_defaults().toggle.state.clone(),
                )),
                cx,
            );
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split }
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
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum ToggleTarget {
    Secondary,
    Primary,
    SecondaryRoundIcon,
    PrimaryRoundIcon,
}

fn wire_toggle(
    cx: &mut Context<ToggleExpositionLeftPane>,
    toggle: &Toggle,
    label: &'static str,
    target: ToggleTarget,
    event_stream: Entity<ControlEventStream>,
) {
    toggle.clone().on_event(cx, move |this, event: &ToggleEvent, cx| {
        if let ToggleEvent::Change { selected } = event {
            match target {
                ToggleTarget::Secondary => this.secondary_selected = *selected,
                ToggleTarget::Primary => this.primary_selected = *selected,
                ToggleTarget::SecondaryRoundIcon => {
                    let selected = *selected;
                    this.secondary_round_icon_toggle.update(cx, |toggle, cx| {
                        toggle.set_presenter(
                            Arc::new(move |model, _| {
                                let foreground = model.look.foreground;
                                round_icon_glyph(selected, foreground).into_any_element()
                            }),
                            cx,
                        );
                    });
                    this.secondary_round_icon_selected = selected;
                }
                ToggleTarget::PrimaryRoundIcon => {
                    let selected = *selected;
                    this.primary_round_icon_toggle.update(cx, |toggle, cx| {
                        toggle.set_presenter(
                            Arc::new(move |model, _| {
                                let foreground = model.look.foreground;
                                round_icon_glyph(selected, foreground).into_any_element()
                            }),
                            cx,
                        );
                    });
                    this.primary_round_icon_selected = selected;
                }
            }
            cx.notify();
        }
        if let Some(line) = format_toggle_event(label, event) {
            event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        }
    });
}

impl Render for ToggleControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-toggle-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn format_toggle_event(source: &str, event: &ToggleEvent) -> Option<String> {
    match event {
        ToggleEvent::Change { selected } => Some(format!("ToggleEvent::Change - {source} (selected: {selected})")),
        ToggleEvent::FocusChanged { focused } => {
            Some(format!("ToggleEvent::FocusChanged - {source} (focused: {focused})"))
        }
        ToggleEvent::HoverChanged { hovered } => {
            Some(format!("ToggleEvent::HoverChanged - {source} (hovered: {hovered})"))
        }
        ToggleEvent::EnabledChanged { enabled } => {
            Some(format!("ToggleEvent::EnabledChanged - {source} (enabled: {enabled})"))
        }
        _ => None,
    }
}

fn round_icon_glyph(selected: bool, foreground: gpui::Hsla) -> impl IntoElement {
    let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
    div().text_size(px(16.0)).line_height(px(16.0)).child(lucide_icon(icon, foreground, 16.0))
}
