use std::sync::Arc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::floating_menu::{
    FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuLook, render_floating_menu,
};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::{
    ControlFocusState as PopupMenuControlFocusState, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplate,
    PopupMenuTemplateHandlers, PopupMenuTriggerStyle,
};
use xgpui_luma::infra::state::MenuPath;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::{ButtonRadiusPreset, ShadcnButtonStyle, ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::button_matrix::{render_button_radius_header_cell};
use crate::studio::style::shared::icons::render_lucide_icon;
use crate::studio::style::shared::preview_handlers::{
    input_noop_bounds, input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

type MenuPreviewScrollWheelHandler = Arc<dyn Fn(&gpui::ScrollEvent, &mut Window, &mut App) + 'static>;

pub(crate) fn render_menu_template_state_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    scroll_wheel: MenuPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab = preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("menu-trigger"));

    section_shell_with_width(
        960.0,
        "Menus",
        "Menu triggers and floating menu panels. Includes the combined Focus+Pressed trigger state.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_menus_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, scroll_wheel, window, cx),
    )
}

fn render_menus_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    scroll_wheel: MenuPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "trigger-sizes" => render_menu_trigger_size_matrix(&look, window, cx),
        "floating-menu" => render_floating_menu_template_preview(&look, scroll_wheel),
        "sizes" => render_floating_menu_size_matrix(&look, scroll_wheel),
        _ => render_menu_trigger_template_matrix(&look, window, cx),
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

const MENU_TRIGGER_TABLE_STATE_COLUMN_WIDTH: f32 = 152.0;
const MENU_TRIGGER_TABLE_RADIUS_COLUMN_WIDTH: f32 = 136.0;
const MENU_TRIGGER_TABLE_SIZE_RADIUS_ROW_HEIGHT: f32 = 55.0;
const MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH: f32 = 120.0;
const MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT: f32 = 40.0;

const MENU_TRIGGER_SIZE_PREVIEW_STYLE: PopupMenuTriggerStyle = PopupMenuTriggerStyle::Outline;

const MENU_TRIGGER_SIZES: [(ButtonSize, &str); 3] =
    [(ButtonSize::Sm, "Small"), (ButtonSize::Md, "Medium"), (ButtonSize::Lg, "Large")];

struct MenuTriggerStyleDef {
    label: &'static str,
    description: &'static str,
    style: PopupMenuTriggerStyle,
}

const MENU_TRIGGER_STYLE_VARIANTS: [MenuTriggerStyleDef; 2] = [
    MenuTriggerStyleDef {
        label: "Outline",
        description: "Bordered menu trigger",
        style: PopupMenuTriggerStyle::Outline,
    },
    MenuTriggerStyleDef { label: "Ghost", description: "Quiet menu trigger", style: PopupMenuTriggerStyle::Ghost },
];

fn menu_trigger_state_samples() -> [ButtonStateSample; 6] {
    [
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
            id: "focus-pressed",
            header: "focus-pressed",
            state: InteractionState { hovered: true, focused: true, pressed: true, ..InteractionState::default() },
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
    ]
}

