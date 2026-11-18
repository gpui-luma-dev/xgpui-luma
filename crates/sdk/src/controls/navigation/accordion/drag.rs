//! Keyed same-Accordion transactions and native header handles.
use std::rc::Rc;
use gpui::{Context, Div, EntityId, Focusable, SharedString, Stateful, Window, div, prelude::*, px};
use crate::infra::drag_drop::{
    DragDropElementExt, DragDropEvent, DropProposal, KeyedDrag, KeyedDropTarget, bind_drag_source,
};
use super::{AccordionControl, AccordionEvent};

/// Shared lifecycle; Accordion currently emits same-control moves only.
pub type AccordionDragEvent = DragDropEvent<EntityId, SharedString>;
/// Synchronous proposal using stable item IDs and a before/append gap.
pub type AccordionDropProposal = DropProposal<EntityId, SharedString>;
pub(super) type AccordionDrag = KeyedDrag<EntityId, SharedString>;
pub(super) type DropHandler =
    Rc<dyn Fn(&mut AccordionControl, &AccordionDropProposal, &mut Window, &mut Context<AccordionControl>)>;

/// Native bindings supplied to custom templates when reordering is enabled.
/// Put each supplied handle inside its header, measure whole items by key, append the
/// targets above the items, and retain the root's input/scroll bindings.
pub struct AccordionDragBindings {
    pub root: Stateful<Div>,
    pub handles: Vec<Option<Stateful<Div>>>,
    pub targets: Vec<gpui::AnyElement>,
    pub bounds: super::geometry::ItemBounds,
    pub motion: crate::motion::DropLayoutMotion<SharedString>,
    pub scroll: gpui::ScrollHandle,
}

impl AccordionDragBindings {
    /// Measure the whole layout item before applying its motion transform.
    /// Custom templates can put the resulting items inside `motion.scope`.
    pub fn measure_item(
        bounds: &super::geometry::ItemBounds,
        id: SharedString,
        child: impl IntoElement,
    ) -> gpui::AnyElement {
        super::geometry::measure(bounds.clone(), id, child)
    }
}

pub(super) fn reorder_help() -> crate::controls::tooltip::Tooltip {
    crate::controls::tooltip::Tooltip::new("Drag to reorder")
}

impl AccordionControl {
    /// Enable native same-control moves; disabled items cannot start a drag.
    pub fn set_reordering(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.reordering = enabled;
        if !enabled {
            self.drag_tooltips.clear();
            self.hovered_drag_handle = None;
            self.cancel_drag(cx);
            self.drop_layout.cancel();
        }
        cx.notify();
    }
    /// Use an enclosing scroll viewport for drag edge scrolling and motion.
    /// Pass None to restore the Accordion's own viewport. The enclosing element
    /// must track this same handle; Accordion will not create a nested scroller.
    pub fn set_drag_scroll_handle(&mut self, handle: Option<gpui::ScrollHandle>, cx: &mut Context<Self>) {
        self.cancel_drag(cx);
        self.drop_layout.cancel();
        self.external_scroll = handle.is_some();
        self.drag_scroll = handle.unwrap_or_default();
        cx.notify();
    }

