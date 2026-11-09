//! Gradients tab stop list: a [`ColorStopList`] mirroring the gradient bar.
//!
//! The gradient bar ([`SliderControl`] thumbs plus `stop_colors`) stays the
//! source of truth. The list is keyed by `ThumbId::as_u64`, so rows, selection
//! and the inline editor survive edits made on the bar. List edits are applied
//! to the bar, and `rebuild_stops` pushes the canonical result back.

use super::*;

use gpui::{App, Focusable, Window};
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

use crate::color_stop_list::{ColorStopListBuilder, ColorStopListEvent, HeaderActions, StopData, StopKey};

impl GradientBuilder {
    pub(super) fn spawn_stop_list(look: Arc<Look>, cx: &mut App) -> Entity<ColorStopList> {
        ColorStopListBuilder::new("color-viz-stop-list")
            .look(look)
            .positioned_stops(std::iter::empty())
            .count_limits(STOP_LIMITS.0, STOP_LIMITS.1)
            .header_actions(HeaderActions::ALL)
            .spawn(cx)
    }

    pub(super) fn wire_stop_list(&mut self, cx: &mut Context<Self>) {
        self._subscriptions.push(cx.subscribe(&self.stop_list, |this, _, event: &ColorStopListEvent, cx| {
            this.handle_stop_list_event(event, cx);
        }));
    }

    /// Push the bar's stops and selection into the list (host setters; no events).
    pub(super) fn sync_stop_list(&mut self, cx: &mut Context<Self>) {
        let stops: Vec<StopData> = self
            .ordered_stop_rows(cx)
            .into_iter()
            .map(|(thumb_id, position, color)| StopData {
                key: thumb_id.as_u64(),
                color: gpui_bridge::from_hsla(color),
                position: Some(position),
            })
            .collect();
        let selected = self.selected_stop.map(ThumbId::as_u64);
        self.stop_list.update(cx, |list, cx| {
            list.set_keyed_stops(stops, cx);
            list.select(selected, cx);
        });
    }

    /// The user grabbed a knob: select its stop and hold the stop's picker open
    /// while the knob keeps focus.
    pub(super) fn open_stop_editor(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) {
        self.selected_stop = Some(thumb_id);
        self.stop_list.update(cx, |list, cx| list.open_editor(Some(thumb_id.as_u64()), cx));
        cx.notify();
    }

    /// A press outside the bar and list: the knob gives up focus, the picker
    /// closes, and nothing stays selected.
    pub(super) fn release_stop_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.gradient_stops.read(cx).focus_handle(cx).contains_focused(window, cx) {
            window.blur(cx);
        }
        if self.selected_stop.take().is_some() {
            self.stop_list.update(cx, |list, cx| {
                list.open_editor(None, cx);
                list.select(None, cx);
            });
            cx.notify();
        }
    }

    fn thumb_for_key(&self, key: StopKey, cx: &Context<Self>) -> Option<ThumbId> {
        self.gradient_stops.read(cx).thumbs().iter().map(|thumb| thumb.id).find(|id| id.as_u64() == key)
    }

    pub(super) fn handle_stop_list_event(&mut self, event: &ColorStopListEvent, cx: &mut Context<Self>) {
        match *event {
            ColorStopListEvent::SelectionChanged(key) => match key.and_then(|key| self.thumb_for_key(key, cx)) {
                Some(thumb_id) => self.select_stop_row(thumb_id, cx),
                None => {
                    self.selected_stop = None;
                    cx.notify();
                }
            },
            ColorStopListEvent::ColorChanged { key, color } => {
                if let (Some(thumb_id), Some(color)) = (self.thumb_for_key(key, cx), to_hsla(color)) {
                    self.stop_colors.insert(thumb_id, color);
                    self.rebuild_stops(cx);
                }
            }
            ColorStopListEvent::Added(key) => self.adopt_added_stop(key, cx),
            ColorStopListEvent::Removed(key) => match self.thumb_for_key(key, cx) {
                Some(thumb_id) => self.handle_delete_stop(thumb_id, cx),
                None => self.rebuild_stops(cx),
            },
            ColorStopListEvent::Reordered | ColorStopListEvent::Reversed | ColorStopListEvent::Distributed => {
                self.adopt_stop_slots(cx);
            }
            ColorStopListEvent::Activated(_) => {}
        }
    }

    /// The list inserted a stop under its own key; create the matching thumb
    /// and let the next sync replace the list's key with the thumb's.
    fn adopt_added_stop(&mut self, key: StopKey, cx: &mut Context<Self>) {
        let added = self.stop_list.read(cx).stops().iter().find(|stop| stop.key == key).copied();
        let Some(StopData { color, position: Some(position), .. }) = added else {
            self.rebuild_stops(cx);
            return;
        };
        let mut inserted = None;
        self.gradient_stops.update(cx, |slider, cx| inserted = slider.insert_thumb_at(position, cx));
        if let Some(thumb_id) = inserted {
            if let Some(color) = to_hsla(color) {
                self.stop_colors.insert(thumb_id, color);
            }
            self.selected_stop = Some(thumb_id);
        }
        self.rebuild_stops(cx);
    }

    /// Reorder / reverse / distribute: the list's sorted stops become the
    /// bar's sorted thumbs slot by slot, so thumbs never need to cross.
    fn adopt_stop_slots(&mut self, cx: &mut Context<Self>) {
        let list = self.stop_list.read(cx);
        let targets: Vec<StopData> = list.stops().to_vec();
        let selected_index = list.selected().and_then(|key| targets.iter().position(|stop| stop.key == key));
        let thumbs: Vec<ThumbId> = self.ordered_stop_rows(cx).into_iter().map(|(id, _, _)| id).collect();
        if thumbs.len() != targets.len() {
            self.rebuild_stops(cx);
            return;
        }
        for (thumb_id, stop) in thumbs.iter().zip(&targets) {
            if let Some(color) = to_hsla(stop.color) {
                self.stop_colors.insert(*thumb_id, color);
            }
        }
        // Moving in order can still clamp against a neighbour that has not moved
        // yet; a few passes settle every thumb because the order is preserved.
        for _ in 0..thumbs.len() {
            let mut moved = false;
            self.gradient_stops.update(cx, |slider, cx| {
                for (thumb_id, stop) in thumbs.iter().zip(&targets) {
                    moved |= slider.set_thumb_position(*thumb_id, stop.position.unwrap_or(0.0), cx);
                }
            });
            if !moved {
                break;
            }
        }
        if let Some(index) = selected_index {
            self.selected_stop = thumbs.get(index).copied();
        }
        self.rebuild_stops(cx);
    }
}

/// Matches the gradient bar's thumb policy.
pub(super) const STOP_LIMITS: (usize, usize) = (2, 8);

fn to_hsla(color: ColorValue) -> Option<gpui::Hsla> {
    gpui_bridge::to_hsla(color, GamutMapping::CssLocalMinde).ok()
}
