//! Popup menu variant template preview: every `PopupMenuVariant` across interaction states.
//!
//! Renders the SDK popup menu template with the state forced, exactly like the button and
//! checkbox matrices. The trigger carries the variant's own trigger style; the variant also
//! styles the highlighted item once the panel opens. Each variant appears twice: once on the
//! theme accent and once locked to gray, which is Radix's `data-accent-color="gray"`.

use std::sync::Arc;

use gpui::{AnyElement, App, Bounds, ClickEvent, IntoElement, MouseDownEvent, MouseUpEvent, Pixels};
use gpui::{Hsla, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button_family::ButtonSize;
use xgpui_luma::controls::popup_menu::{
    ControlFocusState, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplate, PopupMenuTemplateHandlers,
    PopupMenuTriggerStyle,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma_look_radix::{Look, LookControlExt, PopupMenuVariant, Tone, ScaleFamily};
use lucide_svg_static::Icon as LucideIcon;

use super::states::{self, StateSample};
use super::table::{TableRow, TableStyle};

const STATE_COLUMN_WIDTH: f32 = 124.0;

pub struct VariantDef {
    id: &'static str,
    label: &'static str,
    variant: PopupMenuVariant,
    tone: Tone,
    trigger_style: PopupMenuTriggerStyle,
}

/// The Radix Themes menu variant scheme, in Radix's own order.
///
/// The accent pair follows the theme accent; the gray pair paints from the theme's
/// gray palette (Radix's neutral slot), so descriptions name that palette.
pub const RADIX_VARIANTS: [VariantDef; 4] = [
    VariantDef {
        id: "solid",
        label: "Solid",
        variant: PopupMenuVariant::Solid,
        tone: Tone::Accent,
        trigger_style: PopupMenuTriggerStyle::Primary,
    },
    VariantDef {
        id: "soft",
        label: "Soft",
        variant: PopupMenuVariant::Soft,
        tone: Tone::Accent,
        trigger_style: PopupMenuTriggerStyle::Secondary,
    },
    VariantDef {
        id: "solid-gray",
        label: "Solid · Gray",
        variant: PopupMenuVariant::Solid,
        tone: Tone::Gray,
        trigger_style: PopupMenuTriggerStyle::Primary,
    },
    VariantDef {
        id: "soft-gray",
        label: "Soft · Gray",
        variant: PopupMenuVariant::Soft,
        tone: Tone::Gray,
        trigger_style: PopupMenuTriggerStyle::Secondary,
    },
];

pub fn matrix(
    look: &Arc<Look>,
    variants: &[VariantDef],
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let samples = states::samples();
    let style = TableStyle { state_column_width: STATE_COLUMN_WIDTH, ..TableStyle::new(fg, muted) };
    let accent = title_case(look.palette_label(ScaleFamily::Color));
    let gray = title_case(look.palette_label(ScaleFamily::Gray));

    let headers = samples.iter().map(|sample| states::header_cell(sample, muted)).collect();
    let rows = variants
        .iter()
        .map(|def| {
            let template = look.popup_menu_template(def.variant, def.tone);
            TableRow {
                label: SharedString::from(def.label),
                description: SharedString::from(description(def, &accent, &gray)),
                cells: samples.iter().map(|sample| state_cell(&template, def, sample, window, cx)).collect(),
            }
        })
        .collect();

    super::table::render(&style, "VARIANTS", headers, rows)
}

fn description(def: &VariantDef, accent: &str, gray: &str) -> String {
    match (def.variant, def.tone) {
        (PopupMenuVariant::Solid, Tone::Accent) => {
            format!("{accent} 9 face, contrast label")
        }
        (PopupMenuVariant::Soft, Tone::Accent) => {
            format!("{accent} 3 face, {accent} 11 label")
        }
        (PopupMenuVariant::Solid, Tone::Gray) => {
            format!("{gray} 9 face, locked neutral")
        }
        (PopupMenuVariant::Soft, Tone::Gray) => {
            format!("{gray} 3 face, locked neutral")
        }
    }
}

fn title_case(name: &str) -> String {
    super::palettes::title_case(name)
}

fn state_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    def: &VariantDef,
    sample: &StateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("style-guide-menu-{}-{}", def.id, sample.id));
    let label = SharedString::from("Menu");
    let items = items();
    let disclosure = DisclosureIcons::new(LucideIcon::ChevronUp, LucideIcon::ChevronDown);
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        content: Arc::new(|model, _| {
            div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element()
        }),
        items: &items,
        open: false,
        disclosure_progress: 0.0,
        presence: xgpui_luma::OverlayPresence::new(false, false),
        submenu_transition: None,
        highlight: None,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style: def.trigger_style,
        trigger_size: ButtonSize::Md,
        menu_size: ButtonSize::Md,
        icon_only: false,
        icon: None,
        end_icon: None,
        disclosure_icons: &disclosure,
        full_width: false,
        without_elevation: false,
        split: false,
        split_state: Default::default(),
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        enabled: !sample.state.disabled,
        focus: focus_for(sample),
        state: sample.state,
    };

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, preview_handlers(items.len()), window, cx))
        .into_any_element()
}