    /// Retained motion and resolved duration diagnostics.
    pub fn drop_layout_motion(&self) -> crate::motion::DropLayoutMotion<SharedString> {
        self.drop_layout.clone()
    }
    /// Current item order. Moving items retains their content renderers.
    pub fn items(&self) -> &[super::AccordionItem] {
        &self.model.items
    }
    /// Override automatic commits. Complete synchronously using commit_drop or
    /// reject_drop; an unanswered proposal is rejected.
    pub fn set_drop_handler(
        &mut self,
        handler: impl Fn(&mut Self, &AccordionDropProposal, &mut Window, &mut Context<Self>) + 'static,
    ) {
        self.drop_handler = Some(Rc::new(handler));
    }
    /// Atomically reorder live keyed items, preserving disclosure and content state.
    pub fn commit_drop(
        &mut self,
        proposal: &AccordionDropProposal,
        cx: &mut Context<Self>,
    ) -> Result<bool, SharedString> {
        let order = match self.drop_order(proposal, cx.entity_id()) {
            Ok(order) => order,
            Err(error) => {
                self.reject_drop(proposal, error.clone(), cx);
                return Err(error);
            }
        };
        let changed = order.iter().enumerate().any(|(i, j)| i != *j);
        if changed {
            self.drop_layout.start();
            let focused = self.focused_item_index.and_then(|i| self.model.items.get(i)).map(|i| i.id.clone());
            self.model.items = order.iter().map(|i| self.model.items[*i].clone()).collect();
            self.transitions = order.iter().map(|i| self.transitions[*i].clone()).collect();
            self.content_heights_px = order.iter().map(|i| self.content_heights_px[*i]).collect();
            self.focused_item_index = focused.and_then(|id| self.model.items.iter().position(|i| i.id == id));
            self.hovered_item_index = None;
            self.pressed_item_index = None;
            self.item_bounds.borrow_mut().clear();
        }
        for event in proposal.committed(changed, proposal.keys().to_vec()) {
            self.drag_event(event, cx);
        }
        cx.notify();
        Ok(changed)
    }
    /// Reject a live local proposal without mutating items.
    pub fn reject_drop(
        &mut self,
        proposal: &AccordionDropProposal,
        error: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        if self.active_drag.as_ref().is_some_and(|drag| proposal.is_from(drag)) {
            for event in proposal.rejected(error) {
                self.drag_event(event, cx);
            }
        }
    }
    fn drop_order(&self, proposal: &AccordionDropProposal, id: EntityId) -> Result<Vec<usize>, SharedString> {
        if !self.model.enabled
            || !self.model.reordering
            || !proposal.is_active()
            || *proposal.source() != id
            || *proposal.target() != id
            || !self.active_drag.as_ref().is_some_and(|drag| proposal.is_from(drag))
        {
            return Err("Accordion drag is disabled, foreign or ended".into());
        }
        if proposal.keys().iter().any(|id| !self.model.items.iter().any(|item| &item.id == id && item.enabled)) {
            return Err("Dragged Accordion item is missing or disabled".into());
        }
        crate::infra::drag_drop::reorder_indices(
            &self.model.items.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            proposal.keys(),
            proposal.before(),
        )
    }
    pub(super) fn cancel_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(event) = self.active_drag.take().and_then(|drag| drag.cancel()) {
            self.drag_event(event, cx);
        }
    }
    fn drag_event(&mut self, event: AccordionDragEvent, cx: &mut Context<Self>) {
        if matches!(event, DragDropEvent::DragStarted { .. }) {
            self.drop_layout.cancel();
            self.pressed_item_index = None;
        }
        if matches!(event, DragDropEvent::DragEnded { .. }) {
            self.active_drag = None;
        }
        cx.emit(AccordionEvent::DragDrop(event));
        cx.notify();
    }
    fn drop_item(&mut self, proposal: AccordionDropProposal, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.drop_handler.clone() {
            handler(self, &proposal, window, cx);
            if proposal.is_active() {
                self.reject_drop(&proposal, "owner did not accept the drop", cx);
            }
        } else {
            let _ = self.commit_drop(&proposal, cx);
        }
    }
    pub(super) fn drag_bindings(&mut self, cx: &mut Context<Self>) -> Option<AccordionDragBindings> {
        if !self.model.enabled || !self.model.reordering {
            return None;
        }
        let scope = cx.entity_id();
        let look = self.model.template.resolve_drag();
        self.drop_layout.configure_transition(&self.model.motion.profile().drop_layout);
        self.drop_layout.set_motion_enabled(self.model.motion_enabled);
        let mut root = div().id(self.model.id.clone()).cancel_drag_on_escape();
        if !self.external_scroll {
            root = root.overflow_y_scroll().track_scroll(&self.drag_scroll);
        }
        root = crate::infra::drag_drop::bind_edge_scroll::<EntityId, SharedString>(
            root,
            self.drag_scroll.clone(),
            gpui::Axis::Vertical,
            scope,
            self.drag_scroll_pending.clone(),
        );
        let handles = self
            .model
            .items
            .iter()
            .map(|item| {
                if !item.enabled {
                    return None;
                }
                let drag = AccordionDrag::new(scope, scope, vec![item.id.clone()])?;
                let owner = cx.weak_entity();
                let label = item.trigger.label.clone().unwrap_or_else(|| item.id.clone());
                let preview_look = look.clone();
                let focus_id = item.id.clone();
                let id = item.id.clone();
                let hover_id = id.clone();
                let handle = div()
                    .id("drag-handle")
                    .cursor_grab()
                    .aria_label("Drag to reorder")
                    .on_hover(cx.listener(move |control, hovered: &bool, _, cx| {
                        if *hovered {
                            control.hovered_drag_handle = Some(hover_id.clone());
                        } else if control.hovered_drag_handle.as_ref() == Some(&hover_id) {
                            control.hovered_drag_handle = None;
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |control, _, window, cx| {
                            let index = control.model.items.iter().position(|i| i.id == focus_id);
                            control.set_focused_item_index(index, cx);
                            control.focus_handle(cx).focus(window, cx);
                            cx.stop_propagation();
                        }),
                    )
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .debug_selector(move || format!("accordion-drag-{id}"));
                let theme = self.model.template.drag_tooltip_theme();
                let tooltip = self.drag_tooltips.entry(item.id.clone()).or_insert_with(|| {
                    crate::infra::attachments::TooltipHandle::new_with_theme(reorder_help(), theme, cx)
                });
                let handle = tooltip.render(
                    handle,
                    crate::theme::InteractionState {
                        hovered: self.hovered_drag_handle.as_ref() == Some(&item.id),
                        disabled: self.active_drag.is_some(),
                        ..Default::default()
                    },
                    cx,
                );
                Some(
                    bind_drag_source(
                        handle,
                        drag,
                        move |drag, offset, _, cx| {
                            let _ = owner.update(cx, |control, cx| {
                                control.active_drag = Some(drag.clone());
                                cx.notify();
                            });
                            cx.new(|_| crate::infra::drag_drop::DragPreview {
                                label: label.clone(),
                                offset,
                                look: preview_look.clone(),
                                selector: "accordion-drag-preview",
                            })
                        },
                        cx,
                        Self::drag_event,
                    )
                    .cancel_drag_on_escape(),
                )
            })
            .collect();
        let mut targets = Vec::new();
        if let Some(drag) = self.active_drag.as_ref() {
            // One target per canonical gap, spanning adjacent whole-item halves.
            let measured = self.item_bounds.borrow();
            if let Some(bounds) =
                self.model.items.iter().map(|i| measured.get(&i.id).copied()).collect::<Option<Vec<_>>>()
            {
                let top = bounds.first().map_or(px(0.), |b| b.top());
                let positions: Vec<_> =
                    bounds.iter().map(|b| ((b.top() - top).as_f32(), (b.bottom() - top).as_f32())).collect();
                for index in 0..=positions.len() {
                    let previous = index.checked_sub(1).map(|i| positions[i]);
                    let next = positions.get(index).copied();
                    let start = previous.map_or(0., |(t, b)| (t + b) / 2.);
                    let end = next.map_or_else(|| previous.map_or(24., |(_, b)| b), |(t, b)| (t + b) / 2.);
                    let marker = match (previous, next) {
                        (Some((_, b)), Some((t, _))) => (b + t) / 2.,
                        (None, Some((t, _))) => t,
                        (Some((_, b)), None) => b,
                        _ => continue,
                    };
                    let before = self.model.items.get(index).map(|i| i.id.clone());
                    let name: SharedString = before.as_ref().map_or_else(
                        || "accordion-gap-append".into(),
                        |id| format!("accordion-gap-before-{id}").into(),
                    );
                    let group: SharedString = format!("{scope}-{name}").into();
                    let valid = drag.propose(scope, scope, before.clone()).is_some_and(|p| {
                        self.drop_order(&p, scope).is_ok_and(|order| order.iter().enumerate().any(|(i, j)| i != *j))
                    });
                    let color = if valid { look.valid_marker } else { look.invalid_marker };
                    let marker_name = name.clone();
                    let marker = div()
                        .id("marker")
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(px(marker - start - look.geometry.drag_marker_width / 2.0))
                        .h(px(look.geometry.drag_marker_width))
                        .bg(color)
                        .opacity(0.)
                        .debug_selector(move || format!("{marker_name}-marker"))
                        .drag_over::<AccordionDrag>(|style, _, _, _| style)
                        .group_drag_over::<AccordionDrag>(group.clone(), |style| style.opacity(1.));
                    let target_name = name.clone();
                    let target = div()
                        .id(name)
                        .group(group)
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(px(start))
                        .h(px((end - start).max(0.)))
                        .debug_selector(move || target_name.to_string())
                        .child(marker);
                    let owner = cx.weak_entity();
                    let anchor = before.clone();
                    let target = crate::infra::drag_drop::bind_feedback(
                        target,
                        look.clone(),
                        scope,
                        move |drag: &AccordionDrag, app| {
                            owner.upgrade().is_some_and(|owner| {
                                drag.propose(scope, scope, anchor.clone()).is_some_and(|p| {
                                    owner
                                        .read(app)
                                        .drop_order(&p, scope)
                                        .is_ok_and(|o| o.iter().enumerate().any(|(i, j)| i != *j))
                                })
                            })
                        },
                    );
                    targets.push(
                        KeyedDropTarget::new(scope, scope, before).bind(target, cx, Self::drop_item).into_any_element(),
                    );
                }
            }
        }
        Some(AccordionDragBindings {
            root,
            handles,
            targets,
            bounds: self.item_bounds.clone(),
            motion: self.drop_layout.clone(),
            scroll: self.drag_scroll.clone(),
        })
    }
}
