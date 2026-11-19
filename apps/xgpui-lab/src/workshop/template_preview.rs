//! Template preview: a control's template rendered statically in each
//! interaction state, as in the Studio style guides. Bare SDK controls have no
//! variants, so the table has a single row.
use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, Window, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, default_button_template};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::checkbox::{CheckboxData, default_checkbox_template};
use xgpui_luma::theme::InteractionState;

use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

const ROW_LABEL_WIDTH: f32 = 96.;
const STATE_COLUMN_WIDTH: f32 = 96.;
const ROW_HEIGHT: f32 = 56.;
const PAIR_GAP: f32 = 10.;
const BUTTON_COLUMN_WIDTH: f32 = 112.;

/// The interaction-state columns, matching the Studio style guides.
fn states() -> [(&'static str, &'static str, InteractionState); 5] {
    [
        ("default", "Default", InteractionState::default()),
        ("hover", "Hover", InteractionState { hovered: true, ..Default::default() }),
        ("focused", "Focused", InteractionState { focused: true, ..Default::default() }),
        ("pressed", "Pressed", InteractionState { hovered: true, pressed: true, ..Default::default() }),
        ("disabled", "Disabled", InteractionState { disabled: true, ..Default::default() }),
    ]
}

/// A preview table: state columns and labeled rows. Bare SDK controls have no
/// variants, so rows are a single "SDK" row or the control's roles.
pub(super) fn preview_table(
    look: &Look,
    columns: &[&'static str],
    column_width: f32,
    rows: Vec<(&'static str, Vec<AnyElement>)>,
) -> AnyElement {
    let chrome = look.chrome();
    let cell = |content: AnyElement| {
        div()
            .w(px(column_width))
            .min_h(px(ROW_HEIGHT))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .child(content)
    };
    let header = div()
        .flex()
        .typography_style(look.typography_scale(TextSize::Xs))
        .text_color(chrome.muted_text)
        .child(div().w(px(ROW_LABEL_WIDTH)).flex_shrink_0())
        .children(columns.iter().map(|label| cell(div().child(*label).into_any_element())));
    let rows = rows.into_iter().map(|(label, cells)| {
        div()
            .flex()
            .items_center()
            .border_t_1()
            .border_color(chrome.border)
            .child(div().w(px(ROW_LABEL_WIDTH)).flex_shrink_0().child(label))
            .children(cells.into_iter().map(cell))
    });
    div().flex().flex_col().child(header).children(rows).into_any_element()
}

/// The SDK checkbox template, unchecked and checked, in every state.
pub(super) fn checkbox_template_preview(look: &Look, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = default_checkbox_template();
    let cells = states()
        .into_iter()
        .map(|(id, _, state)| {
            let pair = [false, true].map(|checked| {
                let suffix = if checked { "on" } else { "off" };
                render_checkbox(&template, &format!("lab-checkbox-preview-{id}-{suffix}"), checked, state, window, cx)
            });
            div().flex().items_center().gap(px(PAIR_GAP)).children(pair).into_any_element()
        })
        .collect();
    let columns = states().map(|(_, label, _)| label);
    preview_table(look, &columns, STATE_COLUMN_WIDTH, vec![("SDK", cells)])
}

/// The SDK button template in every state, for the Text and Icon roles.
pub(super) fn button_template_preview(look: &Look, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = default_button_template::<()>();
    let rows = [("Text", ButtonFamilyRole::Text), ("Icon", ButtonFamilyRole::Icon)]
        .into_iter()
        .map(|(label, role)| {
            let cells = states()
                .into_iter()
                .map(|(id, _, state)| {
                    let model = ButtonRenderModel {
                        id: SharedString::from(format!("lab-button-preview-{label}-{id}")),
                        content: Arc::new(|_, _| div().child("Button").into_any_element()),
                        icon: matches!(role, ButtonFamilyRole::Icon).then_some(LucideIcon::Plus.into()),
                        role,
                        state,
                        ..Default::default()
                    };
                    template.render(&model, window, cx).into_any_element()
                })
                .collect();
            (label, cells)
        })
        .collect();
    let columns = states().map(|(_, label, _)| label);
    preview_table(look, &columns, BUTTON_COLUMN_WIDTH, rows)
}

/// Render one checkbox through its template with a forced state, without a live control.
fn render_checkbox(
    template: &Arc<dyn ButtonTemplate<CheckboxData>>,
    id: &str,
    checked: bool,
    state: InteractionState,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let model = ButtonRenderModel {
        id: SharedString::from(id.to_owned()),
        data: CheckboxData::new(checked),
        content: Arc::new(|_, _| div().into_any_element()),
        role: ButtonFamilyRole::Icon,
        state,
        ..Default::default()
    };
    template.render(&model, window, cx).into_any_element()
}