fn focus_for(sample: &StateSample) -> ControlFocusState {
    ControlFocusState { focused: sample.state.focused, focus_visible: sample.state.focused }
}

fn items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy-css").label("Copy as CSS"),
        MenuItem::new("copy-json").label("Copy as JSON"),
        MenuItem::new("copy-hex").label("Copy hex values"),
    ]
}

pub(super) fn preview_handlers(item_count: usize) -> PopupMenuTemplateHandlers {
    PopupMenuTemplateHandlers {
        split_faces: None,
        trigger_bounds: Box::new(noop_bounds),
        action_click: Box::new(noop_click),
        trigger_click: Box::new(noop_click),
        trigger_hover: Box::new(noop_hover),
        trigger_mouse_down: Box::new(noop_mouse_down),
        trigger_mouse_up: Box::new(noop_mouse_up),
        trigger_mouse_up_out: Box::new(noop_mouse_up),
        root_mouse_down_out: Box::new(noop_mouse_down),
        item_hovers: (0..item_count).map(|_| Box::new(noop_hover) as _).collect(),
        submenu_hovers: Vec::new(),
        item_clicks: (0..item_count).map(|_| Box::new(noop_click) as _).collect(),
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

/// Always-visible popup panes use the same controlled highlight path as live popups.
pub fn pane_preview(look: &Look, fg: Hsla) -> AnyElement {
    use xgpui_luma::controls::floating_menu::{FloatingMenuHighlight, render_floating_menu_with_submenu_hovers_and_icons};
    use xgpui_luma::infra::state::MenuPath;
    use xgpui_luma_look_radix::floating_menu_look;

    let items = [
        MenuItem::new("copy-url").label("Copy palette URL").icon(LucideIcon::Share2),
        MenuItem::separator("copy-divider"),
        MenuItem::new("copy-css").label("Copy CSS code").icon(LucideIcon::Copy).submenu([
            MenuItem::new("copy-accent").label("Copy accent scale"),
            MenuItem::separator("css-divider"),
            MenuItem::new("copy-gray").label("Copy gray scale"),
        ]),
        MenuItem::new("copy-svg").label("Copy SVG object").icon(LucideIcon::FileCode),
        MenuItem::new("unavailable").label("Unavailable").enabled(false),
    ];
    let mut panels = div().w_full().flex().flex_wrap().gap(px(24.0));
    for def in &RADIX_VARIANTS {
        let id = SharedString::from(format!("guide-popup-pane-{}", def.id));
        let palette = floating_menu_look(look, def.variant, def.tone);
        let selected = MenuPath::Root(2);
        let panel = render_floating_menu_with_submenu_hovers_and_icons(
            &id,
            &items,
            None,
            Some(selected),
            palette,
            items.iter().map(|_| Box::new(noop_hover) as _).collect(),
            Vec::new(),
            items
                .iter()
                .filter(|item| item.is_enabled() && item.submenu_items().is_empty())
                .map(|_| Box::new(noop_click) as _)
                .collect(),
            Some(FloatingMenuHighlight { from: selected, to: selected, progress: 1.0 }),
            DisclosureIcons::default(),
        )
        .block_mouse_except_scroll();
        panels = panels.child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().text_sm().text_color(fg).child(def.label))
                .child(panel),
        );
    }
    panels.into_any_element()
}
