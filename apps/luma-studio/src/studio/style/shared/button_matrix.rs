use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::{ButtonRadiusPreset, ShadcnButtonStyle, ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::icons::render_lucide_icon;
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

pub(crate) const BUTTON_TABLE_STATE_COLUMN_WIDTH: f32 = 152.0;
pub(crate) const BUTTON_TABLE_RADIUS_COLUMN_WIDTH: f32 = 136.0;
pub(crate) const BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT: f32 = 55.0;
pub(crate) const BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH: f32 = 120.0;
pub(crate) const BUTTON_TABLE_SIZE_HEADER_HEIGHT: f32 = 40.0;

pub(crate) const SIZE_PREVIEW_STYLE: ShadcnButtonStyle = ShadcnButtonStyle::Primary;

pub(crate) const BUTTON_SIZES: [(ButtonSize, &str); 3] =
    [(ButtonSize::Sm, "Small"), (ButtonSize::Md, "Medium"), (ButtonSize::Lg, "Large")];

pub(crate) struct ButtonStyleVariantDef {
    pub label: &'static str,
    pub description: &'static str,
    pub style: ShadcnButtonStyle,
}

pub(crate) const BUTTON_STYLE_VARIANTS: [ButtonStyleVariantDef; 5] = [
    ButtonStyleVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    ButtonStyleVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
    ButtonStyleVariantDef {
        label: "Outline",
        description: "Subtle secondary controls",
        style: ShadcnButtonStyle::Outline,
    },
    ButtonStyleVariantDef { label: "Ghost", description: "Quiet utility actions", style: ShadcnButtonStyle::Ghost },
    ButtonStyleVariantDef {
        label: "Content Only",
        description: "Interactive content without chrome",
        style: ShadcnButtonStyle::ContentOnly,
    },
];

pub(crate) const CHOICE_STYLE_VARIANTS: [ButtonStyleVariantDef; 2] = [
    ButtonStyleVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    ButtonStyleVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
];

pub(crate) const CHOICE_CONTROL_STYLE_VARIANTS: [ButtonStyleVariantDef; 3] = [
    ButtonStyleVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    ButtonStyleVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
    ButtonStyleVariantDef {
        label: "Content Only",
        description: "Indicator without container chrome",
        style: ShadcnButtonStyle::ContentOnly,
    },
];

pub(crate) struct IconButtonVariantDef {
    pub label: &'static str,
    pub description: &'static str,
    pub style: ShadcnButtonStyle,
}

pub(crate) const ICON_BUTTON_VARIANTS: [IconButtonVariantDef; 5] = [
    IconButtonVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    IconButtonVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
    IconButtonVariantDef {
        label: "Outline",
        description: "Subtle secondary controls",
        style: ShadcnButtonStyle::Outline,
    },
    IconButtonVariantDef { label: "Ghost", description: "Quiet utility actions", style: ShadcnButtonStyle::Ghost },
    IconButtonVariantDef {
        label: "Content Only",
        description: "Interactive icon without chrome",
        style: ShadcnButtonStyle::ContentOnly,
    },
];

pub(crate) fn render_button_size_radius_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    style: ShadcnButtonStyle,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_button_size_radius_cell(template, look, style, *size, *radius, icon_only, window, cx)
                })
                .collect(),
        }
    }))
    .build()
}

pub(crate) fn render_button_radius_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(label)
        .into_any_element()
}

