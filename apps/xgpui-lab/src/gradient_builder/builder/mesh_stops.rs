//! Mesh and Freeform colors as lab color controls.
//!
//! - The background is a standalone [`ColorStop`].
//! - Points are a [`ColorStopList`] keyed by `mesh_point_ids` (one list per
//!   tab: Mesh points are fixed, Freeform points can be removed). Freeform's
//!   per-point settings ([`PointEditorExtras`]) sit under the open picker.
//!
//! `mesh_background` / `mesh_points` stay the source of truth. Points have many
//! writers (canvas drags, palette animation, Freeform edits), so the list is
//! reconciled once per frame from `GradientBuilder::render` via host setters
//! that notify only on change. User edits arrive as list events.
//!
//! Colors stay source [`ColorValue`]s; pickers edit over Display P3, and each
//! renderer encodes for its own output (see `gradient_builder::encode`).

use super::*;

use gpui::{AnyView, App, IntoElement, Render, SharedString, Window, div, prelude::*};
use xgpui_luma::color::Gamut;

use super::super::color::format_percent;
use super::view::compact_slider_row;
use crate::color_stop::{ColorStop, ColorStopBuilder, ColorStopEvent};
use crate::color_stop_list::{ColorStopListBuilder, ColorStopListEvent, HeaderActions, StopData, StopKey};
use crate::theme::ColorVizLookExt;

/// Freeform's per-point settings, shown under the open point's picker.
pub(super) struct PointEditorExtras {
    look: Arc<Look>,
    spread_slider: Entity<SliderControl>,
    pub(super) randomize_button: Entity<Button>,
    spread: f32,
}

impl PointEditorExtras {
    fn set_spread(&mut self, spread: f32, cx: &mut Context<Self>) {
        if self.spread != spread {
            self.spread = spread;
            cx.notify();
        }
    }
}

impl Render for PointEditorExtras {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .items_end()
            .gap_2()
            .child(div().flex_1().min_w_0().child(compact_slider_row(
                "Point spread",
                self.spread_slider.clone(),
                format!("{:.0}%", self.spread),
                self.look.typography_scale(TextSize::Sm),
                self.look.chrome(),
            )))
            .child(self.randomize_button.clone())
    }
}

impl GradientBuilder {
    pub(super) fn spawn_mesh_background_stop(look: Arc<Look>, color: ColorValue, cx: &mut App) -> Entity<ColorStop> {
        ColorStopBuilder::new("color-viz-mesh-background", color)
            .look(look)
            .edit_gamut(Gamut::DisplayP3)
            .spawn(cx)
    }

    pub(super) fn wire_mesh_background_stop(&mut self, cx: &mut Context<Self>) {
        self._subscriptions
            .push(cx.subscribe(&self.mesh_background_stop, |this, _, event: &ColorStopEvent, cx| {
                if let ColorStopEvent::ColorChanged(color) = *event {
                    this.mesh_background = color;
                    let _ = this.ensure_preview_image_cache(this.preview_size, cx);
                    cx.notify();
                }
            }));
    }

    pub(super) fn spawn_point_lists(look: Arc<Look>, cx: &mut App) -> [Entity<ColorStopList>; 2] {
        let list = |id: &str, removable: bool| {
            ColorStopListBuilder::new(id)
                .look(look.clone())
                .colors(std::iter::empty())
                .count_limits(1, FREEFORM_MAX_POINTS)
                .removable(removable)
                .reorderable(false)
                .header_actions(HeaderActions::NONE)
                .edit_gamut(Gamut::DisplayP3)
        };
        [list("color-viz-mesh-points", false).spawn(cx), list("color-viz-freeform-points", true).spawn(cx)]
    }

    pub(super) fn spawn_point_extras(
        look: Arc<Look>,
        spread_slider: Entity<SliderControl>,
        cx: &mut Context<Self>,
    ) -> Entity<PointEditorExtras> {
        let randomize_button = crate::theme::icon_button("color-viz-point-randomize", LucideIcon::Palette)
            .look(look.as_ref())
            .content_only()
            .spawn(cx)
            .tooltip(Tooltip::new("Randomize point color"), cx);
        cx.new(|_| PointEditorExtras { look, spread_slider, randomize_button, spread: 100.0 })
    }

    pub(super) fn wire_point_lists(&mut self, cx: &mut Context<Self>) {
        for list in [self.mesh_point_list.clone(), self.freeform_point_list.clone()] {
            self._subscriptions.push(cx.subscribe(&list, |this, _, event: &ColorStopListEvent, cx| {
                this.handle_point_list_event(event, cx);
            }));
        }
        let randomize = self.point_extras.read(cx).randomize_button.clone();
        self._subscriptions.push(cx.subscribe(&randomize, |this, _, event: &ButtonEvent, cx| {
            if event.is_click() {
                this.edit_freeform(3, cx);
            }
        }));
    }

