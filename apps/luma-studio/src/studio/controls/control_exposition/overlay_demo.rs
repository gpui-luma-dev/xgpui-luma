use gpui::{AnyElement, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::grid_layout;
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma::GridTrack;
use xgpui_luma::controls::overlay_window::OverlayWindowEvent;
use xgpui_luma_look_shadcn::prelude::{LumaTypographyExt, ShadcnTextRole};
use xgpui_luma_look_shadcn::ShadcnLook;

pub(super) fn overlay_status_for(event: &OverlayWindowEvent, label: &str) -> String {
    match event {
        OverlayWindowEvent::Opened => format!("Opened {label}."),
        OverlayWindowEvent::Dismissed => format!("Dismissed {label}."),
        _ => format!("Updated {label}."),
    }
}

pub(super) fn render_overlay_status(label: &'static str, value: &str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .text_color(chrome.body_text)
                .child(value.to_string()),
        )
        .into_any_element()
}

pub(super) fn render_overlay_dialog_panel(
    look: &ShadcnLook,
    title: impl Into<SharedString>,
    header_end: Option<impl IntoElement + Clone + 'static>,
    body: impl IntoElement,
) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_overlay_dialog_header(look, title.into(), header_end))
        .child(div().w_full().min_w_0().child(body))
        .into_any_element()
}

pub(super) fn render_overlay_dialog_panel_with_footer(
    look: &ShadcnLook,
    title: impl Into<SharedString>,
    header_end: Option<impl IntoElement + Clone + 'static>,
    body: impl IntoElement,
    footer: impl IntoElement,
) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_overlay_dialog_header(look, title.into(), header_end))
        .child(div().w_full().min_w_0().child(body))
        .child(div().w_full().min_w_0().pt(px(4.0)).child(footer))
        .into_any_element()
}

fn render_overlay_dialog_header(
    look: &ShadcnLook,
    title: SharedString,
    header_end: Option<impl IntoElement + Clone + 'static>,
) -> AnyElement {
    let chrome = look.chrome();
    let header_action =
        look.resolve_content_only_button(ButtonFamilyRole::Icon, ControlSize::Md, InteractionState::default());
    let header_h = header_action.height;
    let action_w = header_action.height;

    let title_cell = div()
        .h(px(header_h))
        .min_w_0()
        .flex()
        .items_center()
        .text_color(chrome.body_text)
        .typography_style(look.typography_role(ShadcnTextRole::H4))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title);

    match header_end {
        Some(header_end) => {
            let action_cell = div()
                .h(px(header_h))
                .w(px(action_w))
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .child(header_end);

            grid_layout! {
                rows: 1,
                columns: [GridTrack::Star(1.0), GridTrack::Px(action_w)],
                gap: 8;
                [0, 0] => title_cell,
                [0, 1] => action_cell,
            }
            .into_any_element()
        }
        None => title_cell.into_any_element(),
    }
}
