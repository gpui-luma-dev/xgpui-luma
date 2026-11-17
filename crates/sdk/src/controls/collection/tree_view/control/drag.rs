use crate::infra::locale::{LocaleContext, LogicalStyled};
use gpui::{App, Context, Div, Render, SharedString, Stateful, WeakEntity, Window, div, px, relative};
use crate::infra::drag_drop::{DragDropElementExt, DragDropEvent, bind_drag_source};
use super::super::drag::TreeDrag;
use super::super::{TreeDropLocation, TreeDropPosition, TreeDropProposal};
use super::*;

struct DragPreview<T: Clone + Send + Sync + 'static> {
    label: SharedString,
    template: std::sync::Arc<dyn super::super::TreeViewTemplate<T>>,
    size: crate::theme::ControlSize,
}
impl<T: Clone + Send + Sync + 'static> Render for DragPreview<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("tree-drag-preview")
            .debug_selector(|| "tree-drag-preview".to_owned())
            .child(self.template.render_drag_preview(self.label.clone(), self.size, window, cx))
    }
}

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    fn native_drag_event(&mut self, event: DragDropEvent<WeakEntity<Self>, SharedString>, cx: &mut Context<Self>) {
        if matches!(event, DragDropEvent::DragStarted { .. }) {
            self.drop_layout.cancel();
            self.drag_revision = Some(self.revision);
        }
        self.pressed_node_id = None;
        if let Some(event) = super::super::drag::project_event(event, None) {
            cx.emit(TreeViewEvent::DragDrop(event));
        }
        cx.notify();
    }

    fn proposal(
        target: &WeakEntity<Self>,
        drag: &TreeDrag<T>,
        location: TreeDropLocation,
        cx: &App,
    ) -> Option<TreeDropProposal<T>> {
        let entity = target.upgrade()?;
        let tree = entity.read(cx);
        let scope = tree.model.drag_drop.as_ref()?.scope;
        let inner = drag.propose(scope, target.clone(), location.anchor_id.clone())?;
        Some(TreeDropProposal {
            inner,
            node_id: drag.keys().first()?.clone(),
            location,
            scope,
            target_revision: tree.revision,
        })
    }

    fn bind_drop_zone(
        &self,
        surface: Stateful<Div>,
        location: TreeDropLocation,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Some(config) = self.model.drag_drop.clone() else {
            return surface;
        };
        let target = cx.entity().downgrade();
        let admit_target = target.clone();
        let admit_location = location.clone();
        let style_target = target.clone();
        let style_location = location.clone();
        let color = self.model.template.drop_color(self.model.size);
        let scale = self.model.template.resolve_scale(self.model.size, 1.0);
        surface
            .can_drop(move |value, window, cx| {
                if admit_location.anchor_id.is_none()
                    && admit_target
                        .upgrade()
                        .is_some_and(|target| target.read(cx).row_at(window.mouse_position()).is_some())
                {
                    return false;
                }
                value
                    .downcast_ref::<TreeDrag<T>>()
                    .and_then(|drag| Self::proposal(&admit_target, drag, admit_location.clone(), cx))
                    .is_some_and(|p| p.validate(cx).is_ok())
            })
            .drag_over::<TreeDrag<T>>(move |style, drag, window, cx| {
                if style_location.anchor_id.is_none()
                    && style_target
                        .upgrade()
                        .is_some_and(|target| target.read(cx).row_at(window.mouse_position()).is_some())
                {
                    return style;
                }
                if Self::proposal(&style_target, drag, style_location.clone(), cx)
                    .is_some_and(|p| p.validate(cx).is_ok())
                {
                    style.border_color(color).bg(color.opacity(scale.drag_opacity))
                } else {
                    style
                }
            })
            .on_drop(move |drag: &TreeDrag<T>, window, cx| {
                cx.stop_propagation();
                if let Some(proposal) = Self::proposal(&target, drag, location.clone(), cx) {
                    match proposal.validate(cx) {
                        Ok(()) => {
                            (config.on_drop)(proposal.clone(), window, cx);
                            if proposal.is_active() {
                                proposal.rejected("owner did not accept the drop", cx);
                            }
                        }
                        Err(error) => proposal.rejected(error, cx),
                    }
                }
            })
    }

    // Capture only effective displayed selection; closing rows are excluded.
    fn drag_roots(&self, id: &SharedString) -> Vec<SharedString> {
        if !self.model.drag_drop.as_ref().is_some_and(|config| config.drag_selected) || !self.selected_ids.contains(id)
        {
            return vec![id.clone()];
        }
        let visible = self.visible_ids();
        let selected: HashSet<_> = visible
            .iter()
            .filter(|key| self.selected_ids.contains(*key) && self.model.index.nodes[*key].enabled)
            .collect();
        visible
            .iter()
            .filter(|key| {
                if !selected.contains(key) {
                    return false;
                }
                let mut parent = self.model.index.nodes[*key].parent.as_ref();
                while let Some(id) = parent {
                    if selected.contains(&id) {
                        return false;
                    }
                    parent = self.model.index.nodes[id].parent.as_ref();
                }
                true
            })
            .cloned()
            .collect()
    }

    pub(super) fn bind_row_drag(
        &self,
        mut row: Stateful<Div>,
        id: SharedString,
        label: SharedString,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Some(config) = &self.model.drag_drop else {
            return row;
        };
        let Some(node) = self.flat_cache.get(index) else {
            return row;
        };
        if !self.model.enabled || !node.enabled {
            return row;
        }
        let roots = self.drag_roots(&id);
        let label = if roots.len() > 1 {
            format!("{} items", roots.len()).into()
        } else if let Some(root) = roots.first().filter(|root| *root != &id) {
            self.flat_index_for_id(root).map_or(label.clone(), |i| self.flat_cache[i].label.clone())
        } else {
            label
        };
        if let Some(drag) = TreeDrag::new(config.scope, cx.entity().downgrade(), roots) {
            let template = self.model.template.clone();
            let size = self.model.size;
            row = bind_drag_source(
                row,
                drag,
                move |_, _, _, cx| cx.new(|_| DragPreview { label: label.clone(), template: template.clone(), size }),
                cx,
                Self::native_drag_event,
            );
        }
        if !cx.has_active_drag() {
            return row;
        }
        let parent_id = self.model.index.nodes.get(&id).and_then(|n| n.parent.clone());
        for position in [TreeDropPosition::Before, TreeDropPosition::Into, TreeDropPosition::After] {
            if position == TreeDropPosition::Into && !node.has_children {
                continue;
            }
            let location = TreeDropLocation {
                parent_id: if position == TreeDropPosition::Into {
                    Some(id.clone())
                } else {
                    parent_id.clone()
                },
                anchor_id: Some(id.clone()),
                position,
            };
            let boundary = if node.has_children { 0.25 } else { 0.5 };
            let zone = div()
                .id(("drop-zone", position as usize))
                .absolute()
                .start(
                    px(node.depth as f32 * self.model.template.resolve_scale(self.model.size, 1.0).indentation_width),
                    LocaleContext::current(cx).direction,
                )
                .end(px(0.0), LocaleContext::current(cx).direction)
                .border_color(gpui::transparent_black());
            #[cfg(any(test, feature = "test-support"))]
            let zone = {
                let debug_tree_id = self.model.id.clone();
                let debug_node_id = id.clone();
                zone.debug_selector(move || format!("{debug_tree_id}-{debug_node_id}-{position:?}"))
            };
            let zone = match position {
                TreeDropPosition::Before => zone.top_0().bottom(relative(1.0 - boundary)).border_t(px(self
                    .model
                    .template
                    .resolve_scale(self.model.size, 1.0)
                    .drag_insert_width)),
                TreeDropPosition::After => zone.top(relative(1.0 - boundary)).bottom_0().border_b(px(self
                    .model
                    .template
                    .resolve_scale(self.model.size, 1.0)
                    .drag_insert_width)),
                TreeDropPosition::Into => zone.top(relative(boundary)).bottom(relative(boundary)).border(px(self
                    .model
                    .template
                    .resolve_scale(self.model.size, 1.0)
                    .drag_marker_width)),
            };
            row = row.child(self.bind_drop_zone(zone, location, cx));
        }
        row
    }

    /// Root append fills the blank viewport; rows cover it with their own zones.
    pub(super) fn bind_drag_viewport(&self, body: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        if self.model.drag_drop.is_none() {
            return body;
        }
        let location = TreeDropLocation { parent_id: None, anchor_id: None, position: TreeDropPosition::Into };
        let body = self.bind_drop_zone(body, location, cx);
        let target = cx.entity().downgrade();
        body.on_drag_move(move |event: &gpui::DragMoveEvent<TreeDrag<T>>, window, cx| {
            let drag = event.drag(cx).clone();
            let _ = target.update(cx, |tree, cx| tree.schedule_drag_scroll(drag, window, cx));
        })
        .cancel_drag_on_escape()
    }

    fn row_at(&self, point: gpui::Point<gpui::Pixels>) -> Option<(usize, gpui::Bounds<gpui::Pixels>)> {
        let start = self.list_state.logical_scroll_top().item_ix;
        for i in start..self.flat_cache.len() {
            let Some(bounds) = self.list_state.bounds_for_item(i) else {
                break;
            };
            if bounds.contains(&point) {
                return Some((i, bounds));
            }
            if bounds.top() > point.y {
                break;
            }
        }
        None
    }

    fn pointer_location(&self, pointer: gpui::Point<gpui::Pixels>) -> Option<TreeDropLocation> {
        if !self.list_state.viewport_bounds().contains(&pointer) {
            return None;
        }
        if let Some((i, bounds)) = self.row_at(pointer) {
            let row = &self.flat_cache[i];
            if !row.enabled {
                return None;
            }
            let fraction = (pointer.y - bounds.top()).as_f32() / bounds.size.height.as_f32().max(1.0);
            let boundary = if row.has_children { 0.25 } else { 0.5 };
            let position = if fraction < boundary {
                TreeDropPosition::Before
            } else if fraction >= 1.0 - boundary {
                TreeDropPosition::After
            } else {
                TreeDropPosition::Into
            };
            Some(TreeDropLocation {
                parent_id: if position == TreeDropPosition::Into {
                    Some(row.id.clone())
                } else {
                    self.model.index.nodes.get(&row.id).and_then(|n| n.parent.clone())
                },
                anchor_id: Some(row.id.clone()),
                position,
            })
        } else {
            Some(TreeDropLocation { parent_id: None, anchor_id: None, position: TreeDropPosition::Into })
        }
    }

    fn schedule_drag_scroll(&mut self, drag: TreeDrag<T>, window: &mut Window, cx: &mut Context<Self>) {
        if self.drag_scroll_scheduled {
            return;
        }
        self.drag_scroll_scheduled = true;
        let owner = cx.entity().downgrade();
        window.on_next_frame(move |window, cx| {
            // Resolve and validate outside an entity update: host policy may read
            // either tree, and rows under a stationary pointer change on scroll.
            let accepted = window.is_window_hovered()
                && cx.has_active_drag()
                && owner
                    .upgrade()
                    .and_then(|entity| entity.read(cx).pointer_location(window.mouse_position()))
                    .and_then(|location| Self::proposal(&owner, &drag, location, cx))
                    .is_some_and(|proposal| proposal.validate(cx).is_ok());
            let _ = owner.update(cx, |tree, cx| {
                tree.drag_scroll_scheduled = false;
                if !accepted {
                    tree.drag_scroll_last_frame = None;
                    return;
                }
                let bounds = tree.list_state.viewport_bounds();
                let pointer = window.mouse_position();
                let now = std::time::Instant::now();
                let elapsed = tree
                    .drag_scroll_last_frame
                    .replace(now)
                    .map_or(std::time::Duration::from_secs_f32(1.0 / 60.0), |last| now - last);
                let delta = crate::infra::drag_drop::edge_scroll_delta(
                    (pointer.y - bounds.top()).as_f32(),
                    bounds.size.height.as_f32(),
                    elapsed,
                    540.0,
                );
                let offset = tree.list_state.scroll_px_offset_for_scrollbar().y;
                let maximum = tree.list_state.max_offset_for_scrollbar().y;
                let next = (-offset.as_f32() + delta).clamp(0.0, maximum.as_f32().max(0.0));
                if (next + offset.as_f32()).abs() < 0.01 {
                    tree.drag_scroll_last_frame = None;
                    return;
                }
                tree.list_state.scroll_by(px(next + offset.as_f32()));
                cx.notify();
                window.refresh();
                tree.schedule_drag_scroll(drag, window, cx);
            });
        });
    }
}
