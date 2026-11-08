#![allow(clippy::too_many_arguments, dead_code)]

use gpui::{AnyElement, Div, FontWeight, Hsla, IntoElement, SharedString, div, hsla, px, prelude::*};
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    ButtonInspectMetrics, CheckboxInspectMetrics, RadioButtonInspectMetrics, SwitchInspectMetrics,
};

#[derive(Clone, Copy, Debug)]
pub struct BoxModelLayerColors {
    pub margin: Hsla,
    pub border: Hsla,
    pub padding: Hsla,
    pub content: Hsla,
    pub stroke: Hsla,
    pub highlight: Hsla,
}

impl BoxModelLayerColors {
    /// DevTools layer order mapped to shadcn chart tokens from the loaded theme.
    pub fn from_look(look: &ShadcnLook) -> Self {
        Self {
            margin: chart_fill(look, "chart-5", fallback_margin()),
            border: chart_fill(look, "chart-4", fallback_border()),
            padding: chart_fill(look, "chart-2", fallback_padding()),
            content: chart_fill(look, "chart-1", fallback_content()),
            stroke: look
                .token_color("foreground")
                .map(|color| Hsla { a: 0.45, ..color })
                .unwrap_or_else(|_| fallback_stroke()),
            highlight: look.token_color("chart-3").unwrap_or_else(|_| fallback_highlight()),
        }
    }
}

fn chart_fill(look: &ShadcnLook, token: &str, fallback: Hsla) -> Hsla {
    look.token_color(token).map(|color| Hsla { a: 0.42, ..color }).unwrap_or(fallback)
}

fn fallback_margin() -> Hsla {
    hsla(30.0 / 360.0, 0.45, 0.55, 0.35)
}

fn fallback_border() -> Hsla {
    hsla(45.0 / 360.0, 0.55, 0.65, 0.45)
}

fn fallback_padding() -> Hsla {
    hsla(110.0 / 360.0, 0.35, 0.55, 0.45)
}

fn fallback_content() -> Hsla {
    hsla(210.0 / 360.0, 0.45, 0.55, 0.55)
}

fn fallback_stroke() -> Hsla {
    hsla(0.0, 0.0, 0.15, 0.55)
}

