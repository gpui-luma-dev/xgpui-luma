//! Lab sample layout: color-gallery card spacing with Radix colors, radii, and typography.
use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

/// Paint-only grid for legacy pages and fixed SDK preview surfaces; never captures input.
pub(super) fn ruled_background() -> impl IntoElement {
    grid_background(GridLines { spacing: DEFAULT_GRID_SPACING, inset: 0., center: None, color: default_line_color() })
}

/// Paint-only grid of `spacing`-pixel squares with lines through the center of its bounds moved by
/// `offset` (x, y), so a grid scaled and panned with centered content (e.g. a zoomed stage) grows
/// around and moves with the same point. The lines are a step lighter or darker than `background`, so
/// they read the same in light and dark modes.
pub(super) fn centered_ruled_background(
    spacing: f32,
    offset: (f32, f32),
    background: Option<gpui::Hsla>,
) -> impl IntoElement {
    let color = background.map_or_else(default_line_color, grid_line_color);
    grid_background(GridLines { spacing, inset: 0., center: Some(offset), color })
}

/// How far grid lines sit from their background in lightness.
const GRID_LINE_STEP: f32 = 0.08;

/// Grid lines for `background`: darker on a light background, lighter on a dark one.
pub(super) fn grid_line_color(background: gpui::Hsla) -> gpui::Hsla {
    let step = if background.l > 0.5 {
        -GRID_LINE_STEP
    } else {
        GRID_LINE_STEP
    };
    gpui::Hsla { l: (background.l + step).clamp(0., 1.), a: 1., ..background }
}

/// The legacy pages' fixed light gray.
fn default_line_color() -> gpui::Hsla {
    gpui::rgb(0xe6e6e6).into()
}

const DEFAULT_GRID_SPACING: f32 = 24.;

#[derive(Clone, Copy)]
struct GridLines {
    /// Square size in logical pixels.
    spacing: f32,
    /// No lines within this distance of the edges.
    inset: f32,
    /// Lines pass through the center moved by this (x, y) offset, instead of starting at `inset` from
    /// the top-left corner.
    center: Option<(f32, f32)>,
    color: gpui::Hsla,
}

// Inset the preview grid's first lines beyond its corner radius. GPUI's
// overflow mask is rectangular, so lines on the outer edges leak into corners.
fn grid_background(lines: GridLines) -> impl IntoElement {
    let GridLines { spacing, inset, center, color } = lines;
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if spacing <= 0. {
                return;
            }
            let thickness = px(1. / window.scale_factor());
            let (width, height) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            // The first line's position: `inset`, or the (moved) center's phase so a line passes through it.
            let first = |extent: f32, offset: Option<f32>| match offset {
                Some(offset) => (extent / 2. + offset).rem_euclid(spacing),
                None => inset,
            };
            let positions = move |extent: f32, offset: Option<f32>| {
                let start = first(extent, offset);
                (0..)
                    .map(move |i| start + i as f32 * spacing)
                    .take_while(move |p| *p < extent - inset)
                    .filter(move |p| *p >= inset)
            };
            for x in positions(width, center.map(|(x, _)| x)) {
                let origin = bounds.origin + gpui::point(px(x), px(0.));
                window.paint_quad(gpui::fill(
                    gpui::Bounds::new(origin, gpui::size(thickness, bounds.size.height)),
                    color,
                ));
            }
            for y in positions(height, center.map(|(_, y)| y)) {
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
        .rounded(px(look.radii().xl))
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

#[cfg(test)]
mod grid_tests {
    use super::grid_line_color;

    #[test]
    fn grid_lines_step_away_from_their_background() {
        let light = grid_line_color(gpui::hsla(0., 0., 0.96, 1.));
        let dark = grid_line_color(gpui::hsla(0., 0., 0.17, 1.));
        assert!((light.l - 0.88).abs() < 1e-4, "darker on light: {light:?}");
        assert!((dark.l - 0.25).abs() < 1e-4, "lighter on dark: {dark:?}");
    }
}
