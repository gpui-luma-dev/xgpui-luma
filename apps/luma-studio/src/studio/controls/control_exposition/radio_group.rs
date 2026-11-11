//! Radio group control exposition — gallery-aligned vertical, horizontal, indented, and card layouts.

use xgpui_luma::controls::radio_group::RadioGroupEventExt;
use std::sync::Arc;

use gpui::{AnyElement, Context, Div, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::ControlGroupItemElementTemplate;
use xgpui_luma::controls::radio_group::{
    self as sdk_radio_group, RadioGroup, RadioGroupEvent, RadioGroupItem, RadioGroupItemLike, SelectionMode,
};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector::{ControlGroupMotionReport, control_group_motion_source};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::theme_inspector::ThemeInspector;

const DENSITY_OPTION_COUNT: usize = 3;
const PANE_EXAMPLE_GAP: f32 = 24.0;
const EXAMPLE_CONTENT_GAP: f32 = 8.0;
const EXAMPLE_BORDER_RADIUS: f32 = 6.0;
const EXAMPLE_BORDER_WIDTH: f32 = 1.0;
const EXAMPLE_BORDER_OPACITY: f32 = 0.24;
const EXAMPLE_PADDING_X: f32 = 12.0;
const EXAMPLE_PADDING_Y: f32 = 12.0;
const INDENTED_EXAMPLE_PADDING_Y: f32 = 6.0;
const INDENTED_BUTTON_INDENT_STEP: f32 = 24.0;
const INDENTED_BUTTON_VERTICAL_OVERLAP: f32 = 10.0;
const FIRST_INDENTED_BUTTON_INDEX: usize = 0;
const EXAMPLE_LABEL_TEXT_SIZE: f32 = 12.0;
const EXAMPLE_LABEL_LINE_HEIGHT: f32 = 16.0;
const CHOICE_TEXT_SIZE: f32 = 12.0;
const CHOICE_LINE_HEIGHT: f32 = 16.0;
const DELIVERY_OPTION_COUNT: usize = 4;
const DELIVERY_GROUP_GAP: f32 = 4.0;
const DELIVERY_CARD_MIN_WIDTH: f32 = 116.0;
const DELIVERY_CARD_HEIGHT: f32 = 80.0;
const DELIVERY_CARD_RADIUS: f32 = 8.0;
const DELIVERY_CARD_PADDING_X: f32 = 12.0;
const DELIVERY_DAY_TEXT_SIZE: f32 = 16.0;
const DELIVERY_DAY_LINE_HEIGHT: f32 = 22.0;
const DELIVERY_DATE_TEXT_SIZE: f32 = 14.0;
const DELIVERY_DATE_LINE_HEIGHT: f32 = 20.0;
const DELIVERY_FOCUS_BORDER_WIDTH: f32 = 2.0;
const DELIVERY_DISABLED_OPACITY: f32 = 0.56;

pub struct RadioGroupControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<RadioGroupExpositionLeftPane>,
    theme_inspector: Entity<ThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct RadioGroupExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    vertical_group: RadioGroup<RadioGroupItem>,
    horizontal_group: RadioGroup<RadioGroupItem>,
    indented_group: RadioGroup<RadioGroupItem>,
    delivery_group: RadioGroup<DeliveryWindowItem>,
    vertical_choice: String,
    horizontal_choice: String,
    indented_choice: String,
    delivery_choice: String,
    event_stream: Entity<ControlEventStream>,
}

