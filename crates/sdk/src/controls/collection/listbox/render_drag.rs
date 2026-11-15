//! Render adapters for persistent controls and the raw host-owned compatibility API.

use std::hash::Hash;
use gpui::{App, Context, Div, Entity, EntityId, Hsla, Render, Stateful, Window, prelude::*};
use crate::controls::tooltip::TooltipLook;
use crate::infra::drag_drop::{DragDropEvent, DropProposal, KeyedDrag, KeyedDropTarget, bind_drag_source};
use super::{ListBoxAxis, ListBoxDragDropBinding, ListBoxScrollHandle, ListBoxState};

type Surface = Stateful<Div>;

type DragRow<'a, M, T, K> = Box<dyn Fn(Surface, &T, &K, &TooltipLook, &mut Context<M>) -> Surface + 'a>;
type DropTarget<'a, M, K> = Box<dyn Fn(Surface, Option<K>, &mut Context<M>) -> Surface + 'a>;

/// Per-render drag adapters. The SDK composes insertion targets and viewport
/// wiring; these callbacks preserve persistent and raw drag transaction APIs.
pub struct ListBoxDragDropRenderBindings<'a, M: 'static, T, K> {
    pub(super) row: DragRow<'a, M, T, K>,
    pub(super) target: DropTarget<'a, M, K>,
    pub(super) viewport: Box<dyn Fn(Surface, EntityId, ListBoxAxis) -> Surface + 'a>,
    pub(super) highlight: Box<dyn Fn(Surface, EntityId, Hsla) -> Surface + 'a>,
}

impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxDragDropRenderBindings<'a, M, T, K> {
    /// Adapt a persistent control without retaining its owner or rebuilding callbacks.
    pub fn from_control(
        binding: ListBoxDragDropBinding<M, K>,
        state: &'a ListBoxState<T, K>,
        scroll: &'a ListBoxScrollHandle<K>,
        label: fn(&T) -> gpui::SharedString,
    ) -> Self {
        let row_binding = binding.clone();
        let target_binding = binding.clone();
        let scroll_binding = binding.clone();
        Self {
            row: Box::new(move |surface, item, key, preview, cx| {
                row_binding.clone().preview_style(preview.clone()).bind_row(
                    surface,
                    state.drag_keys(key),
                    label(item),
                    cx,
                )
            }),
            target: Box::new(move |surface, before, cx| target_binding.bind_target(surface, before, cx)),
            viewport: Box::new(move |surface, _, axis| scroll_binding.bind_viewport(surface, scroll, axis)),
            highlight: Box::new(move |surface, _, color| binding.highlight(surface, color)),
        }
    }

    /// Preserve raw host-owned drag scope, custom previews, and proposal completion.
    pub fn custom<S: Clone + 'static, P: Render + 'static>(
        state: &'a ListBoxState<T, K>,
        scroll: &'a ListBoxScrollHandle<K>,
        list: S,
        on_drop: fn(&mut M, DropProposal<S, K>, &mut Window, &mut Context<M>),
        on_event: fn(&mut M, DragDropEvent<S, K>, &mut Context<M>),
        preview: impl Fn(&T, &[K], &mut Window, &mut App) -> Entity<P> + 'static,
    ) -> Self
    where
        T: Clone,
    {
        let source = list.clone();
        let preview = std::rc::Rc::new(preview);
        let target_events = on_event;
        // Read scope and theme at build time so fluent option order is immaterial.
        Self {
            // Host-supplied previews keep their own styling.
            row: Box::new(move |surface, item, key, _preview, cx| {
                let Some(drag) =
                    state.drag_keys(key).and_then(|keys| KeyedDrag::new(cx.entity_id(), source.clone(), keys))
                else {
                    return surface;
                };
                let item = item.clone();
                let preview = preview.clone();
                bind_drag_source(
                    surface.cursor_grab(),
                    drag,
                    move |drag, _, window, cx| preview(&item, drag.keys(), window, cx),
                    cx,
                    on_event,
                )
            }),
            target: Box::new(move |surface, before, cx| {
                KeyedDropTarget::new(cx.entity_id(), list.clone(), before).bind(surface, cx, {
                    let on_event = target_events;
                    move |owner, proposal, window, cx| {
                        on_drop(owner, proposal.clone(), window, cx);
                        if proposal.is_active() {
                            for event in proposal.rejected("owner did not accept the drop") {
                                on_event(owner, event, cx);
                            }
                        }
                    }
                })
            }),
            viewport: Box::new(move |surface, scope, axis| {
                scroll.bind_drag_auto_scroll(surface, axis, move |drag: &KeyedDrag<S, K>| drag.accepts(scope))
            }),
            highlight: Box::new(move |surface, scope, color| {
                surface.drag_over::<KeyedDrag<S, K>>(move |style, drag, _, _| {
                    if drag.accepts(scope) {
                        style.border_color(color)
                    } else {
                        style
                    }
                })
            }),
        }
    }
}
