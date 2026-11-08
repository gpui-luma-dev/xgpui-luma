use gpui::{AnyElement, App, AvailableSpace, Bounds, Pixels, Point, Window, div, point, prelude::*, px, size};
use xgpui_luma::{
    infra::attachments::{AttachmentHost, AttachmentTarget, TooltipEntityExt},
    theme::InteractionState,
};
use crate::{
    Dataset, MarkerVisibility,
    presentation::CartesianPresentation,
    views::{PlotPoint, PlotSlice, PointTooltip, PlotPointEvent, PointEvents, PointTooltipFactory},
};
use super::geometry::CartesianGeometry;

#[derive(Clone)]
pub(super) struct HoverPoint {
    pub(super) area: bool,
    pub(super) bar: bool,
    pub(super) layer: usize,
    pub(super) position: Point<Pixels>,
    pub(super) info: PlotPoint,
}

struct PointTooltipOwner {
    attachments: AttachmentHost,
    anchor: Bounds<Pixels>,
}
impl AttachmentTarget for PointTooltipOwner {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
    fn attachment_anchor(&self, _: Bounds<Pixels>) -> Bounds<Pixels> {
        self.anchor
    }
}

#[derive(Default)]
struct TooltipState {
    current: Vec<HoverPoint>,
    slice: bool,
    active: bool,
    owner: Option<gpui::Entity<PointTooltipOwner>>,
    bounds: Bounds<Pixels>,
    motion: motion::AnchorMotion,
}

// Border observations are painted with clipped markers and remain selectable.
// GPUI's rectangle hit test excludes the right/bottom edge, so use inclusive
// bounds for observations while keeping normal hit testing for the pointer.
fn point_visible(position: Point<Pixels>, bounds: Bounds<Pixels>) -> bool {
    position.x >= bounds.left()
        && position.x <= bounds.right()
        && position.y >= bounds.top()
        && position.y <= bounds.bottom()
}

pub(super) fn interactive(style: &CartesianPresentation, tooltip: bool) -> bool {
    tooltip || style.has_hover_markers()
}

pub(super) fn nearest(
    geometry: &CartesianGeometry,
    data: &Dataset,
    style: &CartesianPresentation,
    pointer: Point<Pixels>,
    visible: Bounds<Pixels>,
) -> Option<HoverPoint> {
    if !visible.contains(&pointer) {
        return None;
    }
    let mut best = None;
    let mut distance = 12.0_f32.powi(2);
    for (area, layer, line) in geometry
        .areas
        .iter()
        .enumerate()
        .map(|(i, a)| (true, i, &a.boundary))
        .chain(geometry.lines.iter().enumerate().map(|(i, l)| (false, i, l)))
    {
        let visual = if area {
            style.area_for(&line.series).outline
        } else {
            Some(style.line_for(&line.series))
        };
        let Some(marker) = visual
            .and_then(|line| line.markers)
            .filter(|marker| f32::from(marker.radius).is_finite() && marker.radius > px(0.0))
        else {
            continue;
        };
        for (position, sample) in line.runs.iter().flatten().zip(line.samples.iter().flatten()) {
            if !point_visible(*position, visible) {
                continue;
            }
            let delta = *position - pointer;
            let next = f32::from(delta.x).powi(2) + f32::from(delta.y).powi(2);
            if next <= distance {
                distance = next;
                best = Some(HoverPoint {
                    bar: false,
                    area,
                    layer,
                    position: *position,
                    info: PlotPoint {
                        series: line.series.clone(),
                        label: data.get(&line.series).map_or_else(String::new, |series| series.label().to_owned()),
                        sample: *sample,
                        color: marker.fill,
                    },
                });
            }
        }
    }
    best
}

