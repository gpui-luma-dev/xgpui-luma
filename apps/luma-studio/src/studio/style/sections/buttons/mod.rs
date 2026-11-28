#![allow(deprecated)] // Reads and builds the compatibility `ButtonFamilyLook` until its callers move to style overrides (#433).
use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::button::ButtonRenderModel;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnShadow};

use crate::studio::style::shared::button_matrix::{
    BUTTON_STYLE_VARIANTS, BUTTON_TABLE_STATE_COLUMN_WIDTH, SIZE_PREVIEW_STYLE, render_button_size_radius_matrix,
    render_icon_button_state_header_cell, shadcn_style_id,
};
use crate::studio::style::shared::samples::{ButtonStateSample, ButtonTemplateVariant};
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

pub(crate) fn render_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
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
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Buttons",
        "State and variant matrix.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_button_preview_tabbed_content(
            look.as_ref(),
            &samples,
            preview_tabs,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_button_preview_tabbed_content(
    look: &ShadcnLook,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_button_size_matrix(look, window, cx)
    } else if active_tab.as_ref() == "shadows" {
        render_button_shadow_matrix(look, window, cx)
    } else {
        render_button_template_matrix(look, samples, window, cx)
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

fn render_button_template_matrix(
    look: &ShadcnLook,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let variant = ButtonTemplateVariant::TextButton;
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome).state_column_width(BUTTON_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(BUTTON_STYLE_VARIANTS.iter().map(|row| {
        VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_button_state_sample(look, row.style, variant, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_button_size_matrix(look: &ShadcnLook, window: &mut Window, cx: &mut App) -> AnyElement {
    render_button_size_radius_matrix(look, SIZE_PREVIEW_STYLE, false, window, cx)
}

fn render_button_shadow_matrix(look: &ShadcnLook, window: &mut Window, cx: &mut App) -> AnyElement {
    const SHADOWS: [(ShadcnShadow, &str); 9] = [
        (ShadcnShadow::None, "none"),
        (ShadcnShadow::TwoXs, "2xs"),
        (ShadcnShadow::Xs, "xs"),
        (ShadcnShadow::Sm, "sm"),
        (ShadcnShadow::Default, "default"),
        (ShadcnShadow::Md, "md"),
        (ShadcnShadow::Lg, "lg"),
        (ShadcnShadow::Xl, "xl"),
        (ShadcnShadow::TwoXl, "2xl"),
    ];

    let chrome = look.chrome();
    let shadow_template = look.button_family_template::<()>(ShadcnButtonStyle::Primary, None);
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .children(SHADOWS.into_iter().map(|(shadow, label)| {
            let primary = look.clone();
            let shadows = look.shadow(shadow);
            let model = ButtonRenderModel {
                id: SharedString::from(format!("luma-studio-button-shadow-{label}")),
                data: (),
                content: ButtonTemplateVariant::TextButton.content(),
                role: ButtonFamilyRole::Text,
                state: InteractionState::default(),
                round: false,
                radius_override: std::cell::Cell::new(None),
                elevation: true,
                compact: false,
                // The shadow token under test replaces the primary face's own shadow.
                look: Some(Arc::new(move |model| {
                    let mut resolved =
                        primary.button_face(ShadcnButtonStyle::Primary, model.role, primary.size(), None, model.state);
                    resolved.shadow = (!shadows.is_empty()).then(|| shadows.clone());
                    resolved
                })),
                ..Default::default()
            };

            div()
                .w_full()
                .flex()
                .items_center()
                .gap(px(24.0))
                .child(div().w(px(100.0)).text_sm().text_color(chrome.muted_text).child(format!("shadow-{label}")))
                .child(shadow_template.render(&model, window, cx))
                .into_any_element()
        }))
        .into_any_element()
}

pub(crate) fn render_button_state_sample(
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    variant: ButtonTemplateVariant,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-button-preview-{}-{}-{}",
        shadcn_style_id(style),
        variant.id(),
        sample.id
    ));
    let template = look.button_family_template::<()>(style, None);
    let model = ButtonRenderModel {
        id,
        data: (),
        content: variant.content(),
        role: if matches!(variant, ButtonTemplateVariant::IconButton) {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        ..Default::default()
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}
