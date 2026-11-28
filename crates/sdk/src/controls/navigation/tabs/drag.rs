//! Same-strip keyed transactions. Content and focus stay owned by Tabs.
use std::rc::Rc;
use gpui::{Context, Div, EntityId, SharedString, Stateful, Window, div, prelude::*, px};
use crate::infra::drag_drop::{
    DragDropElementExt, DragDropEvent, DropProposal, KeyedDrag, KeyedDropTarget, bind_drag_source,
};
use super::{Tabs, TabsEvent};

/// Shared lifecycle for a single tab strip; transfer phases are not emitted.
pub type TabsDragEvent = DragDropEvent<EntityId, SharedString>;
/// Live synchronous request to reorder stable IDs within one strip.
pub type TabsDropProposal = DropProposal<EntityId, SharedString>;
pub(super) type TabDrag = KeyedDrag<EntityId, SharedString>;
pub(super) type DropHandler = Rc<dyn Fn(&mut Tabs, &TabsDropProposal, &mut Window, &mut Context<Tabs>)>;

impl Tabs {
    /// Enable same-strip reordering. Disabled and dropdown-trigger tabs cannot
    /// initiate a drag; gaps beside them remain valid insertion destinations.
    pub fn set_reordering(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.reordering = enabled;
        self.group.update(cx, |_, cx| cx.notify());
        if !enabled {
            self.cancel_drag(cx);
        }
        cx.notify();
    }

    /// Retained keyed post-drop geometry and running transitions.
    pub fn drop_layout_motion(&self) -> crate::motion::DropLayoutMotion<SharedString> {
        self.drop_layout.clone()
    }

