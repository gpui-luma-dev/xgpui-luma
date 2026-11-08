use xgpui_luma::motion::{Transition, Easing};
use std::{collections::HashMap, time::Duration};
use gpui::{AnyElement, App, Bounds, Context, Pixels, Window, div, point, px, size, prelude::*};
use xgpui_luma::{
    motion::VisualTransition,
    infra::attachments::{AttachmentHost, AttachmentTarget, TooltipEntityExt},
    theme::InteractionState,
};
use crate::{
    PieSlice, PieSliceEvent,
    views::pie::{PieEvents, PieTooltip},
};
use super::{PieLayout, polar};
use super::super::points::motion::AnchorMotion;

#[derive(Default)]
struct ExpansionState {
    motion: HashMap<usize, VisualTransition>,
    bounds: Option<Bounds<Pixels>>,
    duration: Duration,
}
pub(super) fn expansions(
    slices: &[PieSlice],
    active: Option<usize>,
    bounds: Bounds<Pixels>,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> HashMap<usize, f32> {
    let state = window.use_state(cx, |_, _| ExpansionState::default());
    state.update(cx, |state, cx| {
        let reset = state.bounds != Some(bounds) || state.duration != duration;
        state.motion.retain(|index, _| slices.iter().any(|slice| slice.sample_index == *index));
        let result = slices
            .iter()
            .map(|slice| {
                let target = if active == Some(slice.sample_index) { 1.0 } else { 0.0 };
                let transition = state.motion.entry(slice.sample_index).or_insert_with(|| {
                    VisualTransition::with_transition(target, Transition::timed(duration, Easing::OutCubic))
                });
                if reset {
                    *transition =
                        VisualTransition::with_transition(target, Transition::timed(duration, Easing::OutCubic));
                }
                transition.sync();
                if duration.is_zero() {
                    transition.snap_to(target);
                } else {
                    transition.set_target(target);
                }
                transition.schedule_frame(window, cx);
                (slice.sample_index, transition.progress())
            })
            .collect();
        state.bounds = Some(bounds);
        state.duration = duration;
        result
    })
}
fn same(a: Option<&PieSlice>, b: Option<&PieSlice>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.ring_index == b.ring_index
                && a.series == b.series
                && a.sample_index == b.sample_index
                && a.sample == b.sample
                && a.label == b.label
                && a.fraction == b.fraction
        }
        _ => false,
    }
}
#[derive(Default)]
struct HoverState {
    current: Option<PieSlice>,
}
pub(super) fn report_hover(slice: Option<PieSlice>, events: &PieEvents, window: &mut Window, cx: &mut App) {
    let state = window.use_state(cx, |_, _| HoverState::default());
    let changed = state.update(cx, |state, _| {
        let changed = !same(state.current.as_ref(), slice.as_ref());
        state.current = slice.clone();
        changed
    });
    if changed {
        let events = events.clone();
        cx.defer(move |cx| (events.0)(PieSliceEvent::HoverChanged { slice }, cx));
    }
}
struct TooltipOwner {
    attachments: AttachmentHost,
    anchor: Bounds<Pixels>,
}
impl AttachmentTarget for TooltipOwner {
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
    current: Option<PieSlice>,
    owner: Option<gpui::Entity<TooltipOwner>>,
    motion: AnchorMotion,
}
pub(super) fn tooltip(
    slice: Option<&PieSlice>,
    layout: &PieLayout,
    bounds: Bounds<Pixels>,
    factory: &PieTooltip,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let state = window.use_state(cx, |_, _| TooltipState::default());
    let target = state.update(cx, |state, cx: &mut Context<TooltipState>| {
        let Some(slice) = slice else {
            state.current = None;
            state.owner = None;
            state.motion.clear();
            return None;
        };
        if !same(state.current.as_ref(), Some(slice)) {
            let config = (factory.0)(slice);
            let owner = state.owner.clone().unwrap_or_else(|| {
                cx.new(|_| TooltipOwner { attachments: AttachmentHost::default(), anchor: Bounds::default() })
            });
            state.owner = Some(owner.tooltip(config, cx));
            state.current = Some(slice.clone());
        }
        let sector = layout.sectors.iter().find(|sector| sector.index == slice.sample_index)?;
        let anchor = polar(layout.center, (layout.inner + sector.radius) / 2.0, sector.start + sector.sweep / 2.0);
        let anchor = state.motion.update(anchor, bounds, duration);
        state.motion.schedule_frame(window, cx);
        Some((anchor, state.owner.clone()?))
    });
    let (anchor, owner) = target?;
    let mut element = owner.update(cx, |owner, cx| {
        owner.anchor = Bounds::new(anchor - point(px(4.0), px(4.0)), size(px(8.0), px(8.0)));
        owner
            .attachments
            .render(
                div().id("pie-tooltip").w(bounds.size.width).h(bounds.size.height),
                InteractionState { hovered: true, ..InteractionState::default() },
                cx,
            )
            .into_any_element()
    });
    element.prepaint_as_root(bounds.origin, bounds.size.map(gpui::AvailableSpace::Definite), window, cx);
    Some(element)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hover_content_changes_when_normalization_or_name_changes() {
        let data = crate::Dataset::new()
            .with_series(crate::Series::new("a", "A", [crate::Sample::new(0.0, 3.0), crate::Sample::new(1.0, 1.0)]))
            .unwrap();
        let changed = crate::Dataset::new()
            .with_series(crate::Series::new("a", "A", [crate::Sample::new(0.0, 3.0), crate::Sample::new(1.0, 3.0)]))
            .unwrap();
        let plot = crate::PiePlot::new("a");
        let a = crate::render::pie::slices(&data, &plot, &["First".into()])[0].clone();
        let b = crate::render::pie::slices(&changed, &plot, &["First".into()])[0].clone();
        assert_eq!(a.sample, b.sample);
        assert_eq!(a.fraction, 0.75);
        assert_eq!(b.fraction, 0.5);
        assert!(!same(Some(&a), Some(&b)));
        let renamed = crate::render::pie::slices(&data, &plot, &["Renamed".into()])[0].clone();
        assert!(!same(Some(&a), Some(&renamed)));
        assert!(same(Some(&a), Some(&a.clone())));
    }
}
