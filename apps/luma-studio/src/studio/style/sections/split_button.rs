use std::sync::Arc;

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::popup_menu::{
    ControlFocusState, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplateHandlers, PopupMenuTriggerStyle,
};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::motion::overlay_presence::OverlayPresence;
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::button_matrix::render_icon_button_state_header_cell;
use crate::studio::style::shared::preview_handlers::{
    input_noop_bounds, input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

pub(crate) fn render_split_button_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    section_shell_with_width(
        960.0,
        "Split Button",
        "Primary, Secondary, Outline, and Ghost variants across interaction states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_split_button_template_preview(&look, window, cx),
    )
}

fn render_split_button_template_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let samples = [
        ButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
        ButtonStateSample {
            id: "hover",
            header: "hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "focused",
            header: "focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ];
    let variants = [
        ("Primary", PopupMenuTriggerStyle::Primary, "High emphasis actions"),
        ("Secondary", PopupMenuTriggerStyle::Secondary, "Secondary actions"),
        ("Outline", PopupMenuTriggerStyle::Outline, "Bordered actions"),
        ("Ghost", PopupMenuTriggerStyle::Ghost, "Quiet utility actions"),
    ];
    let template = look.split_button_template();
    let chrome = look.chrome();

    VariantStateTable::new(VariantStateTableStyle::from_chrome(&chrome).state_column_width(140.0))
        .row_group_label("Text")
        .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
        .rows(variants.iter().map(|(label, style, description)| {
            VariantStateTableRow {
                label: SharedString::from(*label),
                description: SharedString::from(*description),
                cells: samples
                    .iter()
                    .map(|sample| render_split_button_template_cell(&template, *style, label, sample, window, cx))
                    .collect(),
            }
        }))
        .build()
}

fn render_split_button_template_cell(
    template: &Arc<dyn xgpui_luma::controls::split_button::SplitButtonTemplate>,
    style: PopupMenuTriggerStyle,
    label: &'static str,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("style-split-template-{}-{}", label.to_lowercase(), sample.id));
    let label = SharedString::from("Text");
    let items = split_button_items();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        content: Arc::new(|model, _| {
            div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element()
        }),
        items: &items,
        open: false,
        disclosure_progress: 0.0,
        presence: OverlayPresence::new(false, false),
        submenu_transition: None,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style: style,
        trigger_size: ControlSize::Md,
        menu_size: ControlSize::Md,
        icon_only: false,
        icon: None,
        end_icon: None,
        disclosure_icons: &xgpui_luma::infra::icon::DisclosureIcons::new(
            LucideIcon::ChevronUp,
            LucideIcon::ChevronDown,
        ),
        full_width: false,
        without_elevation: false,
        split: true,
        split_state: xgpui_luma::controls::split_button::SplitButtonState {
            primary: sample.state,
            disclosure: InteractionState {
                focused: sample.state.focused,
                disabled: sample.state.disabled,
                ..Default::default()
            },
        },
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        highlight: None,
        enabled: !sample.state.disabled,
        focus: ControlFocusState::default(),
        state: sample.state,
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, split_button_template_handlers(items.len()), window, cx))
        .into_any_element()
}

fn split_button_template_handlers(item_count: usize) -> PopupMenuTemplateHandlers {
    PopupMenuTemplateHandlers {
        split_faces: None,
        trigger_bounds: Box::new(input_noop_bounds),
        action_click: Box::new(input_noop_click),
        trigger_click: Box::new(input_noop_click),
        trigger_hover: Box::new(input_noop_hover),
        trigger_mouse_down: Box::new(input_noop_mouse_down),
        trigger_mouse_up: Box::new(input_noop_mouse_up),
        trigger_mouse_up_out: Box::new(input_noop_mouse_up),
        root_mouse_down_out: Box::new(input_noop_mouse_down),
        item_hovers: (0..item_count).map(|_| Box::new(input_noop_hover) as _).collect(),
        submenu_hovers: Vec::new(),
        item_clicks: (0..item_count).map(|_| Box::new(input_noop_click) as _).collect(),
    }
}

fn split_button_items() -> [MenuItem; 3] {
    [
        MenuItem::new("save-as").label("Save as").icon(LucideIcon::Save),
        MenuItem::new("duplicate").label("Duplicate").icon(LucideIcon::Copy),
        MenuItem::new("export").label("Export").icon(LucideIcon::Download),
    ]
}