pub(super) fn select(
    geometry: &CartesianGeometry,
    data: &Dataset,
    style: &CartesianPresentation,
    pointer: Point<Pixels>,
    visible: Bounds<Pixels>,
) -> Vec<HoverPoint> {
    if !style.has_slice_markers() {
        return nearest(geometry, data, style, pointer, visible).into_iter().collect();
    }
    if !visible.contains(&pointer) {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    let mut slices = Vec::new();
    for (area, layer, line) in geometry
        .areas
        .iter()
        .enumerate()
        .map(|(i, a)| (true, i, &a.boundary))
        .chain(geometry.lines.iter().enumerate().map(|(i, l)| (false, i, l)))
    {
        let visual = if area {
            style.area_for(&line.series).outline
        } else {
            Some(style.line_for(&line.series))
        };
        let Some(marker) = visual.and_then(|line| line.markers).filter(|marker| {
            marker.visibility == MarkerVisibility::HoverSlice
                && marker.radius > px(0.0)
                && f32::from(marker.radius).is_finite()
        }) else {
            continue;
        };
        for (position, sample) in line.runs.iter().flatten().zip(line.samples.iter().flatten()) {
            if !point_visible(*position, visible) {
                continue;
            }
            slices.push((position.x, sample.x));
            candidates.push(HoverPoint {
                bar: false,
                area,
                layer,
                position: *position,
                info: PlotPoint {
                    series: line.series.clone(),
                    label: data.get(&line.series).map_or_else(String::new, |series| series.label().to_owned()),
                    sample: *sample,
                    color: marker.fill,
                },
            });
        }
        // A missing observation owns an X slice too: never jump across its gap.
        if let Some(series) = data.get(&line.series) {
            for sample in series.samples().iter().filter(|sample| sample.x.is_finite() && !sample.y.is_finite()) {
                let x = geometry.x_position(sample.x);
                if x >= visible.left() && x <= visible.right() {
                    slices.push((x, sample.x));
                }
            }
        }
    }
    let Some((_, selected)) = slices
        .iter()
        .min_by(|a, b| f32::from(a.0 - pointer.x).abs().total_cmp(&f32::from(b.0 - pointer.x).abs()))
    else {
        return Vec::new();
    };
    // One marker per layer even when a source run repeats the selected X value.
    let mut selected_points = Vec::new();
    for candidate in candidates.into_iter().filter(|point| point.info.sample.x == *selected) {
        if !selected_points
            .iter()
            .any(|point: &HoverPoint| point.area == candidate.area && point.layer == candidate.layer)
        {
            selected_points.push(candidate);
        }
    }
    selected_points
}

pub(super) fn tooltip_target(
    hovered: &[HoverPoint],
    factory: &PointTooltip,
    visible: Bounds<Pixels>,
    geometry: &CartesianGeometry,
    slice: bool,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let state = window.use_state(cx, |_, _| TooltipState::default());
    let target =
        state.update(cx, |state, cx| {
            if let Some(first) = hovered.first() {
                let changed = !state.active
                    || state.slice != slice
                    || state.current.len() != hovered.len()
                    || state.current.iter().zip(hovered).any(|(previous, next)| {
                        previous.bar != next.bar
                            || previous.area != next.area
                            || previous.layer != next.layer
                            || previous.info.series != next.info.series
                            || previous.info.sample != next.info.sample
                    });
                if changed {
                    let config = match &factory.factory {
                        PointTooltipFactory::Point(factory) => factory(&first.info),
                        PointTooltipFactory::Slice(factory) => factory(&PlotSlice {
                            coordinate: first.info.sample.x,
                            points: hovered.iter().map(|point| point.info.clone()).collect(),
                        }),
                    };
                    // Retain the owner entity across adjacent slice targets. The SDK
                    // replaces its attachment when the observation content changes.
                    let owner = state.owner.clone().filter(|_| state.slice && slice);
                    state.owner = Some(
                        owner
                            .unwrap_or_else(|| {
                                cx.new(|_| PointTooltipOwner {
                                    attachments: AttachmentHost::default(),
                                    anchor: Bounds::default(),
                                })
                            })
                            .tooltip(config, cx),
                    );
                }
                state.current = hovered.to_vec();
                state.slice = slice;
                state.bounds = if slice {
                    visible
                } else {
                    Bounds::new(first.position - point(px(12.0), px(12.0)), size(px(24.0), px(24.0)))
                };
            } else if slice
                || state.current.iter().any(|point| {
                    if point.bar {
                        return true;
                    }
                    let line = if point.area {
                        geometry.areas.get(point.layer).map(|area| &area.boundary)
                    } else {
                        geometry.lines.get(point.layer)
                    };
                    !point_visible(point.position, visible)
                        || line.is_none_or(|line| {
                            line.series != point.info.series
                                || !line.runs.iter().flatten().zip(line.samples.iter().flatten()).any(
                                    |(position, sample)| *position == point.position && *sample == point.info.sample,
                                )
                        })
                })
            {
                state.current.clear();
                state.owner = None;
                state.motion.clear();
            }
            // Refresh factories after leaving the plot (e.g. changing tooltip
            // controls), while retaining them across empty proximity regions.
            if !hovered.is_empty() {
                state.active = true;
            } else if !geometry.plot_bounds.contains(&window.mouse_position()) {
                state.active = false;
            }
            let first = state.current.first()?;
            let owner = state.owner.as_ref()?.clone();
            let anchor = if slice {
                point(first.position.x, window.mouse_position().y)
            } else {
                first.position
            };
            let position = state.motion.update(anchor, visible, factory.motion);
            state.motion.schedule_frame(window, cx);
            Some((position, state.bounds, owner))
        });
    let (anchor_position, bounds, owner) = target?;
    let root = div()
        .id("plot-point-tooltip")
        .w(bounds.size.width)
        .h(bounds.size.height)
        .role(gpui::Role::Image)
        .aria_label("Plot observation")
        .when(slice, |root| {
            root.on_mouse_move(|_, window, _| window.refresh()).on_hover(|_, window, _| window.refresh())
        });
    let mut element = owner.update(cx, |owner, cx| {
        owner.anchor = Bounds::new(anchor_position - point(px(12.0), px(12.0)), size(px(24.0), px(24.0)));
        owner
            .attachments
            .render(root, InteractionState { hovered: !hovered.is_empty(), ..InteractionState::default() }, cx)
            .into_any_element()
    });
    element.prepaint_as_root(
        bounds.origin,
        size(AvailableSpace::Definite(bounds.size.width), AvailableSpace::Definite(bounds.size.height)),
        window,
        cx,
    );
    Some(element)
}

#[cfg(test)]
mod tests;

// Keep event identity separate from tooltip lifetimes and visual changes.
#[derive(Default)]
struct PointEventState {
    current: Vec<HoverPoint>,
}

pub(super) fn report_hover(hovered: &[HoverPoint], observer: &PointEvents, window: &mut Window, cx: &mut App) {
    let state = window.use_state(cx, |_, _| PointEventState::default());
    let changed = state.update(cx, |state, _| {
        let changed = state.current.len() != hovered.len()
            || state.current.iter().zip(hovered).any(|(previous, next)| {
                previous.bar != next.bar
                    || previous.area != next.area
                    || previous.layer != next.layer
                    || previous.info.series != next.info.series
                    || previous.info.sample != next.info.sample
            });
        state.current = hovered.to_vec();
        changed
    });
    if changed {
        let event = PlotPointEvent::HoverChanged { points: hovered.iter().map(|point| point.info.clone()).collect() };
        let observer = observer.clone();
        cx.defer(move |cx| observer.report(event, cx));
    }
}

#[cfg(test)]
mod events_tests;

pub(super) mod motion;