fn render_menu_trigger_template_matrix(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.popup_menu_template();
    let samples = menu_trigger_state_samples();
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome).state_column_width(MENU_TRIGGER_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_menu_trigger_state_header_cell(sample, chrome.muted_text)))
    .rows(MENU_TRIGGER_STYLE_VARIANTS.iter().map(|row| {
        VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_menu_trigger_state_cell(&template, look, row.style, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_menu_trigger_size_matrix(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.popup_menu_template();
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(MENU_TRIGGER_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(MENU_TRIGGER_TABLE_SIZE_RADIUS_ROW_HEIGHT),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(MENU_TRIGGER_SIZES.iter().map(|(size, label)| {
        VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_menu_trigger_size_radius_cell(
                        &template,
                        look,
                        MENU_TRIGGER_SIZE_PREVIEW_STYLE,
                        *size,
                        *radius,
                        window,
                        cx,
                    )
                })
                .collect(),
        }
    }))
    .build()
}

fn render_floating_menu_size_matrix(look: &Arc<ShadcnLook>, scroll_wheel: MenuPreviewScrollWheelHandler) -> AnyElement {
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(280.0)
            .header_height(MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(160.0),
    )
    .row_group_label("SIZE")
    .column_headers([render_button_radius_header_cell("Menu items", chrome.muted_text)])
    .rows(MENU_TRIGGER_SIZES.iter().map(|(size, label)| VariantStateTableRow {
        label: SharedString::from(*label),
        description: SharedString::from(""),
        cells: vec![render_floating_menu_size_cell(look, *size, scroll_wheel.clone())],
    }))
    .build()
}

fn render_floating_menu_size_cell(
    look: &Arc<ShadcnLook>,
    size: ButtonSize,
    scroll_wheel: MenuPreviewScrollWheelHandler,
) -> AnyElement {
    let menu_look = floating_menu_look_for_size(look, size);
    let id = SharedString::from(format!("luma-studio-floating-menu-size-{}", menu_trigger_size_id(size)));
    let items = floating_menu_default_items();
    let cell_scroll_wheel = scroll_wheel.clone();

    div()
        .w_full()
        .h_full()
        .flex()
        .items_start()
        .justify_center()
        .py(px(8.0))
        .on_scroll(move |event, window, cx| cell_scroll_wheel(event, window, cx))
        .child(
            render_floating_menu(
                &id,
                &items,
                None,
                None,
                menu_look,
                floating_menu_noop_hovers(items.len()),
                floating_menu_noop_clicks(items.len()),
            )
            .on_scroll(move |event, window, cx| scroll_wheel(event, window, cx)),
        )
        .into_any_element()
}

fn floating_menu_look_for_size(look: &Arc<ShadcnLook>, size: ButtonSize) -> FloatingMenuLook {
    xgpui_luma_look_shadcn::paint::floating_menu_look(look.mode_tokens().as_ref(), look.mode(), size)
}

fn render_menu_trigger_state_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    _look: &ShadcnLook,
    trigger_style: PopupMenuTriggerStyle,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-menu-trigger-preview-{}-{}",
        menu_trigger_style_id(trigger_style),
        sample.id
    ));
    let label = SharedString::from("Menu");
    let items = popup_menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        content: Arc::new(|model, _| {
            gpui::div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element()
        }),
        items: &items,
        open: false,
        disclosure_progress: 0.0,
        presence: xgpui_luma::OverlayPresence::new(false, false),
        submenu_transition: None,
        highlight: None,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style,
        trigger_size: ButtonSize::Md,
        menu_size: ButtonSize::Md,
        icon_only: false,
        icon: None,
        end_icon: None,
        disclosure_icons: &xgpui_luma::infra::icon::DisclosureIcons::new(
            LucideIcon::ChevronUp,
            LucideIcon::ChevronDown,
        ),
        full_width: false,
        without_elevation: false,
        split: false,
        split_state: Default::default(),
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        enabled: !sample.state.disabled,
        focus: menu_trigger_focus_for_sample(sample),
        state: sample.state,
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .into_any_element()
}

fn render_menu_trigger_size_radius_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    look: &ShadcnLook,
    trigger_style: PopupMenuTriggerStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-menu-trigger-size-preview-{}-{}-{}",
        menu_trigger_style_id(trigger_style),
        menu_trigger_size_id(size),
        menu_trigger_radius_id(radius),
    ));
    let label = SharedString::from("Menu");
    let items = popup_menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        content: Arc::new(|model, _| {
            gpui::div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element()
        }),
        items: &items,
        open: false,
        disclosure_progress: 0.0,
        presence: xgpui_luma::OverlayPresence::new(false, false),
        submenu_transition: None,
        highlight: None,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style,
        trigger_size: size,
        menu_size: ButtonSize::Md,
        icon_only: false,
        icon: None,
        end_icon: None,
        disclosure_icons: &xgpui_luma::infra::icon::DisclosureIcons::new(
            LucideIcon::ChevronUp,
            LucideIcon::ChevronDown,
        ),
        full_width: false,
        without_elevation: false,
        split: false,
        split_state: Default::default(),
        trigger_radius_override: Some(menu_trigger_radius_px(look, size, radius)),
        open_submenu: None,
        active_path: None,
        enabled: true,
        focus: PopupMenuControlFocusState::default(),
        state: InteractionState::default(),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .into_any_element()
}

fn menu_trigger_focus_for_sample(sample: &ButtonStateSample) -> PopupMenuControlFocusState {
    match sample.id {
        "focused" | "focus-pressed" | "pressed" => PopupMenuControlFocusState { focused: true, focus_visible: true },
        _ => PopupMenuControlFocusState::default(),
    }
}

fn menu_trigger_radius_px(look: &ShadcnLook, size: ButtonSize, radius: ButtonRadiusPreset) -> f32 {
    xgpui_luma_look_shadcn::paint::button_look_semantic(
        look.mode_tokens().as_ref(),
        look.mode(),
        ShadcnButtonStyle::Outline,
        ButtonFamilyRole::Text,
        size,
        Some(radius),
        InteractionState::default(),
    )
    .radius
}

fn menu_trigger_style_id(style: PopupMenuTriggerStyle) -> &'static str {
    match style {
        PopupMenuTriggerStyle::Primary => "primary",
        PopupMenuTriggerStyle::Secondary => "secondary",
        PopupMenuTriggerStyle::Outline => "outline",
        PopupMenuTriggerStyle::Ghost => "ghost",
    }
}

fn menu_trigger_size_id(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Sm => "sm",
        ButtonSize::Md => "md",
        ButtonSize::Lg => "lg",
    }
}

