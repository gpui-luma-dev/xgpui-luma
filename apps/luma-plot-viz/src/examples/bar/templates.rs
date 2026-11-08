use gpui::{AnyElement, App, IntoElement, Window, div, prelude::*, px};
use xgpui_luma_plot::BarRenderModel;

pub(super) fn rounded(bar: &BarRenderModel, _: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(bar.bounds.size.width)
        .h(bar.bounds.size.height)
        .rounded(px(6.0))
        .bg(bar.presentation.fill)
        .into_any_element()
}

/// A developer-defined shape using only resolved template placement.
pub(super) fn tapered(bar: &BarRenderModel, _: &mut Window, _: &mut App) -> AnyElement {
    let horizontal = bar.orientation.is_horizontal();
    let positive = if horizontal {
        bar.endpoint.x >= bar.baseline.x
    } else {
        bar.endpoint.y >= bar.baseline.y
    };
    let fill = bar.presentation.fill;
    gpui::canvas(
        move |_, _, _| (),
        move |bounds, _, window, _| {
            let (start, end, half) = if horizontal {
                (
                    gpui::point(if positive { bounds.left() } else { bounds.right() }, bounds.center().y),
                    gpui::point(if positive { bounds.right() } else { bounds.left() }, bounds.center().y),
                    bounds.size.height / 2.0,
                )
            } else {
                (
                    gpui::point(bounds.center().x, if positive { bounds.top() } else { bounds.bottom() }),
                    gpui::point(bounds.center().x, if positive { bounds.bottom() } else { bounds.top() }),
                    bounds.size.width / 2.0,
                )
            };
            let offset = |amount| {
                if horizontal {
                    gpui::point(px(0.0), amount)
                } else {
                    gpui::point(amount, px(0.0))
                }
            };
            let mut path = gpui::PathBuilder::fill();
            path.move_to(start - offset(half));
            path.line_to(start + offset(half));
            path.line_to(end + offset(half * 0.3));
            path.line_to(end - offset(half * 0.3));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, fill);
            }
        },
    )
    .w(bar.bounds.size.width)
    .h(bar.bounds.size.height)
    .into_any_element()
}
