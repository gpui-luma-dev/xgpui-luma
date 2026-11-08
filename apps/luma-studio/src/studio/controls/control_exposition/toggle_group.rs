//! Toggle group control exposition — gallery-aligned icon placement groups.

use xgpui_luma::controls::control_group::ControlGroupEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Hsla, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::icon_group::{IconGroup, IconGroupEvent, IconGroupItem};
use xgpui_luma::controls::control_group::{
    ControlGroupItemLike, ControlGroupItemTemplate, animated_toggle_button_item_template,
    make_control_group_item_template,
};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector::{ControlGroupMotionReport, control_group_motion_source};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::theme_inspector::ThemeInspector;

const STYLE_VARIANTS: [(&str, ShadcnButtonStyle); 4] = [
    ("Primary", ShadcnButtonStyle::Primary),
    ("Secondary", ShadcnButtonStyle::Secondary),
    ("Outline", ShadcnButtonStyle::Outline),
    ("Ghost", ShadcnButtonStyle::Ghost),
];

const PLACEMENT: [(&str, &str); 4] = [("top", "Top"), ("bottom", "Bottom"), ("left", "Left"), ("right", "Right")];

#[derive(Clone)]
enum DemoSelection {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone)]
struct DemoGroup {
    title: &'static str,
    group: IconGroup<IconGroupItem>,
    selection: DemoSelection,
}

pub struct ToggleGroupControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ToggleGroupExpositionLeftPane>,
    theme_inspector: Entity<ThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ToggleGroupExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demos: Vec<DemoGroup>,
    event_stream: Entity<ControlEventStream>,
}

impl ToggleGroupControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("toggle-group").expect("toggle-group catalog entry");

        let mut demos: Vec<DemoGroup> = STYLE_VARIANTS
            .iter()
            .map(|(title, style)| DemoGroup {
                title,
                group: icon_group(
                    look.clone(),
                    format!("controls-doc-{}-placement-toggle-group", title.to_ascii_lowercase()),
                    *style,
                    &["bottom"],
                    false,
                    cx,
                ),
                selection: DemoSelection::Single("Bottom".into()),
            })
            .collect();

        demos.push(DemoGroup {
            title: "Multiple · Secondary",
            group: icon_group(
                look.clone(),
                "controls-doc-edge-toggle-group",
                ShadcnButtonStyle::Secondary,
                &["top", "left"],
                true,
                cx,
            ),
            selection: DemoSelection::Multiple(vec!["Top".into(), "Left".into()]),
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toggle-group-event-log",
                "Toggle placement icons; IconGroupEvent variants from the Primary group appear below.",
            )
        });

        let left_pane = cx.new(|_| ToggleGroupExpositionLeftPane {
            look: look.clone(),
            entry,
            demos: demos.clone(),
            event_stream: event_stream.clone(),
        });
        let subscriptions = Vec::new();
        for (index, demo) in demos.iter().enumerate() {
            let group = demo.group.clone();
            let event_stream = event_stream.clone();
            let left_pane = left_pane.clone();
            group.on_event(cx, move |_this, event: &IconGroupEvent, cx| {
                left_pane.update(cx, |pane, cx| pane.apply_event(index, event, cx));
                if index == 0
                    && let Some(line) = format_icon_group_event(event)
                {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            });
        }

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-toggle-group-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::TOGGLE_GROUP_INSPECTOR_SPEC,
            super::inspector::MotionOnlyInspector::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(ControlGroupMotionReport::new(
                    demos.iter().map(|demo| (demo.title, control_group_motion_source(demo.group.clone()))),
                )),
                cx,
            );
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
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

impl ToggleGroupExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for demo in &self.demos {
            demo.group.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }

    fn apply_event(&mut self, index: usize, event: &IconGroupEvent, cx: &mut Context<Self>) {
        let Some(demo) = self.demos.get_mut(index) else {
            return;
        };
        let IconGroupEvent::Change { changed_id, selected, selected_ids, .. } = event else {
            return;
        };
        demo.selection = match &demo.selection {
            DemoSelection::Single(_) => DemoSelection::Single(if *selected {
                label_for(changed_id.as_ref())
            } else {
                "None".into()
            }),
            DemoSelection::Multiple(_) => DemoSelection::Multiple(if selected_ids.is_empty() {
                vec!["None".into()]
            } else {
                selected_ids.iter().map(|id| label_for(id.as_ref())).collect()
            }),
        };
        cx.notify();
    }
}

