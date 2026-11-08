use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::button::{ButtonContentContext, ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::toggle::ToggleData;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::{ButtonRadiusPreset, ShadcnButtonStyle, ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::button_matrix::{
    BUTTON_SIZES, BUTTON_STYLE_VARIANTS, BUTTON_TABLE_RADIUS_COLUMN_WIDTH, BUTTON_TABLE_SIZE_HEADER_HEIGHT,
    BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT, BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH, ICON_BUTTON_VARIANTS,
    SIZE_PREVIEW_STYLE, button_size_id, radius_label_id, render_button_radius_header_cell,
    render_icon_button_state_header_cell, shadcn_style_id, toggle_template_preview_samples,
};
use crate::studio::style::shared::icons::render_lucide_icon;
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

const TOGGLE_TEXT_TABLE_STATE_COLUMN_WIDTH: f32 = 112.0;

pub(crate) fn render_toggle_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template = look.toggle_template(ShadcnButtonStyle::Secondary);
    let samples = toggle_template_preview_samples();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Toggles",
        "Style variants across interaction states. Selected and Disabled · Selected columns show toggled on.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_toggle_preview_tabbed_content(
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

fn render_toggle_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_toggle_sizes_preview(look, template, window, cx)
    } else {
        render_toggle_template_preview(look, template, samples, window, cx)
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

fn render_toggle_template_preview(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children([
            render_toggle_text_template_matrix(look, template, samples, window, cx),
            render_toggle_icon_template_matrix(look, template, samples, window, cx),
        ])
        .into_any_element()
}

fn render_toggle_text_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome).state_column_width(TOGGLE_TEXT_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(BUTTON_STYLE_VARIANTS.iter().map(|row| {
        VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_toggle_state_sample(template, look, row.style, false, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_toggle_icon_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    VariantStateTable::new(VariantStateTableStyle::from_chrome(&chrome))
        .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
        .rows(ICON_BUTTON_VARIANTS.iter().map(|row| {
            VariantStateTableRow {
                label: SharedString::from(row.label),
                description: SharedString::from(row.description),
                cells: samples
                    .iter()
                    .map(|sample| render_toggle_state_sample(template, look, row.style, true, sample, window, cx))
                    .collect(),
            }
        }))
        .build()
}

fn render_toggle_sizes_preview(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children([
            render_toggle_text_size_matrix(look, template, window, cx),
            render_toggle_icon_size_matrix(look, template, window, cx),
        ])
        .into_any_element()
}

fn render_toggle_text_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_toggle_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, false, window, cx)
}

fn render_toggle_icon_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_toggle_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, true, window, cx)
}

fn render_toggle_size_radius_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
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
                    render_toggle_size_radius_cell(template, look, style, *size, *radius, icon_only, window, cx)
                })
                .collect(),
        }
    }))
    .build()
}

fn render_toggle_size_radius_cell(
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = true;
    let id = SharedString::from(format!(
        "luma-studio-toggle-size-radius-preview-{}-{}-{}-{}",
        shadcn_style_id(style),
        button_size_id(size),
        radius_label_id(radius),
        if icon_only { "icon" } else { "text" }
    ));
    let content: xgpui_luma::controls::button::ControlPresenter<ButtonContentContext<ToggleData>> = if icon_only {
        Arc::new(move |model, _| {
            let icon_size = button_preview_look(model).map(|look| look.icon_size).unwrap_or(16.0);
            render_lucide_icon(
                LucideIcon::Heart,
                button_preview_look(model).map(|look| look.foreground).unwrap_or_default(),
                icon_size,
            )
        })
    } else {
        let label = SharedString::from("Toggle");
        Arc::new(move |_, _| div().child(label.clone()).into_any_element())
    };
    let look_source = if icon_only {
        toggle_icon_look_for_semantic(Arc::new(look.clone()), style, size, radius, selected)
    } else {
        toggle_look_for_semantic(Arc::new(look.clone()), style, size, radius, selected)
    };
    let model = ButtonRenderModel {
        id,
        data: ToggleData::new(selected),
        content,
        role: ButtonFamilyRole::Toggle { selected },
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: Some(look_source),
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

fn toggle_selected_for_sample(sample: &ButtonStateSample) -> bool {
    matches!(sample.id, "selected" | "disabled-selected")
}

fn render_toggle_state_sample(
    template: &Arc<dyn ButtonTemplate<ToggleData>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    icon_only: bool,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = toggle_selected_for_sample(sample);
    let id = SharedString::from(format!(
        "luma-studio-toggle-preview-{}-{}-{}",
        shadcn_style_id(style),
        if icon_only { "icon" } else { "text" },
        sample.id
    ));
    let content: xgpui_luma::controls::button::ControlPresenter<ButtonContentContext<ToggleData>> = if icon_only {
        Arc::new(move |model, _| round_icon_glyph(model, selected))
    } else {
        let label = SharedString::from("Toggle");
        Arc::new(move |_, _| div().child(label.clone()).into_any_element())
    };
    let model = ButtonRenderModel {
        id,
        data: ToggleData::new(selected),
        content,
        role: ButtonFamilyRole::Toggle { selected },
        size: ButtonSize::Md,
        state: sample.state,
        round: icon_only,
        radius_override: std::cell::Cell::new(None),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: Some(if icon_only {
            toggle_icon_look_for_style(Arc::new(look.clone()), style)
        } else {
            toggle_button_look_for_style(Arc::new(look.clone()), style)
        }),
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

fn toggle_icon_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> xgpui_luma::controls::button::ButtonLookSource<ToggleData> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        xgpui_luma_look_shadcn::paint::toggle_icon_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            model.data.selected,
            model.size,
            Some(ButtonRadiusPreset::Full),
            model.state,
        )
    })
}

fn toggle_icon_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    selected: bool,
) -> xgpui_luma::controls::button::ButtonLookSource<ToggleData> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        xgpui_luma_look_shadcn::paint::toggle_icon_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            selected,
            size,
            Some(radius),
            model.state,
        )
    })
}

fn toggle_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    selected: bool,
) -> xgpui_luma::controls::button::ButtonLookSource<ToggleData> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        xgpui_luma_look_shadcn::paint::toggle_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            selected,
            size,
            Some(radius),
            model.state,
        )
    })
}

fn button_preview_look(
    model: &ButtonContentContext<ToggleData>,
) -> Option<xgpui_luma::controls::button_family::ButtonFamilyLook> {
    Some(model.look.clone())
}

fn toggle_button_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> xgpui_luma::controls::button::ButtonLookSource<ToggleData> {
    Arc::new(move |model| {
        let role = ButtonFamilyRole::Toggle { selected: model.data.selected };
        match style {
            ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(role, model.size, model.state),
            ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(role, model.size, model.state),
            ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(role, model.size, model.state),
            ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(role, model.size, model.state),
            ShadcnButtonStyle::ContentOnly => theme.as_ref().resolve_content_only_button(role, model.size, model.state),
        }
    })
}

pub(crate) fn round_icon_glyph(model: &ButtonContentContext<ToggleData>, selected: bool) -> AnyElement {
    let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
    let icon_size = model.look.icon_size;
    let color = model.look.foreground;
    render_lucide_icon(icon, color, icon_size)
}
