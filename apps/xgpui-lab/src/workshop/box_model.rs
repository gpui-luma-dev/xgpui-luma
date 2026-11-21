//! Box-model diagram for the style inspector: nested margin › border › padding › content layers with
//! each edge's value, laid out like Luma Studio's theme inspector (copied, not shared, so the Lab keeps
//! no dependency on Studio).

use gpui::{Div, Hsla, SharedString, div, hsla, prelude::*, px};
use xgpui_luma::style::lss::{BoxModel, Edges};
use xgpui_luma::theme::ThemeMode;

use super::pane_type;

const EDGE_BAND: f32 = 22.0;
const EDGE_SIDE: f32 = 28.0;
const CONTENT_HEIGHT: f32 = 36.0;

/// Neutral layer fills, as in Studio's inspector.
#[derive(Clone, Copy)]
struct Colors {
    margin: Hsla,
    border: Hsla,
    padding: Hsla,
    content: Hsla,
    stroke: Hsla,
    label: Hsla,
}

impl Colors {
    fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self {
                margin: hsla(0.0, 0.0, 0.16, 1.0),
                border: hsla(0.0, 0.0, 0.24, 1.0),
                padding: hsla(0.0, 0.0, 0.32, 1.0),
                content: hsla(0.0, 0.0, 0.40, 1.0),
                stroke: hsla(0.0, 0.0, 0.88, 0.92),
                label: hsla(0.0, 0.0, 0.94, 0.96),
            },
            ThemeMode::Light => Self {
                margin: hsla(0.0, 0.0, 0.94, 1.0),
                border: hsla(0.0, 0.0, 0.86, 1.0),
                padding: hsla(0.0, 0.0, 0.78, 1.0),
                content: hsla(0.0, 0.0, 0.70, 1.0),
                stroke: hsla(0.0, 0.0, 0.18, 0.88),
                label: hsla(0.0, 0.0, 0.14, 0.96),
            },
        }
    }
}

/// The part's box model as nested layers. The content box shows the part's size (`auto` when unset).
pub(super) fn diagram(id: SharedString, model: &BoxModel, mode: ThemeMode) -> Div {
    let colors = Colors::for_mode(mode);
    let size = |v: Option<f32>| v.map_or("auto".to_string(), format_value);
    let content = div()
        .w_full()
        .min_w(px(0.0))
        .h(px(CONTENT_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .bg(colors.content)
        .border_1()
        .border_color(colors.stroke)
        .child(
            div()
                .font_family(pane_type::MONO)
                .text_size(pane_type::BOX_CONTENT_LABEL)
                .line_height(px(14.0))
                .text_color(colors.label)
                .child(format!(
                    "{} × {}",
                    size(model.width.or(model.min_width)),
                    size(model.height.or(model.min_height))
                )),
        );
    let border = model.border_width;
    let padding = layer("padding", &model.padding, colors.padding, true, colors, content);
    let border = layer(
        "border",
        &Edges { top: border, right: border, bottom: border, left: border },
        colors.border,
        false,
        colors,
        padding,
    );
    let margin = layer("margin", &model.margin, colors.margin, true, colors, border);
    div().debug_selector(move || id.to_string()).w_full().min_w(px(0.0)).child(margin)
}

fn layer(label: &'static str, edges: &Edges, fill: Hsla, dashed: bool, colors: Colors, child: impl IntoElement) -> Div {
    let shell = div().w_full().min_w(px(0.0)).flex().flex_col().bg(fill).border_1().border_color(colors.stroke);
    let shell = if dashed { shell.border_dashed() } else { shell };
    let band = |value: f32| {
        div()
            .w_full()
            .h(px(EDGE_BAND))
            .flex()
            .items_center()
            .justify_center()
            .child(edge_value(value, colors))
    };
    let side =
        |value: f32| div().w(px(EDGE_SIDE)).flex().items_center().justify_center().child(edge_value(value, colors));
    shell
        .child(
            band(edges.top).relative().child(
                div()
                    .absolute()
                    .top(px(2.0))
                    .left(px(4.0))
                    .text_size(pane_type::BOX_LAYER_LABEL)
                    .line_height(px(12.0))
                    .font_weight(pane_type::BOX_LAYER_WEIGHT)
                    .text_color(colors.label)
                    .child(label),
            ),
        )
        .child(
            div()
                .flex()
                .w_full()
                .items_stretch()
                .child(side(edges.left))
                .child(div().flex_1().min_w(px(0.0)).w_full().child(child))
                .child(side(edges.right)),
        )
        .child(band(edges.bottom))
}

fn edge_value(value: f32, colors: Colors) -> Div {
    div()
        .font_family(pane_type::MONO)
        .text_size(pane_type::BOX_EDGE_VALUE)
        .line_height(px(12.0))
        .text_color(colors.label)
        .child(format_value(value))
}

fn format_value(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.1}")
    }
}