    pub fn items(&self) -> &[super::TabsItem] {
        &self.items
    }
    /// Override automatic commits. Complete synchronously with commit_drop or
    /// reject_drop; an unanswered proposal is rejected without changing data.
    pub fn set_drop_handler(
        &mut self,
        handler: impl Fn(&mut Self, &TabsDropProposal, &mut Window, &mut Context<Self>) + 'static,
    ) {
        self.drop_handler = Some(Rc::new(handler));
    }
    /// Validate current IDs and atomically reorder without replacing retained content.
    pub fn commit_drop(&mut self, proposal: &TabsDropProposal, cx: &mut Context<Self>) -> Result<bool, SharedString> {
        let result = self.drop_order(proposal, cx.entity_id());
        let order = match result {
            Ok(order) => order,
            Err(error) => {
                self.reject_drop(proposal, error.clone(), cx);
                return Err(error);
            }
        };
        let changed = order.iter().enumerate().any(|(i, index)| i != *index);
        if changed {
            self.drop_layout.start();
            self.items = order.into_iter().map(|index| self.items[index].clone()).collect();
            // Unlike set_items, a move must not restart/finish body transitions,
            // recreate content, or change the selected/keyboard-active identity.
            self.item_bounds.clear();
            self.indicator_motion.clear_geometry();
            self.group.update(cx, |group, cx| group.set_items(self.items.clone(), cx));
        }
        for event in proposal.committed(changed, proposal.keys().to_vec()) {
            self.drag_event(event, cx);
        }
        cx.notify();
        Ok(changed)
    }
    /// Reject a live proposal from this strip without changing its items.
    pub fn reject_drop(&mut self, proposal: &TabsDropProposal, error: impl Into<SharedString>, cx: &mut Context<Self>) {
        if self.active_drag.as_ref().is_some_and(|drag| proposal.is_from(drag)) {
            for event in proposal.rejected(error) {
                self.drag_event(event, cx);
            }
        }
    }
    fn drop_order(&self, p: &TabsDropProposal, entity_id: EntityId) -> Result<Vec<usize>, SharedString> {
        if !self.reordering
            || !self.enabled
            || !p.is_active()
            || *p.source() != entity_id
            || *p.target() != entity_id
            || !self.active_drag.as_ref().is_some_and(|drag| p.is_from(drag))
        {
            return Err("Tab drag is disabled, foreign or ended".into());
        }
        if p.keys()
            .iter()
            .any(|id| !self.items.iter().any(|item| item.id() == id && item.enabled && !item.is_dropdown_trigger()))
        {
            return Err("Dragged tab is missing, disabled or a dropdown trigger".into());
        }
        crate::infra::drag_drop::reorder_indices(
            &self.items.iter().map(|item| item.id().clone()).collect::<Vec<_>>(),
            p.keys(),
            p.before(),
        )
    }
    pub(super) fn cancel_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(event) = self.active_drag.take().and_then(|drag| drag.cancel()) {
            self.drag_event(event, cx);
        }
    }
    fn drag_event(&mut self, event: TabsDragEvent, cx: &mut Context<Self>) {
        if matches!(event, DragDropEvent::DragStarted { .. }) {
            self.drop_layout.cancel();
        }
        if matches!(event, DragDropEvent::DragEnded { .. }) {
            self.active_drag = None;
        }
        cx.emit(TabsEvent::DragDrop(event));
        self.group.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
    fn drop_tab(&mut self, proposal: TabsDropProposal, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.drop_handler.clone() {
            handler(self, &proposal, window, cx);
            if proposal.is_active() {
                self.reject_drop(&proposal, "owner did not accept the drop", cx);
            }
        } else {
            let _ = self.commit_drop(&proposal, cx);
        }
    }
    /// Bind overflow scrolling for native/custom strip templates.
    pub fn bind_drag_strip(&self, surface: Stateful<Div>, cx: &Context<Self>) -> Stateful<Div> {
        if !self.reordering || !self.enabled {
            return surface;
        }
        crate::infra::drag_drop::bind_edge_scroll::<EntityId, SharedString>(
            surface.overflow_x_scroll().track_scroll(&self.drag_scroll),
            self.drag_scroll.clone(),
            gpui::Axis::Horizontal,
            cx.entity_id(),
            self.drag_scroll_pending.clone(),
        )
        .cancel_drag_on_escape()
    }
    /// Bind a native keyed source to a themed tab button.
    pub fn bind_tab_drag(
        &self,
        mut surface: Stateful<Div>,
        id: &SharedString,
        label: SharedString,
        style: std::sync::Arc<xgpui_luma_style::Resolved>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        if !self.reordering || !self.enabled {
            return surface;
        }
        surface = surface.flex_none();
        let Some(index) = self.items.iter().position(|item| item.id() == id) else {
            return surface;
        };
        #[cfg(any(test, feature = "test-support"))]
        {
            let id = id.clone();
            surface = surface.debug_selector(move || format!("tab-drag-{id}"));
        }
        let item = &self.items[index];
        let scope = cx.entity_id();
        if item.enabled
            && !item.is_dropdown_trigger()
            && let Some(drag) = TabDrag::new(scope, scope, vec![id.clone()])
        {
            let preview_style = style.clone();
            let owner = cx.weak_entity();
            surface = bind_drag_source(
                surface.cursor_grab(),
                drag,
                move |drag, offset, _, cx| {
                    let _ = owner.update(cx, |tabs, cx| {
                        tabs.active_drag = Some(drag.clone());
                        cx.notify();
                    });
                    cx.new(|_| crate::infra::drag_drop::DragPreview {
                        label: label.clone(),
                        offset,
                        style: preview_style.clone(),
                        selector: "tabs-drag-preview",
                    })
                },
                cx,
                Self::drag_event,
            );
        }
        surface.cancel_drag_on_escape()
    }

    /// Bind one target per insertion gap, spanning the adjacent tab halves.
    /// Call once after adding the tab buttons, using the strip's gap and inset.
    pub fn bind_tab_drop_targets(
        &self,
        mut strip: Stateful<Div>,
        gap: f32,
        inset: f32,
        style: std::sync::Arc<xgpui_luma_style::Resolved>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let scope = cx.entity_id();
        let Some(drag) = self.active_drag.as_ref().filter(|drag| drag.accepts(scope)) else {
            return strip;
        };
        if !self.reordering || !self.enabled {
            return strip;
        }
        let Some(bounds) = self
            .item_bounds
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()
            .filter(|b| b.len() == self.items.len() && !b.is_empty())
        else {
            return strip;
        };
        let mut left = inset;
        let positions: Vec<_> = bounds
            .iter()
            .map(|bounds| {
                let start = left;
                left += bounds.size.width.as_f32() + gap;
                (start, left - gap)
            })
            .collect();
        for index in 0..=positions.len() {
            let previous = index.checked_sub(1).map(|i| positions[i]);
            let next = positions.get(index).copied();
            let start = previous.map_or(0., |(l, r)| (l + r) / 2.);
            let end = next.map_or_else(|| previous.map_or(inset, |(_, r)| r + inset), |(l, r)| (l + r) / 2.);
            let marker = match (previous, next) {
                (Some((_, r)), Some((l, _))) => (r + l) / 2.,
                (None, Some((l, _))) => l,
                (Some((_, r)), None) => r,
                _ => continue,
            };
            let before = self.items.get(index).map(|item| item.id().clone());
            let name: SharedString = before
                .as_ref()
                .map_or_else(|| "tab-gap-append".into(), |key| format!("tab-gap-before-{key}").into());
            let group: SharedString = format!("{}-{name}", scope).into();
            let valid = drag.propose(scope, scope, before.clone()).is_some_and(|proposal| {
                self.drop_order(&proposal, scope).is_ok_and(|order| order.iter().enumerate().any(|(i, j)| i != *j))
            });
            let color = crate::infra::drag_drop::drag_marker_color(&style, valid);
            let marker = div()
                .id("marker")
                .absolute()
                .top_0()
                .bottom_0()
                .left(px(marker - start - crate::theme::stylesheet::SharedGeometry::sdk().drag_marker_width / 2.0))
                .w(px(crate::theme::stylesheet::SharedGeometry::sdk().drag_marker_width))
                .bg(color)
                .opacity(0.)
                // GPUI only resolves group drag styles for elements with a hitbox.
                // Register drag styling on the marker without adding a drop target.
                .drag_over::<TabDrag>(|style, _, _, _| style)
                .group_drag_over::<TabDrag>(group.clone(), |style| style.opacity(1.));
            #[cfg(any(test, feature = "test-support"))]
            let marker = {
                let name = name.clone();
                marker.debug_selector(move || format!("{name}-marker"))
            };
            let target = div()
                .id(name.clone())
                .group(group)
                .absolute()
                .top_0()
                .bottom_0()
                .left(px(start))
                .w(px((end - start).max(0.)))
                .child(marker);
            #[cfg(any(test, feature = "test-support"))]
            let target = {
                let name = name.clone();
                target.debug_selector(move || name.to_string())
            };
            let owner = cx.weak_entity();
            let anchor = before.clone();
            let target =
                crate::infra::drag_drop::bind_feedback(target, style.clone(), scope, move |drag: &TabDrag, app| {
                    owner.upgrade().is_some_and(|owner| {
                        drag.propose(scope, scope, anchor.clone()).is_some_and(|p| {
                            owner
                                .read(app)
                                .drop_order(&p, scope)
                                .is_ok_and(|order| order.iter().enumerate().any(|(i, j)| i != *j))
                        })
                    })
                });
            strip = strip.child(KeyedDropTarget::new(scope, scope, before).bind(target, cx, Self::drop_tab));
        }
        strip
    }
}