fn fallback_highlight() -> Hsla {
    hsla(210.0 / 360.0, 0.85, 0.55, 1.0)
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricFieldHighlight {
    None,
    Height,
    PaddingX,
    PaddingY,
    Gap,
    Radius,
    BorderWidth,
    FocusRingWidth,
    FocusRingOffset,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InspectEdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl InspectEdgeInsets {
    pub fn symmetric(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    pub fn max_edge(self) -> f32 {
        self.top.max(self.right).max(self.bottom).max(self.left)
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            top: self.top.max(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
        }
    }

    pub fn has_overflow(self) -> bool {
        self.top > 0.0 || self.right > 0.0 || self.bottom > 0.0 || self.left > 0.0
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InspectOccupationSnapshot {
    pub paint: InspectEdgeInsets,
    pub layout: InspectEdgeInsets,
}

impl InspectOccupationSnapshot {
    pub fn has_overflow(self) -> bool {
        self.paint.has_overflow() || self.layout.has_overflow()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InspectBoxModelSnapshot {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub border_width: f32,
    pub gap: f32,
    pub radius: f32,
    pub focus_ring_width: f32,
    pub focus_ring_offset: f32,
}

impl InspectBoxModelSnapshot {
    pub fn from_button_metrics(metrics: &ButtonInspectMetrics) -> Self {
        Self {
            height: metrics.height.value_px,
            padding_x: metrics.padding_x.value_px,
            padding_y: metrics.padding_y.value_px,
            border_width: metrics.border_width.value_px,
            gap: metrics.gap.value_px,
            radius: metrics.radius.value_px,
            focus_ring_width: metrics.focus_ring_width.value_px,
            focus_ring_offset: metrics.focus_ring_offset.value_px,
        }
    }

    pub fn from_checkbox_metrics(metrics: &CheckboxInspectMetrics, indicator_only: bool) -> Self {
        if indicator_only {
            choice_indicator_only_box_model(
                metrics.indicator_size.value_px,
                metrics.border_width.value_px,
                metrics.indicator_radius.value_px,
                metrics.focus_ring_width.value_px,
                metrics.focus_ring_offset.value_px,
            )
        } else {
            choice_indicator_box_model(
                metrics.height.value_px,
                metrics.indicator_size.value_px,
                metrics.gap.value_px,
                metrics.border_width.value_px,
                metrics.control_radius.value_px,
                metrics.focus_ring_width.value_px,
                metrics.focus_ring_offset.value_px,
            )
        }
    }

    pub fn from_radio_metrics(metrics: &RadioButtonInspectMetrics, indicator_only: bool) -> Self {
        if indicator_only {
            choice_indicator_only_box_model(
                metrics.indicator_size.value_px,
                metrics.border_width.value_px,
                metrics.control_radius.value_px,
                metrics.focus_ring_width.value_px,
                metrics.focus_ring_offset.value_px,
            )
        } else {
            choice_indicator_box_model(
                metrics.height.value_px,
                metrics.indicator_size.value_px,
                metrics.gap.value_px,
                metrics.border_width.value_px,
                metrics.control_radius.value_px,
                metrics.focus_ring_width.value_px,
                metrics.focus_ring_offset.value_px,
            )
        }
    }

    pub fn from_switch_metrics(metrics: &SwitchInspectMetrics) -> Self {
        let track_padding = metrics.track_padding.value_px;
        Self {
            height: metrics.track_height.value_px,
            padding_x: track_padding,
            padding_y: track_padding,
            border_width: metrics.border_width.value_px,
            gap: metrics.gap.value_px,
            radius: metrics.track_radius.value_px,
            focus_ring_width: metrics.focus_ring_width.value_px,
            focus_ring_offset: metrics.focus_ring_offset.value_px,
        }
    }

    pub fn from_textfield_metrics(metrics: &xgpui_luma_look_shadcn::inspect::TextFieldInspectMetrics) -> Self {
        Self {
            height: metrics.min_height.value_px,
            padding_x: metrics.padding_x.value_px,
            padding_y: metrics.padding_y.value_px,
            border_width: metrics.border_width.value_px,
            gap: 0.0,
            radius: metrics.radius.value_px,
            focus_ring_width: metrics.focus_ring_width.value_px,
            focus_ring_offset: metrics.focus_ring_offset.value_px,
        }
    }

    pub fn from_badge_metrics(metrics: &xgpui_luma_look_shadcn::inspect::BadgeInspectMetrics) -> Self {
        Self {
            height: metrics.min_height.value_px,
            padding_x: metrics.padding_x.value_px,
            padding_y: metrics.padding_y.value_px,
            border_width: 0.0,
            gap: metrics.gap.value_px,
            radius: metrics.radius.value_px,
            focus_ring_width: 0.0,
            focus_ring_offset: 0.0,
        }
    }
}

fn choice_indicator_only_box_model(
    indicator_size: f32,
    border_width: f32,
    indicator_radius: f32,
    focus_ring_width: f32,
    focus_ring_offset: f32,
) -> InspectBoxModelSnapshot {
    InspectBoxModelSnapshot {
        height: indicator_size,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width,
        gap: 0.0,
        radius: indicator_radius,
        focus_ring_width,
        focus_ring_offset,
    }
}

fn choice_indicator_box_model(
    height: f32,
    indicator_size: f32,
    gap: f32,
    border_width: f32,
    control_radius: f32,
    focus_ring_width: f32,
    focus_ring_offset: f32,
) -> InspectBoxModelSnapshot {
    let padding_y = ((height - indicator_size) / 2.0).max(0.0);
    InspectBoxModelSnapshot {
        height,
        padding_x: 0.0,
        padding_y,
        border_width,
        gap,
        radius: control_radius,
        focus_ring_width,
        focus_ring_offset,
    }
}

#[allow(dead_code)]
pub fn metric_field_highlight(name: &str) -> MetricFieldHighlight {
    match name {
        "height" => MetricFieldHighlight::Height,
        "padding x" => MetricFieldHighlight::PaddingX,
        "padding y" => MetricFieldHighlight::PaddingY,
        "gap" => MetricFieldHighlight::Gap,
        "radius" => MetricFieldHighlight::Radius,
        "border width" => MetricFieldHighlight::BorderWidth,
        "focus ring width" => MetricFieldHighlight::FocusRingWidth,
        "focus ring offset" => MetricFieldHighlight::FocusRingOffset,
        _ => MetricFieldHighlight::Height,
    }
}

mod layout {
    pub(super) const EDGE_BAND: f32 = 22.0;
    pub(super) const EDGE_SIDE: f32 = 28.0;
    pub(super) const CONTENT_HEIGHT: f32 = 36.0;
    pub(super) const LAYER_LABEL_SIZE: f32 = 10.0;
    pub(super) const EDGE_VALUE_SIZE: f32 = 10.0;
    pub(super) const CONTENT_LABEL_SIZE: f32 = 11.0;
}

/// Fixed gutter size for diagram labels; not proportional to the measured value.
fn label_gutter_band(value: f32) -> f32 {
    if value > 0.0 { layout::EDGE_BAND } else { 0.0 }
}

fn label_gutter_side(value: f32) -> f32 {
    if value > 0.0 { layout::EDGE_SIDE } else { 0.0 }
}

pub fn render_box_model_diagram(
    id: SharedString,
    model: &InspectBoxModelSnapshot,
    occupation: Option<&InspectOccupationSnapshot>,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    chrome_outline: Hsla,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> impl IntoElement {
    let body = build_box_model_core(model, highlight, colors, label_color, mono, mono_font.clone());
    let body = if matches!(highlight, MetricFieldHighlight::FocusRingWidth | MetricFieldHighlight::FocusRingOffset) {
        div()
            .w_full()
            .border_1()
            .border_dashed()
            .border_color(colors.highlight)
            .p(px(layout::EDGE_BAND))
            .child(body)
            .into_any_element()
    } else {
        body.into_any_element()
    };

    let body =
        wrap_chrome_and_occupation(body, occupation, colors, chrome_outline, label_color, mono, mono_font.clone());

    let caption = highlight_caption(model, highlight);
    let occupation_caption =
        occupation.and_then(|occupation| occupation_summary_caption(model.height, occupation, mono, label_color));

    div()
        .id(id)
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .when_some(caption, |stack, caption| {
            stack.gap(px(6.0)).child(
                div()
                    .w_full()
                    .text_size(px(mono.size))
                    .line_height(px(mono.line_height))
                    .text_color(label_color)
                    .child(caption),
            )
        })
        .when_some(occupation_caption, |stack, caption| stack.child(caption))
        .child(body)
}

fn wrap_chrome_and_occupation(
    body: AnyElement,
    occupation: Option<&InspectOccupationSnapshot>,
    colors: BoxModelLayerColors,
    chrome_outline: Hsla,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> AnyElement {
    let Some(occupation) = occupation else {
        return body;
    };

    let chrome = div().w_full().min_w(px(0.0)).border_1().border_color(chrome_outline).child(body);

    if !occupation.has_overflow() {
        return chrome.into_any_element();
    }

    let paint = occupation.paint;
    let occupation_fill = Hsla { a: 0.18, ..colors.highlight };
    let top_band = label_gutter_band(paint.top);
    let bottom_band = label_gutter_band(paint.bottom);
    let left_side = label_gutter_side(paint.left);
    let right_side = label_gutter_side(paint.right);

    let mut shell = div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(occupation_fill)
        .border_1()
        .border_dashed()
        .border_color(colors.highlight);

    if top_band > 0.0 {
        shell = shell.child(
            div()
                .relative()
                .w_full()
                .h(px(top_band))
                .flex()
                .items_center()
                .justify_center()
                .child(occupation_edge_value(paint.top, colors.highlight, mono_font.clone(), mono))
                .child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .left(px(4.0))
                        .text_size(px(layout::LAYER_LABEL_SIZE))
                        .line_height(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(label_color)
                        .child("occupation"),
                ),
        );
    }

    let mut row = div().flex().w_full().items_stretch();
    if left_side > 0.0 {
        row = row.child(div().w(px(left_side)).flex().items_center().justify_center().child(occupation_edge_value(
            paint.left,
            colors.highlight,
            mono_font.clone(),
            mono,
        )));
    }
    row = row.child(div().flex_1().min_w(px(0.0)).w_full().child(chrome));
    if right_side > 0.0 {
        row = row.child(div().w(px(right_side)).flex().items_center().justify_center().child(occupation_edge_value(
            paint.right,
            colors.highlight,
            mono_font.clone(),
            mono,
        )));
    }
    shell = shell.child(row);

    if bottom_band > 0.0 {
        shell = shell.child(
            div()
                .w_full()
                .h(px(bottom_band))
                .flex()
                .items_center()
                .justify_center()
                .child(occupation_edge_value(paint.bottom, colors.highlight, mono_font, mono)),
        );
    }

    shell.into_any_element()
}

fn occupation_edge_value(value: f32, color: Hsla, mono_font: SharedString, mono: &LumaTextStyle) -> Div {
    div()
        .font_family(mono_font)
        .text_size(px(layout::EDGE_VALUE_SIZE))
        .line_height(px(12.0))
        .font_weight(mono.weight)
        .text_color(color)
        .child(format_metric_value(value))
}

fn occupation_summary_caption(
    chrome_height: f32,
    occupation: &InspectOccupationSnapshot,
    mono: &LumaTextStyle,
    label_color: Hsla,
) -> Option<Div> {
    if !occupation.has_overflow() {
        return None;
    }

    let layout_height = chrome_height + occupation.layout.top + occupation.layout.bottom;
    let paint_height = chrome_height + occupation.paint.top + occupation.paint.bottom;
    Some(
        div()
            .w_full()
            .pt(px(6.0))
            .text_size(px(mono.size))
            .line_height(px(mono.line_height))
            .text_color(label_color)
            .child(format!(
                "layout height {} · paint height {}",
                format_metric_value(layout_height),
                format_metric_value(paint_height),
            )),
    )
}

fn highlight_caption(model: &InspectBoxModelSnapshot, highlight: MetricFieldHighlight) -> Option<String> {
    match highlight {
        MetricFieldHighlight::None => None,
        MetricFieldHighlight::Height => Some(format!("height · {}", format_metric_value(model.height))),
        MetricFieldHighlight::Gap => Some(format!("content gap · {}", format_metric_value(model.gap))),
        MetricFieldHighlight::Radius => Some(format!("corner radius · {}", format_metric_value(model.radius))),
        MetricFieldHighlight::FocusRingWidth => {
            Some(format!("focus ring · width {}", format_metric_value(model.focus_ring_width)))
        }
        MetricFieldHighlight::FocusRingOffset => Some(format!(
            "focus ring · offset {} · width {}",
            format_metric_value(model.focus_ring_offset),
            format_metric_value(model.focus_ring_width)
        )),
        _ => None,
    }
}

fn build_box_model_core(
    model: &InspectBoxModelSnapshot,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> Div {
    let content_label = "\"content\"".to_string();
    let margin = px_value(0.0);
    let border = px_value(model.border_width);
    let padding_x = px_value(model.padding_x);
    let padding_y = px_value(model.padding_y);

    let content = content_box(content_label, colors.content, colors, label_color, mono_font.clone());

    let padding = box_layer(
        "padding",
        EdgeValues::symmetric_vertical(&padding_y, &padding_x),
        layer_style(
            colors.padding,
            colors,
            true,
            matches!(highlight, MetricFieldHighlight::PaddingX | MetricFieldHighlight::PaddingY),
        ),
        Layer::Padding,
        highlight,
        colors,
        label_color,
        mono,
        mono_font.clone(),
        content,
    );

    let border = box_layer(
        "border",
        EdgeValues::all(&border),
        layer_style(colors.border, colors, false, matches!(highlight, MetricFieldHighlight::BorderWidth)),
        Layer::Border,
        highlight,
        colors,
        label_color,
        mono,
        mono_font.clone(),
        padding,
    );

    box_layer(
        "margin",
        EdgeValues::all(&margin),
        layer_style(colors.margin, colors, true, matches!(highlight, MetricFieldHighlight::Height)),
        Layer::Margin,
        highlight,
        colors,
        label_color,
        mono,
        mono_font,
        border,
    )
}

struct EdgeValues {
    top: SharedString,
    right: SharedString,
    bottom: SharedString,
    left: SharedString,
}

impl EdgeValues {
    fn all(value: &SharedString) -> Self {
        Self { top: value.clone(), right: value.clone(), bottom: value.clone(), left: value.clone() }
    }

    fn symmetric_vertical(vertical: &SharedString, horizontal: &SharedString) -> Self {
        Self { top: vertical.clone(), right: horizontal.clone(), bottom: vertical.clone(), left: horizontal.clone() }
    }
}

struct LayerStyle {
    fill: Hsla,
    stroke: Hsla,
    dashed: bool,
}

fn layer_style(fill: Hsla, colors: BoxModelLayerColors, dashed: bool, highlighted: bool) -> LayerStyle {
    LayerStyle { fill, stroke: if highlighted { colors.highlight } else { colors.stroke }, dashed }
}

#[derive(Clone, Copy)]
enum Layer {
    Margin,
    Border,
    Padding,
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

fn box_layer(
    label: &'static str,
    edges: EdgeValues,
    style: LayerStyle,
    layer: Layer,
    highlight: MetricFieldHighlight,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    child: impl IntoElement,
) -> Div {
    let mut shell =
        div().w_full().min_w(px(0.0)).flex().flex_col().bg(style.fill).border_1().border_color(style.stroke);
    if style.dashed {
        shell = shell.border_dashed();
    }

    shell
        .child(
            div()
                .relative()
                .w_full()
                .h(px(layout::EDGE_BAND))
                .flex()
                .items_center()
                .justify_center()
                .child(edge_value(
                    edges.top,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Top, layer),
                ))
                .child(div().absolute().top(px(2.0)).left(px(4.0)).child(layer_title(label, label_color))),
        )
        .child(
            div()
                .flex()
                .w_full()
                .items_stretch()
                .child(edge_side(
                    edges.left,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Left, layer),
                ))
                .child(div().flex_1().min_w(px(0.0)).w_full().child(child))
                .child(edge_side(
                    edges.right,
                    label_color,
                    colors,
                    mono,
                    mono_font.clone(),
                    highlight_edge(highlight, Edge::Right, layer),
                )),
        )
        .child(div().w_full().h(px(layout::EDGE_BAND)).flex().items_center().justify_center().child(edge_value(
            edges.bottom,
            label_color,
            colors,
            mono,
            mono_font,
            highlight_edge(highlight, Edge::Bottom, layer),
        )))
}

fn content_box(
    label: String,
    fill: Hsla,
    colors: BoxModelLayerColors,
    label_color: Hsla,
    mono_font: SharedString,
) -> Div {
    div()
        .w_full()
        .min_w(px(0.0))
        .h(px(layout::CONTENT_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .bg(fill)
        .border_1()
        .border_color(colors.stroke)
        .child(
            div()
                .font_family(mono_font)
                .text_size(px(layout::CONTENT_LABEL_SIZE))
                .line_height(px(14.0))
                .text_color(label_color)
                .child(label),
        )
}

fn highlight_edge(highlight: MetricFieldHighlight, edge: Edge, layer: Layer) -> bool {
    match (highlight, layer, edge) {
        (MetricFieldHighlight::None, _, _) => false,
        (MetricFieldHighlight::Height, Layer::Margin, _) => true,
        (MetricFieldHighlight::BorderWidth, Layer::Border, _) => true,
        (MetricFieldHighlight::PaddingX, Layer::Padding, Edge::Left | Edge::Right) => true,
        (MetricFieldHighlight::PaddingY, Layer::Padding, Edge::Top | Edge::Bottom) => true,
        _ => false,
    }
}

fn layer_title(label: &'static str, color: Hsla) -> Div {
    div()
        .text_size(px(layout::LAYER_LABEL_SIZE))
        .line_height(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

fn edge_side(
    value: SharedString,
    color: Hsla,
    colors: BoxModelLayerColors,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    highlighted: bool,
) -> Div {
    div().w(px(layout::EDGE_SIDE)).flex().items_center().justify_center().child(edge_value(
        value,
        color,
        colors,
        mono,
        mono_font,
        highlighted,
    ))
}

fn edge_value(
    value: SharedString,
    color: Hsla,
    colors: BoxModelLayerColors,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    highlighted: bool,
) -> Div {
    div()
        .font_family(mono_font)
        .text_size(px(layout::EDGE_VALUE_SIZE))
        .line_height(px(12.0))
        .font_weight(if highlighted { FontWeight::SEMIBOLD } else { mono.weight })
        .text_color(if highlighted { colors.highlight } else { color })
        .child(value)
}

fn px_value(value: f32) -> SharedString {
    format_metric_value(value).into()
}

fn format_metric_value(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.1}")
    }
}