    pub(super) fn active_point_list(&self) -> &Entity<ColorStopList> {
        if self.selected_tab == BuilderTab::Freeform {
            &self.freeform_point_list
        } else {
            &self.mesh_point_list
        }
    }

    fn point_index(&self, key: StopKey) -> Option<usize> {
        self.mesh_point_ids.iter().position(|id| *id == key)
    }

    /// Reconcile the active point list with `mesh_points` (once per frame).
    pub(super) fn sync_point_list(&mut self, cx: &mut Context<Self>) {
        if self.selected_tab == BuilderTab::Gradients {
            return;
        }
        let stops: Vec<StopData> = self
            .mesh_point_ids
            .iter()
            .zip(&self.mesh_points)
            .map(|(key, point)| StopData { key: *key, color: point.color, position: None })
            .collect();
        let details: Vec<(StopKey, SharedString)> = self
            .mesh_point_ids
            .iter()
            .zip(&self.mesh_points)
            .map(|(key, point)| (*key, format!("({}, {})", format_percent(point.u), format_percent(point.v)).into()))
            .collect();
        let selected = self.selected_mesh_point.and_then(|index| self.mesh_point_ids.get(index)).copied();
        let freeform = self.selected_tab == BuilderTab::Freeform;
        let extra: Option<(StopKey, AnyView)> =
            selected.filter(|_| freeform).map(|key| (key, self.point_extras.clone().into()));
        if freeform {
            let spread =
                self.selected_mesh_point.and_then(|index| self.point_spreads.get(index)).copied().unwrap_or(100.0);
            self.point_extras.update(cx, |extras, cx| extras.set_spread(spread, cx));
        }
        self.active_point_list().clone().update(cx, |list, cx| {
            list.set_keyed_stops(stops, cx);
            list.set_row_details(details, cx);
            list.select(selected, cx);
            list.set_editor_extra(extra, cx);
        });
    }

    pub(super) fn handle_point_list_event(&mut self, event: &ColorStopListEvent, cx: &mut Context<Self>) {
        match *event {
            ColorStopListEvent::SelectionChanged(key) => {
                self.selected_mesh_point = key.and_then(|key| self.point_index(key));
                self.mesh_color_target =
                    self.selected_mesh_point.map(MeshColorTarget::Point).unwrap_or(MeshColorTarget::Background);
                self.active_mesh_drag = None;
                self.sync_point_spread(cx);
                cx.notify();
            }
            ColorStopListEvent::ColorChanged { key, color } => {
                if let Some(index) = self.point_index(key)
                    && let Some(point) = self.mesh_points.get_mut(index)
                {
                    point.color = color;
                    let _ = self.ensure_preview_image_cache(self.preview_size, cx);
                    cx.notify();
                }
            }
            ColorStopListEvent::Removed(key) => {
                if let Some(index) = self.point_index(key) {
                    self.selected_mesh_point = Some(index);
                    self.edit_freeform(1, cx);
                }
            }
            _ => {}
        }
    }

    /// A canvas handle was grabbed: select the point and hold its picker open.
    pub(super) fn open_point_editor(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(key) = self.mesh_point_ids.get(index).copied() {
            self.active_point_list().clone().update(cx, |list, cx| list.open_editor(Some(key), cx));
        }
    }

    /// A press outside the points section: close the picker and clear the
    /// selection, unless a canvas handle owns the press (it selects instead).
    pub(super) fn release_point_editor(&mut self, cx: &mut Context<Self>) {
        if self.active_mesh_drag.is_some() || self.active_field_drag.is_some() {
            return;
        }
        if self.selected_mesh_point.take().is_some() {
            self.mesh_color_target = MeshColorTarget::Background;
            self.sync_point_spread(cx);
            self.active_point_list().clone().update(cx, |list, cx| {
                list.open_editor(None, cx);
                list.select(None, cx);
            });
            cx.notify();
        }
    }

    /// Push `mesh_background` into its control after a host-side change (tab swap, reset).
    pub(super) fn sync_mesh_background_stop(&mut self, cx: &mut Context<Self>) {
        let color = self.mesh_background;
        self.mesh_background_stop.update(cx, |stop, cx| stop.set_color(color, cx));
    }
}

/// Freeform's point cap (also the list's count limit).
pub(super) const FREEFORM_MAX_POINTS: usize = 16;