impl RadioGroupControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("radio-group").expect("radio-group catalog entry");

        let secondary_vertical = look.radio_group_template(ShadcnButtonStyle::Secondary);
        let secondary_horizontal = look.radio_group_horizontal_template(ShadcnButtonStyle::Secondary);
        let radio_template = look.radio_button_template(ShadcnButtonStyle::Secondary);

        let vertical_group = shadcn::RadioGroup::new("controls-doc-radio-group-vertical")
            .look(look.as_ref())
            .template(secondary_vertical)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let horizontal_group = shadcn::RadioGroup::horizontal("controls-doc-radio-group-horizontal")
            .look(look.as_ref())
            .template(secondary_horizontal)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let indented_group = sdk_radio_group::new("controls-doc-radio-group-indented")
            .items(density_items())
            .item_element_template(sdk_radio_group::radio_group_button_item_element_template(radio_template))
            .with_item_layout(|mut items, _, _, _| {
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .child(indented_option(0, items.take_index(0)))
                    .child(indented_option(1, items.take_index(1)))
                    .child(indented_option(2, items.take_index(2)))
            })
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let delivery_group = sdk_radio_group::new("controls-doc-radio-group-delivery")
            .items(delivery_window_items())
            .item_element_template(delivery_window_item_element_template(look.clone()))
            .with_item_layout(|items, _, _, _| {
                div().flex().items_center().gap(px(DELIVERY_GROUP_GAP)).children(items.into_elements())
            })
            .selected(DeliveryWindow::Today.id())
            .selection_mode(SelectionMode::SingleAllowNone)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-radio-group-event-log",
                "Select radio group items; RadioGroupEvent variants from the vertical group appear below.",
            )
        });

        let left_pane = cx.new(|_| RadioGroupExpositionLeftPane {
            look: look.clone(),
            entry,
            vertical_group: vertical_group.clone(),
            horizontal_group: horizontal_group.clone(),
            indented_group: indented_group.clone(),
            delivery_group: delivery_group.clone(),
            vertical_choice: Density::Comfortable.id().to_string(),
            horizontal_choice: Density::Comfortable.id().to_string(),
            indented_choice: Density::Comfortable.id().to_string(),
            delivery_choice: DeliveryWindow::Today.id().to_string(),
            event_stream: event_stream.clone(),
        });
        wire_group(cx, left_pane.clone(), &vertical_group, "vertical", GroupTarget::Vertical, event_stream.clone());
        wire_group(
            cx,
            left_pane.clone(),
            &horizontal_group,
            "horizontal",
            GroupTarget::Horizontal,
            event_stream.clone(),
        );
        wire_group(cx, left_pane.clone(), &indented_group, "indented", GroupTarget::Indented, event_stream.clone());
        wire_group(cx, left_pane.clone(), &delivery_group, "delivery", GroupTarget::Delivery, event_stream);

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-radio-group-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::RADIO_GROUP_INSPECTOR_SPEC,
            super::inspector::MotionOnlyInspector::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(ControlGroupMotionReport::new([
                    ("Vertical default", control_group_motion_source(vertical_group.read(cx).inner_group())),
                    ("Horizontal default", control_group_motion_source(horizontal_group.read(cx).inner_group())),
                    ("Horizontal custom indent", control_group_motion_source(indented_group.read(cx).inner_group())),
                    ("Delivery window", control_group_motion_source(delivery_group.read(cx).inner_group())),
                ])),
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

impl RadioGroupExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for group in [&self.vertical_group, &self.horizontal_group, &self.indented_group] {
            group.update(cx, |_, cx| cx.notify());
        }
        self.delivery_group.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum GroupTarget {
    Vertical,
    Horizontal,
    Indented,
    Delivery,
}

fn wire_group<T: RadioGroupItemLike + 'static>(
    cx: &mut Context<RadioGroupControlExposition>,
    pane: Entity<RadioGroupExpositionLeftPane>,
    group: &RadioGroup<T>,
    label: &'static str,
    target: GroupTarget,
    event_stream: Entity<ControlEventStream>,
) {
    group.clone().on_event(cx, move |_this, event: &RadioGroupEvent, cx| {
        if let RadioGroupEvent::Change { value } = event {
            let choice = value.as_ref().map_or_else(|| "None".to_string(), ToString::to_string);
            pane.update(cx, |pane, cx| {
                match target {
                    GroupTarget::Vertical => pane.vertical_choice = choice,
                    GroupTarget::Horizontal => pane.horizontal_choice = choice,
                    GroupTarget::Indented => pane.indented_choice = choice,
                    GroupTarget::Delivery => pane.delivery_choice = choice,
                }
                cx.notify();
            });
        }
        if matches!(target, GroupTarget::Vertical)
            && let Some(line) = format_radio_group_event(label, event)
        {
            event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        }
    });
}

impl Render for RadioGroupControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-radio-group-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

impl Render for RadioGroupExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(PANE_EXAMPLE_GAP))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(PANE_EXAMPLE_GAP))
                        .child(render_example(
                            "Vertical default",
                            self.vertical_group.clone(),
                            &self.vertical_choice,
                            chrome.body_text,
                        ))
                        .child(render_example(
                            "Horizontal default",
                            self.horizontal_group.clone(),
                            &self.horizontal_choice,
                            chrome.body_text,
                        )),
                )
                .child(render_example_with_padding(
                    "Horizontal custom indent",
                    self.indented_group.clone(),
                    &self.indented_choice,
                    chrome.body_text,
                    INDENTED_EXAMPLE_PADDING_Y,
                ))
                .child(render_example(
                    "Delivery window",
                    self.delivery_group.clone(),
                    &self.delivery_choice,
                    chrome.body_text,
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-radio-group-left-pane")
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Density {
    Compact,
    Comfortable,
    Expanded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DeliveryWindow {
    Today,
    Wednesday,
    Thursday,
    Friday,
}

impl Density {
    fn id(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
            Self::Expanded => "expanded",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
            Self::Expanded => "Expanded",
        }
    }
}

impl DeliveryWindow {
    fn id(&self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
        }
    }

    fn day(&self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
        }
    }

    fn date(&self) -> &'static str {
        match self {
            Self::Today => "May 15",
            Self::Wednesday => "May 16",
            Self::Thursday => "May 17",
            Self::Friday => "May 18",
        }
    }
}

