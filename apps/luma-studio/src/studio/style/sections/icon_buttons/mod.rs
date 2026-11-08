use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonTemplate, default_button_template};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::sections::buttons::render_button_state_sample;
use crate::studio::style::shared::button_matrix::{
    ICON_BUTTON_VARIANTS, SIZE_PREVIEW_STYLE, render_button_size_radius_matrix, render_icon_button_state_header_cell,
};
use crate::studio::style::shared::samples::{ButtonStateSample, ButtonTemplateVariant};
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

pub(crate) fn render_icon_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template: Arc<dyn ButtonTemplate<()>> = default_button_template();
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
        "Icon Button",
        "Icon-only button states across style variants.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_icon_button_preview_tabbed_content(
            look.as_ref(),
            &template,
            &samples,
            preview_tabs,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_icon_button_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_icon_button_size_matrix(look, template, window, cx)
    } else {
        render_icon_button_matrix(look, template, samples, window, cx)
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

fn render_icon_button_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let variant = ButtonTemplateVariant::IconButton;
    let chrome = look.chrome();

    VariantStateTable::new(VariantStateTableStyle::from_chrome(&chrome))
        .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
        .rows(ICON_BUTTON_VARIANTS.iter().map(|row| {
            VariantStateTableRow {
                label: SharedString::from(row.label),
                description: SharedString::from(row.description),
                cells: samples
                    .iter()
                    .map(|sample| render_button_state_sample(template, look, row.style, variant, sample, window, cx))
                    .collect(),
            }
        }))
        .build()
}

fn render_icon_button_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_button_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, true, window, cx)
}