impl Render for ToggleGroupControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-toggle-group-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

impl Render for ToggleGroupExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.0))
                .children(
                    self.demos
                        .iter()
                        .map(|demo| demo_section(demo, chrome.body_text, chrome.muted_text).into_any_element()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-toggle-group-left-pane")
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

fn icon_group(
    look: Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    style: ShadcnButtonStyle,
    selected: &[&'static str],
    multiple: bool,
    cx: &mut Context<ToggleGroupControlExposition>,
) -> IconGroup<IconGroupItem> {
    let mut builder = shadcn::IconGroup::new(id)
        .look(look.as_ref())
        .items(items(&PLACEMENT))
        // Ghost's hover/pressed fill is intentionally the same accent treatment
        // as its selected state, so selection animation reads as a flash.
        .animated_selection(!matches!(style, ShadcnButtonStyle::Ghost))
        .item_template(theme_aware_toggle_item_template(look.clone(), style))
        .with_item_layout(move |mut items, _, _, _| {
            let group_look = look.control_group_theme().resolve_list(true);
            xgpui_luma::hstack![
                gap = 6 align = center;
                items.take("top"),
                items.take("bottom"),
                items.take("left"),
                items.take("right"),
            ]
            .relative()
            .overflow_hidden()
            .rounded_full()
            .bg(group_look.background)
            .border_1()
            .border_color(group_look.border)
            .px(px(6.0))
            .py(px(4.0))
        });
    builder = apply_selection(builder, selected, multiple);
    builder.spawn(cx)
}

fn theme_aware_toggle_item_template(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> ControlGroupItemTemplate<IconGroupItem> {
    make_control_group_item_template(move |item, window, cx| {
        let button_template = look.animated_toggle_item_template(style);
        animated_toggle_button_item_template(button_template, true, |item: &IconGroupItem| {
            xgpui_luma::infra::icon::lucide_icon(
                placement_icon(item.id().as_ref()),
                gpui::hsla(0.0, 0.0, 1.0, 1.0),
                16.0,
            )
        })(item, window, cx)
    })
}

fn apply_selection(
    builder: shadcn::IconGroup<IconGroupItem>,
    selected: &[&'static str],
    multiple: bool,
) -> shadcn::IconGroup<IconGroupItem> {
    if multiple {
        builder.multiple().selected_ids(selected.iter().copied())
    } else {
        builder.selected(selected[0])
    }
}

fn demo_section(demo: &DemoGroup, body: Hsla, muted: Hsla) -> impl IntoElement {
    let caption = match &demo.selection {
        DemoSelection::Single(value) => format!("Single: {value}"),
        DemoSelection::Multiple(values) => format!("Selected: {}", values.join(", ")),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(muted)
                .child(demo.title),
        )
        .child(demo.group.clone())
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(body).child(caption))
}

fn items(entries: &[(&'static str, &'static str)]) -> Vec<IconGroupItem> {
    entries.iter().map(|(id, label)| IconGroupItem::new(*id).label(*label)).collect()
}

fn label_for(id: &str) -> String {
    PLACEMENT
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, label)| (*label).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn placement_icon(id: &str) -> LucideIcon {
    match id {
        "top" => LucideIcon::PanelTop,
        "bottom" => LucideIcon::PanelBottom,
        "left" => LucideIcon::PanelLeft,
        "right" => LucideIcon::PanelRight,
        _ => LucideIcon::Settings,
    }
}

fn format_icon_group_event(event: &IconGroupEvent) -> Option<String> {
    match event {
        IconGroupEvent::Change { changed_id, selected, selected_ids, .. } => Some(format!(
            "IconGroupEvent::Change {{ changed_id: \"{changed_id}\", selected: {selected}, selected_ids: {selected_ids:?} }}"
        )),
        IconGroupEvent::FocusChanged { focused } => {
            Some(format!("IconGroupEvent::FocusChanged {{ focused: {focused} }}"))
        }
        _ => None,
    }
}