fn render_button_size_radius_cell(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-button-size-radius-preview-{}-{}-{}-{}",
        shadcn_style_id(style),
        button_size_id(size),
        radius_label_id(radius),
        if icon_only { "icon" } else { "text" }
    ));
    let content: xgpui_luma::controls::button::ControlPresenter<
        xgpui_luma::controls::button::ButtonContentContext<()>,
    > = if icon_only {
        Arc::new(move |model, _| {
            let icon_size = model.look.icon_size;
            render_lucide_icon(LucideIcon::Heart, model.look.foreground, icon_size)
        })
    } else {
        let label = SharedString::from("Button");
        Arc::new(move |_, _| div().child(label.clone()).into_any_element())
    };
    let model = ButtonRenderModel {
        id,
        data: (),
        content,
        role: if icon_only {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: Some(button_look_for_semantic(Arc::new(look.clone()), style, size, radius)),
        ..Default::default()
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

pub(crate) fn button_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
) -> xgpui_luma::controls::button::ButtonLookSource<()> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        xgpui_luma_look_shadcn::paint::button_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            model.role,
            size,
            Some(radius),
            model.state,
        )
    })
}

pub(crate) fn button_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> xgpui_luma::controls::button::ButtonLookSource<()> {
    Arc::new(move |model| match style {
        ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(model.role, model.size, model.state),
        ShadcnButtonStyle::ContentOnly => {
            theme.as_ref().resolve_content_only_button(model.role, model.size, model.state)
        }
    })
}

pub(crate) fn radius_label_id(radius: ButtonRadiusPreset) -> &'static str {
    match radius {
        ButtonRadiusPreset::None => "none",
        ButtonRadiusPreset::Small => "small",
        ButtonRadiusPreset::Medium => "medium",
        ButtonRadiusPreset::Large => "large",
        ButtonRadiusPreset::Full => "full",
    }
}

pub(crate) fn shadcn_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}

pub(crate) fn button_size_id(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Sm => "sm",
        ButtonSize::Md => "md",
        ButtonSize::Lg => "lg",
    }
}

pub(crate) fn render_icon_button_state_header_cell(sample: &ButtonStateSample, muted_text: gpui::Hsla) -> AnyElement {
    render_interaction_state_header_cell(sample.id, sample.header, muted_text)
}

pub(crate) fn render_interaction_state_header_cell(
    id: &'static str,
    header: &'static str,
    muted_text: gpui::Hsla,
) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(div().text_color(muted_text).child(render_lucide_icon(
            icon_button_state_header_icon(id),
            muted_text,
            16.0,
        )))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted_text)
                .child(icon_button_state_display_label(header)),
        )
        .into_any_element()
}

fn icon_button_state_header_icon(state_id: &'static str) -> LucideIcon {
    match state_id {
        "default" => LucideIcon::House,
        "hover" => LucideIcon::MousePointer2,
        "focused" => LucideIcon::SquareDashed,
        "focus-pressed" => LucideIcon::ArrowDown,
        "pressed" => LucideIcon::ArrowDown,
        "selected" => LucideIcon::Check,
        "disabled" => LucideIcon::CircleMinus,
        "disabled-pressed" | "disabled-selected" => LucideIcon::CircleMinus,
        _ => LucideIcon::House,
    }
}

fn icon_button_state_display_label(header: &'static str) -> &'static str {
    match header {
        "default" => "Default",
        "hover" => "Hover",
        "focused" => "Focused",
        "focus-pressed" => "Focus/Pressed",
        "pressed" => "Pressed",
        "selected" => "Selected",
        "disabled" => "Disabled",
        "disabled-pressed" => "Disabled · On",
        "disabled-selected" => "Disabled · Selected",
        _ => header,
    }
}

pub(crate) fn toggle_interaction_state_samples() -> [ButtonStateSample; 7] {
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
        ButtonStateSample {
            id: "disabled-pressed",
            header: "disabled-pressed",
            state: InteractionState { disabled: true, pressed: true, ..InteractionState::default() },
        },
    ]
}

/// Toggle template preview columns, including resting selected (on) vs interaction pressed.
pub(crate) fn toggle_template_preview_samples() -> [ButtonStateSample; 7] {
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
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        ButtonStateSample { id: "selected", header: "selected", state: InteractionState::default() },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "disabled-selected",
            header: "disabled-selected",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ]
}
