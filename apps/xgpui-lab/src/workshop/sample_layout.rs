//! Lab sample layout: color-gallery card spacing with Radix colors, radii, and typography.
use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

/// Paint-only grid for legacy pages and fixed SDK preview surfaces; never captures input.
pub(super) fn ruled_background() -> impl IntoElement {
    grid_background(0.)
}

// Inset the preview grid's first lines beyond its corner radius. GPUI's
// overflow mask is rectangular, so lines on the outer edges leak into corners.
fn grid_background(offset: f32) -> impl IntoElement {
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let spacing = 24.;
            let thickness = px(1. / window.scale_factor());
            let color = gpui::rgb(0xe6e6e6);
            for column in 0..(f32::from(bounds.size.width) / spacing).ceil() as usize {
                let x = column as f32 * spacing + offset;
                if x >= f32::from(bounds.size.width) - offset {
                    continue;
                }
                let origin = bounds.origin + gpui::point(px(x), px(0.));
                window.paint_quad(gpui::fill(
                    gpui::Bounds::new(origin, gpui::size(thickness, bounds.size.height)),
                    color,
                ));
            }
            for row in 0..(f32::from(bounds.size.height) / spacing).ceil() as usize {
                let y = row as f32 * spacing + offset;
                if y >= f32::from(bounds.size.height) - offset {
                    continue;
                }
                let origin = bounds.origin + gpui::point(px(0.), px(y));
                window
                    .paint_quad(gpui::fill(gpui::Bounds::new(origin, gpui::size(bounds.size.width, thickness)), color));
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}

pub(super) fn page(id: &'static str, content: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .size_full()
        .min_w_0()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(16.))
        .child(content)
        .into_any_element()
}

pub(super) fn card(
    look: &Look,
    id: &'static str,
    title: &'static str,
    description: &'static str,
    width: f32,
    content: impl IntoElement,
) -> AnyElement {
    let chrome = look.chrome();
    div()
        .id(id)
        .debug_selector(move || id.into())
        .w(px(width))
        .max_w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(14.))
        .rounded(px(look.metrics().radius.xl))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(18.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Sm))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Xs))
                        .text_color(chrome.muted_text)
                        .child(description),
                ),
        )
        .child(div().w_full().min_w_0().child(content))
        .into_any_element()
}

/// Borderless, app-themed section for converted SDK sample pages.
pub(super) fn section(
    look: &Look,
    id: &'static str,
    title: &'static str,
    description: &'static str,
    width: f32,
    content: impl IntoElement,
) -> AnyElement {
    let chrome = look.chrome();
    div()
        .id(id)
        .debug_selector(move || id.into())
        .w(px(width))
        .max_w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(14.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Sm))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Xs))
                        .text_color(chrome.muted_text)
                        .child(description),
                ),
        )
        .child(content)
        .into_any_element()
}

/// Fixed SDK presentation surface; the paint-only grid never captures input.
pub(super) fn sdk_preview(id: &'static str, content: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.into())
        .relative()
        .w_full()
        .min_w_0()
        .rounded(px(6.))
        .bg(gpui::rgb(0xfafafa))
        .text_color(gpui::rgb(0x202020))
        .child(grid_background(12.))
        .child(div().relative().p(px(24.)).child(super::sdk_sample::isolated(content)))
        .into_any_element()
}