#[derive(Clone, Debug)]
struct DeliveryWindowItem {
    id: gpui::SharedString,
    day: gpui::SharedString,
    date: gpui::SharedString,
}

impl DeliveryWindowItem {
    fn new(
        id: impl Into<gpui::SharedString>,
        day: impl Into<gpui::SharedString>,
        date: impl Into<gpui::SharedString>,
    ) -> Self {
        Self { id: id.into(), day: day.into(), date: date.into() }
    }
}

impl RadioGroupItemLike for DeliveryWindowItem {
    fn id(&self) -> &gpui::SharedString {
        &self.id
    }

    fn label(&self) -> &gpui::SharedString {
        &self.day
    }
}

struct DeliveryWindowTheme {
    background: gpui::Hsla,
    foreground: gpui::Hsla,
    muted_foreground: gpui::Hsla,
    selected_background: gpui::Hsla,
    selected_foreground: gpui::Hsla,
    focus_ring: gpui::Hsla,
}

impl DeliveryWindowTheme {
    fn resolve(look: &ShadcnLook) -> Self {
        let chrome = look.chrome();
        Self {
            background: theme_color(look, "card", chrome.panel_background),
            foreground: theme_color(look, "card-foreground", chrome.body_text),
            muted_foreground: theme_color(look, "muted-foreground", chrome.muted_text),
            selected_background: theme_color(look, "primary", theme_color(look, "accent", chrome.panel_background)),
            selected_foreground: theme_color(
                look,
                "primary-foreground",
                theme_color(look, "accent-foreground", chrome.title_text),
            ),
            focus_ring: theme_color(look, "ring", chrome.border),
        }
    }
}

fn theme_color(look: &ShadcnLook, token: &str, fallback: gpui::Hsla) -> gpui::Hsla {
    look.token_color(token).unwrap_or(fallback)
}

fn density_items() -> [RadioGroupItem; DENSITY_OPTION_COUNT] {
    [
        RadioGroupItem::new(Density::Compact.id()).label(Density::Compact.label()),
        RadioGroupItem::new(Density::Comfortable.id()).label(Density::Comfortable.label()),
        RadioGroupItem::new(Density::Expanded.id()).label(Density::Expanded.label()),
    ]
}

fn delivery_window_items() -> [DeliveryWindowItem; DELIVERY_OPTION_COUNT] {
    [
        DeliveryWindowItem::new(DeliveryWindow::Today.id(), DeliveryWindow::Today.day(), DeliveryWindow::Today.date()),
        DeliveryWindowItem::new(
            DeliveryWindow::Wednesday.id(),
            DeliveryWindow::Wednesday.day(),
            DeliveryWindow::Wednesday.date(),
        ),
        DeliveryWindowItem::new(
            DeliveryWindow::Thursday.id(),
            DeliveryWindow::Thursday.day(),
            DeliveryWindow::Thursday.date(),
        ),
        DeliveryWindowItem::new(
            DeliveryWindow::Friday.id(),
            DeliveryWindow::Friday.day(),
            DeliveryWindow::Friday.date(),
        ),
    ]
}

