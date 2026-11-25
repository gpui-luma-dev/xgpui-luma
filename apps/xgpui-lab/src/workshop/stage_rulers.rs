//! Pixel rulers along the Control stage's top and left edges (issue #397). They read in the control's
//! own pixels: the origin is the control's top-left, and the ticks follow the stage zoom and pan.

use gpui::{App, Bounds, Hsla, Pixels, TextAlign, Window, point, px, size};

/// Each ruler's thickness on screen.
pub(super) const RULER_SIZE: f32 = 22.;
const LABEL_SIZE: f32 = 9.;
const LABEL_LINE_HEIGHT: f32 = 11.;
/// Labelled ticks are at least this far apart on screen, and unlabelled ones at least `MIN_TICK_GAP`.
const MIN_LABEL_GAP: f32 = 48.;
const MIN_TICK_GAP: f32 = 6.;
/// Tick lengths: a labelled tick, a midpoint tick, any other tick.
const MAJOR_TICK: f32 = 9.;
const MID_TICK: f32 = 6.;
const MINOR_TICK: f32 = 3.;
/// Steps between labels, in control pixels.
const LABEL_STEPS: [f32; 10] = [5., 10., 20., 25., 50., 100., 200., 250., 500., 1000.];

/// The rulers' colors: their background, and the ticks, labels and edge.
#[derive(Clone, Copy)]
pub(super) struct RulerColors {
    pub background: Hsla,
    pub ink: Hsla,
}

/// The step between labels and between ticks, in control pixels, for `zoom` (screen pixels per control
/// pixel). Ticks are `None` when even the coarsest subdivision would crowd.
fn steps(zoom: f32) -> (f32, Option<f32>) {
    let label = LABEL_STEPS.into_iter().find(|step| step * zoom >= MIN_LABEL_GAP).unwrap_or(1000.);
    let tick = [label / 10., label / 5., label / 2.].into_iter().find(|step| step * zoom >= MIN_TICK_GAP);
    (label, tick)
}

/// The control-pixel values from `start` to `end` that are multiples of `step`.
fn multiples(start: f32, end: f32, step: f32) -> impl Iterator<Item = f32> {
    let first = (start / step).ceil() as i64;
    let last = (end / step).floor() as i64;
    (first..=last).map(move |i| i as f32 * step)
}

/// Paint both rulers over `stage`, measured from `control`'s top-left at `zoom`.
pub(super) fn paint_rulers(
    stage: Bounds<Pixels>,
    control: Bounds<Pixels>,
    zoom: f32,
    colors: RulerColors,
    window: &mut Window,
    cx: &mut App,
) {
    let f = |p: Pixels| f32::from(p);
    let (left, top, right, bottom) = (f(stage.left()), f(stage.top()), f(stage.right()), f(stage.bottom()));
    let (origin_x, origin_y) = (f(control.left()), f(control.top()));
    let rect = |x: f32, y: f32, w: f32, h: f32| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));
    let edge = Hsla { a: colors.ink.a * 0.6, ..colors.ink };
    let extent = Hsla { a: colors.ink.a * 0.2, ..colors.ink };

    // Backgrounds, the control's extent on each, and the inner edges.
    window.paint_quad(gpui::fill(rect(left, top, right - left, RULER_SIZE), colors.background));
    window.paint_quad(gpui::fill(rect(left, top, RULER_SIZE, bottom - top), colors.background));
    let ruler_x0 = (left + RULER_SIZE).max(f(control.left()));
    let ruler_x1 = right.min(f(control.right()));
    if ruler_x1 > ruler_x0 {
        window.paint_quad(gpui::fill(rect(ruler_x0, top, ruler_x1 - ruler_x0, RULER_SIZE), extent));
    }
    let ruler_y0 = (top + RULER_SIZE).max(f(control.top()));
    let ruler_y1 = bottom.min(f(control.bottom()));
    if ruler_y1 > ruler_y0 {
        window.paint_quad(gpui::fill(rect(left, ruler_y0, RULER_SIZE, ruler_y1 - ruler_y0), extent));
    }
    window.paint_quad(gpui::fill(rect(left, top + RULER_SIZE - 1., right - left, 1.), edge));
    window.paint_quad(gpui::fill(rect(left + RULER_SIZE - 1., top, 1., bottom - top), edge));

    let (label_step, tick_step) = steps(zoom);
    let tick_len = |value: f32| {
        if (value / label_step).round() * label_step == value {
            MAJOR_TICK
        } else if (value / (label_step / 2.)).round() * (label_step / 2.) == value {
            MID_TICK
        } else {
            MINOR_TICK
        }
    };
    let step = tick_step.unwrap_or(label_step);

    // Top ruler: x in control pixels.
    let to_value = |screen: f32, origin: f32| (screen - origin) / zoom;
    for value in multiples(to_value(left + RULER_SIZE, origin_x), to_value(right, origin_x), step) {
        let x = (origin_x + value * zoom).round();
        let len = tick_len(value);
        window.paint_quad(gpui::fill(rect(x, top + RULER_SIZE - len, 1., len), colors.ink));
        if len == MAJOR_TICK {
            label(format!("{value:.0}"), point(px(x + 3.), px(top + 2.)), colors.ink, window, cx);
        }
    }
    // Left ruler: y in control pixels, labels just below their tick.
    for value in multiples(to_value(top + RULER_SIZE, origin_y), to_value(bottom, origin_y), step) {
        let y = (origin_y + value * zoom).round();
        let len = tick_len(value);
        window.paint_quad(gpui::fill(rect(left + RULER_SIZE - len, y, len, 1.), colors.ink));
        if len == MAJOR_TICK {
            label(format!("{value:.0}"), point(px(left + 2.), px(y + 1.)), colors.ink, window, cx);
        }
    }
    // The corner where the rulers meet.
    window.paint_quad(gpui::fill(rect(left, top, RULER_SIZE - 1., RULER_SIZE - 1.), colors.background));
}

fn label(text: String, origin: gpui::Point<Pixels>, color: Hsla, window: &mut Window, cx: &mut App) {
    let run = gpui::TextRun { len: text.len(), font: gpui::font(super::pane_type::MONO), color, ..Default::default() };
    let line = window.text_system().shape_line(text.into(), px(LABEL_SIZE), &[run], None);
    if let Err(error) = line.paint(origin, px(LABEL_LINE_HEIGHT), TextAlign::Left, None, window, cx) {
        tracing::error!("Workshop ruler label: {error:#}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_spread_out_and_ticks_appear_as_the_zoom_grows() {
        assert_eq!(steps(1.), (50., Some(10.)));
        assert_eq!(steps(2.), (25., Some(5.)));
        assert_eq!(steps(10.), (5., Some(1.)));
        assert_eq!(multiples(-12., 31., 10.).collect::<Vec<_>>(), vec![-10., 0., 10., 20., 30.]);
    }
}