fn menu_trigger_radius_id(radius: ButtonRadiusPreset) -> &'static str {
    match radius {
        ButtonRadiusPreset::None => "none",
        ButtonRadiusPreset::Small => "small",
        ButtonRadiusPreset::Medium => "medium",
        ButtonRadiusPreset::Large => "large",
        ButtonRadiusPreset::Full => "full",
    }
}

fn render_menu_trigger_state_header_cell(sample: &ButtonStateSample, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(div().text_color(muted_text).child(render_lucide_icon(
            menu_trigger_state_header_icon(sample.id),
            muted_text,
            16.0,
        )))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted_text)
                .child(menu_trigger_state_display_label(sample.header)),
        )
        .into_any_element()
}

fn menu_trigger_state_header_icon(state_id: &'static str) -> LucideIcon {
    match state_id {
        "default" => LucideIcon::House,
        "hover" => LucideIcon::MousePointer2,
        "focused" => LucideIcon::SquareDashed,
        "focus-pressed" => LucideIcon::ArrowDown,
        "pressed" => LucideIcon::ArrowDown,
        "disabled" => LucideIcon::CircleMinus,
        _ => LucideIcon::House,
    }
}

fn menu_trigger_state_display_label(header: &'static str) -> &'static str {
    match header {
        "default" => "Default",
        "hover" => "Hover",
        "focused" => "Focused",
        "focus-pressed" => "Focus+Pressed",
        "pressed" => "Pressed",
        "disabled" => "Disabled",
        _ => header,
    }
}

fn popup_menu_preview_handlers(root_count: usize, submenu_click_count: usize) -> PopupMenuTemplateHandlers {
    let click_count = root_count + submenu_click_count;

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
        item_hovers: (0..root_count).map(|_| Box::new(input_noop_hover) as _).collect(),
        submenu_hovers: Vec::new(),
        item_clicks: (0..click_count).map(|_| Box::new(input_noop_click) as _).collect(),
    }
}

fn popup_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn floating_menu_default_items() -> [MenuItem; 4] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::separator("file-actions-separator"),
        MenuItem::new("archive").label("Archive"),
    ]
}

fn floating_menu_disabled_items() -> [MenuItem; 3] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download).enabled(false),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2),
    ]
}

fn floating_menu_submenu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("inspect").label("Inspect"),
    ]
}

fn menu_sample_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn floating_menu_noop_hovers(count: usize) -> Vec<FloatingMenuHoverHandler> {
    (0..count).map(|_| Box::new(input_noop_hover) as FloatingMenuHoverHandler).collect()
}

fn floating_menu_noop_clicks(count: usize) -> Vec<FloatingMenuClickHandler> {
    (0..count).map(|_| Box::new(input_noop_click) as FloatingMenuClickHandler).collect()
}

fn render_floating_menu_template_preview(
    look: &Arc<ShadcnLook>,
    scroll_wheel: MenuPreviewScrollWheelHandler,
) -> AnyElement {
    let chrome = look.chrome();
    let menu_look = floating_menu_look_for_size(look, ButtonSize::Md);

    div()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_center()
        .gap(px(16.0))
        .child(render_floating_menu_state_sample(
            "Standard",
            &menu_look,
            chrome.muted_text,
            &floating_menu_default_items(),
            None,
            scroll_wheel.clone(),
        ))
        .child(render_floating_menu_state_sample(
            "Hover / active item",
            &menu_look,
            chrome.muted_text,
            &floating_menu_default_items(),
            Some(MenuPath::Root(1)),
            scroll_wheel.clone(),
        ))
        .child(render_floating_menu_state_sample(
            "Disabled item",
            &menu_look,
            chrome.muted_text,
            &floating_menu_disabled_items(),
            None,
            scroll_wheel.clone(),
        ))
        .child(render_floating_menu_state_sample(
            "Submenu affordance",
            &menu_look,
            chrome.muted_text,
            &floating_menu_submenu_items(),
            None,
            scroll_wheel,
        ))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_floating_menu_state_sample(
    label: &'static str,
    look: &FloatingMenuLook,
    label_color: gpui::Hsla,
    items: &[MenuItem],
    active_path: Option<MenuPath>,
    scroll_wheel: MenuPreviewScrollWheelHandler,
) -> AnyElement {
    let id = SharedString::from(format!("luma-studio-floating-menu-sample-{}", menu_sample_id(label)));
    let root_count = items.len();
    let sample_scroll_wheel = scroll_wheel.clone();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(7.0))
        .on_scroll(move |event, window, cx| sample_scroll_wheel(event, window, cx))
        .child(
            render_floating_menu(
                &id,
                items,
                None,
                active_path,
                look.clone(),
                floating_menu_noop_hovers(root_count),
                floating_menu_noop_clicks(root_count),
            )
            .on_scroll(move |event, window, cx| scroll_wheel(event, window, cx)),
        )
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}