fn delivery_window_card(
    item: &DeliveryWindowItem,
    selected: bool,
    state: xgpui_luma::infra::state::CompositeItemState,
    theme: &DeliveryWindowTheme,
) -> Div {
    let day = &item.day;
    let date = &item.date;
    let foreground = if selected {
        theme.selected_foreground
    } else {
        theme.foreground
    };
    let date_color = if selected {
        theme.selected_foreground
    } else {
        theme.muted_foreground
    };

    let card = div()
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .min_w(px(DELIVERY_CARD_MIN_WIDTH))
        .h(px(DELIVERY_CARD_HEIGHT))
        .px(px(DELIVERY_CARD_PADDING_X))
        .rounded(px(DELIVERY_CARD_RADIUS))
        .bg(if selected {
            theme.selected_background
        } else {
            theme.background
        })
        .text_color(foreground)
        .child(
            div()
                .text_size(px(DELIVERY_DAY_TEXT_SIZE))
                .line_height(px(DELIVERY_DAY_LINE_HEIGHT))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(day.clone()),
        )
        .child(
            div()
                .text_size(px(DELIVERY_DATE_TEXT_SIZE))
                .line_height(px(DELIVERY_DATE_LINE_HEIGHT))
                .text_color(date_color)
                .child(date.clone()),
        );

    let card = card.child(
        xgpui_luma::infra::focus_adorner::FocusAdorner::new(xgpui_luma::theme::FocusMetricTokens {
            gap: 0.0,
            width: DELIVERY_FOCUS_BORDER_WIDTH,
            offset: 0.0,
        })
        .overlay(theme.focus_ring, state.active && !state.disabled, DELIVERY_CARD_RADIUS, true),
    );

    if !state.disabled {
        card.cursor_pointer()
    } else {
        card.opacity(DELIVERY_DISABLED_OPACITY)
    }
}

fn indented_option(index: usize, option: AnyElement) -> AnyElement {
    div()
        .pl(px(index as f32 * INDENTED_BUTTON_INDENT_STEP))
        .when(index > FIRST_INDENTED_BUTTON_INDEX, |row| row.mt(px(-INDENTED_BUTTON_VERTICAL_OVERLAP)))
        .child(option)
        .into_any_element()
}

fn delivery_window_item_element_template(look: Arc<ShadcnLook>) -> ControlGroupItemElementTemplate<DeliveryWindowItem> {
    Arc::new(move |item, _, _, _| {
        let theme = DeliveryWindowTheme::resolve(&look);
        delivery_window_card(item.item, item.selected, item.state, &theme).id(format!(
            "{}-{}",
            item.group_id,
            item.item.id()
        ))
    })
}

fn render_example<T>(label: &'static str, group: RadioGroup<T>, choice: &str, text_color: gpui::Hsla) -> AnyElement
where
    T: RadioGroupItemLike + 'static,
{
    render_example_with_padding(label, group, choice, text_color, EXAMPLE_PADDING_Y)
}

fn render_example_with_padding<T>(
    label: &'static str,
    group: RadioGroup<T>,
    choice: &str,
    text_color: gpui::Hsla,
    padding_y: f32,
) -> AnyElement
where
    T: RadioGroupItemLike + 'static,
{
    div()
        .flex()
        .flex_col()
        .flex_none()
        .items_start()
        .gap(px(EXAMPLE_CONTENT_GAP))
        .rounded(px(EXAMPLE_BORDER_RADIUS))
        .border(px(EXAMPLE_BORDER_WIDTH))
        .border_color(text_color.opacity(EXAMPLE_BORDER_OPACITY))
        .px(px(EXAMPLE_PADDING_X))
        .py(px(padding_y))
        .child(
            div()
                .text_size(px(EXAMPLE_LABEL_TEXT_SIZE))
                .line_height(px(EXAMPLE_LABEL_LINE_HEIGHT))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label),
        )
        .child(group)
        .child(
            div()
                .text_size(px(CHOICE_TEXT_SIZE))
                .line_height(px(CHOICE_LINE_HEIGHT))
                .text_color(text_color)
                .child(format!("Choice: {choice}")),
        )
        .into_any_element()
}

fn format_radio_group_event(source: &str, event: &RadioGroupEvent) -> Option<String> {
    match event {
        RadioGroupEvent::Change { value } => {
            let value = value.as_ref().map_or_else(|| "None".to_string(), |value| format!("\"{value}\""));
            Some(format!("RadioGroupEvent::Change - {source} (value: {value})"))
        }
        RadioGroupEvent::FocusChanged { focused } => {
            Some(format!("RadioGroupEvent::FocusChanged - {source} (focused: {focused})"))
        }
        _ => None,
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn delivery_window_focus_keeps_intrinsic_card_bounds() {
    let item = DeliveryWindowItem::new("long", "A long delivery window label", "May 15");
    let theme = DeliveryWindowTheme::resolve(&ShadcnLook::built_in());
    xgpui_luma::infra::focus_adorner::testing::assert_stable_layout(move |state, selected, _, _| {
        delivery_window_card(
            &item,
            selected,
            xgpui_luma::infra::state::CompositeItemState {
                active: state.focused,
                disabled: state.disabled,
                selected,
                ..Default::default()
            },
            &theme,
        )
        .into_any_element()
    });
}
