use std::sync::Arc;

use gpui::{AnyElement, App, Bounds, IntoElement, Pixels, SharedString, Window, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::controls::tabs::{
    ControlFocusState as TabsControlFocusState, TabsBoundsHandler, TabsClickHandler, TabsHoverHandler, TabsItem,
    TabsItemState, TabsMouseDownHandler, TabsMouseUpHandler, TabsRenderItem, TabsRenderModel, TabsTemplate,
    TabsTemplateHandlers, TabsWidthMode,
};
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::preview_handlers::{
    input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone, Copy)]
struct TabsStateSample {
    id: &'static str,
    label: &'static str,
    active_index: usize,
    target_index: usize,
    target_state: TabsItemState,
    enabled: bool,
}

pub(crate) fn render_tabs_template_section(look: Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let template = look.tabs_template();
    let samples = tabs_state_samples();

    section_shell_with_width(
        960.0,
        "Tabs",
        "Inactive, active, hover, focus, pressed, and disabled.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_tabs_state_sample(&template, sample, chrome.muted_text, window, cx)),
                ),
            )
            .into_any_element(),
    )
}

fn render_tabs_state_sample(
    template: &Arc<dyn TabsTemplate>,
    sample: TabsStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("luma-studio-tabs-navigation-preview-{}", sample.id));
    let items = tabs_preview_tabs();
    let active_id = items.get(sample.active_index).map(TabsItem::id);
    let render_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_enabled = sample.enabled && item.is_enabled();
            let active = active_id.is_some_and(|active_id| active_id == item.id());
            let mut state = TabsItemState { selected: active, ..TabsItemState::default() };

            if index == sample.target_index {
                state = sample.target_state;
                state.selected = active;
            }

            if !item_enabled {
                state.disabled = true;
                state.hovered = false;
                state.pressed = false;
                state.active = false;
                state.focus_visible = false;
            }

            TabsRenderItem {
                id: item.id(),
                label: item.label_text(),
                trigger_kind: item.trigger_kind_value(),
                leading_accessory: item.leading_accessory_ref(),
                trailing_accessory: item.trailing_accessory_ref(),
                active,
                enabled: item_enabled,
                state,
                disclosure_progress: item.disclosure_progress(),
            }
        })
        .collect::<Vec<_>>();
    let model = TabsRenderModel {
        drag_drop: None,
        id: &id,
        size: ControlSize::Md,
        width_mode: TabsWidthMode::Intrinsic,
        items: render_items,
        active_id,
        enabled: sample.enabled,
        focus: TabsControlFocusState {
            focused: sample.target_state.active,
            focus_visible: sample.target_state.focus_visible,
        },
        indicator: None,
        indicator_motion: None,
        disclosure_icons: &xgpui_luma::infra::icon::DisclosureIcons::new(
            LucideIcon::ChevronUp,
            LucideIcon::ChevronDown,
        ),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, tabs_preview_handlers(items.len()), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn tabs_state_samples() -> [TabsStateSample; 7] {
    [
        TabsStateSample {
            id: "inactive",
            label: "Inactive",
            active_index: 1,
            target_index: 0,
            target_state: TabsItemState::default(),
            enabled: true,
        },
        TabsStateSample {
            id: "active",
            label: "Active",
            active_index: 1,
            target_index: 1,
            target_state: TabsItemState { selected: true, ..TabsItemState::default() },
            enabled: true,
        },
        TabsStateSample {
            id: "hover",
            label: "Hover",
            active_index: 1,
            target_index: 0,
            target_state: TabsItemState { hovered: true, ..TabsItemState::default() },
            enabled: true,
        },
        TabsStateSample {
            id: "focus",
            label: "Focus",
            active_index: 1,
            target_index: 1,
            target_state: TabsItemState {
                selected: true,
                active: true,
                focus_visible: true,
                ..TabsItemState::default()
            },
            enabled: true,
        },
        TabsStateSample {
            id: "pressed",
            label: "Pressed",
            active_index: 1,
            target_index: 1,
            target_state: TabsItemState {
                selected: true,
                active: true,
                hovered: true,
                pressed: true,
                focus_visible: true,
                ..TabsItemState::default()
            },
            enabled: true,
        },
        TabsStateSample {
            id: "disabled-item",
            label: "Disabled item",
            active_index: 1,
            target_index: 2,
            target_state: TabsItemState { disabled: true, ..TabsItemState::default() },
            enabled: true,
        },
        TabsStateSample {
            id: "disabled-list",
            label: "Disabled list",
            active_index: 1,
            target_index: 1,
            target_state: TabsItemState { selected: true, disabled: true, ..TabsItemState::default() },
            enabled: false,
        },
    ]
}

fn tabs_preview_handlers(count: usize) -> TabsTemplateHandlers {
    TabsTemplateHandlers {
        item_bounds: (0..count).map(|_| Box::new(input_noop_bounds) as TabsBoundsHandler).collect(),
        item_hovers: (0..count).map(|_| Box::new(input_noop_hover) as TabsHoverHandler).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(input_noop_mouse_down) as TabsMouseDownHandler).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(input_noop_click) as TabsClickHandler).collect(),
    }
}

fn input_noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn tabs_preview_tabs() -> [TabsItem; 3] {
    [
        TabsItem::new("overview").label("Overview"),
        TabsItem::new("activity").label("Activity"),
        TabsItem::new("settings").label("Settings").enabled(false),
    ]
}
