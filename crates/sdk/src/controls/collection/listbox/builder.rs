//! Shared borrowed listbox builder over persistent SDK state and bindings.

use std::hash::Hash;

use gpui::{
    AnyElement, App, Axis, Context, Div, ElementId, Entity, EntityId, Hsla, Render, Stateful, Window, div, prelude::*,
    px,
};
use crate::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxControl, ListBoxFlow, ListBoxInput, ListBoxLayout, ListBoxScrollHandle,
    ListBoxState, ListBoxVisibleItem, ListBoxVirtualization,
};
use crate::infra::drag_drop::{
    DragDropElementExt, DragDropEvent, DropEdge, DropProposal, DropZone, KeyedDrag, KeyedDropTarget, bind_drag_source,
};
use super::{ListBoxTemplate, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxRenderParts};

use crate::controls::tooltip::TooltipLook;

type Surface = Stateful<Div>;
type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);
type ItemRenderer<'a, T, K> = Box<dyn Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> AnyElement + 'a>;
type DragRow<'a, M, T, K> = Box<dyn Fn(Surface, &T, &K, &TooltipLook, &mut Context<M>) -> Surface + 'a>;
type DropTarget<'a, M, K> = Box<dyn Fn(Surface, Option<K>, &mut Context<M>) -> Surface + 'a>;

struct DragDropBindings<'a, M: 'static, T, K> {
    row: DragRow<'a, M, T, K>,
    target: DropTarget<'a, M, K>,
    viewport: Box<dyn Fn(Surface, EntityId, ListBoxAxis) -> Surface + 'a>,
    highlight: Box<dyn Fn(Surface, EntityId, Hsla) -> Surface + 'a>,
}

/// Shared fixed-size or content-sized list composition. Reuse SDK state and handles between renders.
/// Drag handlers and gap overlays are installed only for configured drag/drop.
pub struct ListBoxBuilder<'a, M: 'static, T, K> {
    motion_edits: crate::motion::MotionOverrides<crate::controls::listbox::ListBoxMotion>,
    motion_enabled: Option<bool>,
    raw_motion: crate::motion::MotionConfig<crate::controls::listbox::ListBoxMotion>,
    id: &'static str,
    state: &'a ListBoxState<T, K>,
    binding: &'a mut ListBoxBinding,
    scroll: &'a ListBoxScrollHandle<K>,
    layout: ListBoxLayout,
    axis: ListBoxAxis,
    item_width: f32,
    content_sized: bool,
    virtualization: ListBoxVirtualization,
    on_input: InputHandler<M, K>,
    item_id: fn(&K) -> ElementId,
    item: ItemRenderer<'a, T, K>,
    label: &'static str,
    empty: Option<AnyElement>,
    drag_drop: Option<DragDropBindings<'a, M, T, K>>,
    drop_layout: Option<crate::motion::DropLayoutMotion<K>>,
    motion_config: Option<&'a mut crate::motion::MotionConfig<crate::controls::listbox::ListBoxMotion>>,
}

impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    crate::forward_motion!(crate::controls::listbox::ListBoxMotion, edits motion_edits);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = Some(enabled);
        self
    }

    /// Reuse the host's existing SDK state and handles. Stable item IDs must be
    /// unique within this list and remain the same when items move or reorder.
    /// Layout and the template passed to `build` are explicit: no look's geometry
    /// or paint is installed as an SDK default. The content callback must keep
    /// the supplied bound surface in its returned element.
    #[allow(clippy::too_many_arguments)]
    pub fn new<E: IntoElement>(
        id: &'static str,
        state: &'a ListBoxState<T, K>,
        binding: &'a mut ListBoxBinding,
        scroll: &'a ListBoxScrollHandle<K>,
        on_input: InputHandler<M, K>,
        item_id: fn(&K) -> ElementId,
        layout: ListBoxLayout,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self {
            motion_edits: Default::default(),
            motion_enabled: None,
            raw_motion: Default::default(),
            id,
            state,
            binding,
            scroll,
            on_input,
            item_id,
            item: Box::new(move |surface, item_state, cx| item(surface, item_state, cx).into_any_element()),
            axis: ListBoxAxis::Vertical,
            item_width: 0.0,
            content_sized: false,
            virtualization: ListBoxVirtualization::default(),
            layout,
            label: id,
            empty: None,
            drag_drop: None,
            drop_layout: None,
            motion_config: None,
        }
    }

    /// Compose a persistent SDK control, including its registered drag/drop
    /// callbacks and configuration. No render-time callback registration occurs.
    pub fn from_control<E: IntoElement>(
        id: &'static str,
        control: &'a mut ListBoxControl<M, T, K>,
        layout: ListBoxLayout,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self::from_parts(id, control.render_parts(), layout, item)
    }

    /// Compose an already borrowed control without rebuilding its state or callbacks.
    pub fn from_parts<E: IntoElement>(
        id: &'static str,
        parts: ListBoxRenderParts<'a, M, T, K>,
        layout: ListBoxLayout,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self::new(id, parts.state, parts.binding, parts.scroll, parts.on_input, parts.item_id, layout, item)
            .virtualization(parts.virtualization)
            .motion_config(parts.motion_config)
            .drop_layout_motion(parts.drop_layout)
            .control_drag_drop(parts.drag_drop, parts.item_label)
    }

    fn motion_config(
        mut self,
        config: &'a mut crate::motion::MotionConfig<crate::controls::listbox::ListBoxMotion>,
    ) -> Self {
        self.motion_config = Some(config);
        self
    }

    /// Reuse SDK-owned drop motion for raw/host-owned composition.
    pub fn drop_layout_motion(mut self, motion: crate::motion::DropLayoutMotion<K>) -> Self {
        self.drop_layout = Some(motion);
        self
    }

    fn control_drag_drop(
        mut self,
        binding: Option<crate::controls::listbox::ListBoxDragDropBinding<M, K>>,
        label: fn(&T) -> gpui::SharedString,
    ) -> Self {
        if let Some(binding) = binding {
            let row_binding = binding.clone();
            let target_binding = binding.clone();
            let scroll_binding = binding.clone();
            let state = self.state;
            let scroll = self.scroll;
            self.drag_drop = Some(DragDropBindings {
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
            });
        }
        self
    }

    /// Set viewport and item geometry; the supplied template owns appearance.
    pub fn layout(mut self, layout: ListBoxLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set item flow and viewport dimensions without choosing appearance.
    pub fn flow(mut self, flow: ListBoxFlow, padding: (f32, f32)) -> Self {
        self.layout = flow.layout(padding);
        self.axis = match flow {
            ListBoxFlow::Horizontal { item_width, .. } => {
                self.item_width = item_width;
                ListBoxAxis::Horizontal
            }
            _ => ListBoxAxis::Vertical,
        };
        self.content_sized = matches!(flow, ListBoxFlow::VerticalContent { .. });
        self
    }

    /// Fixed-width cards in an available-width horizontal viewport.
    pub fn horizontal(mut self, item_width: f32) -> Self {
        self.axis = ListBoxAxis::Horizontal;
        self.item_width = item_width;
        self
    }

    /// Let vertical row templates determine their height at the available width.
    /// Use an explicit viewport height in `layout`. Templates must size naturally
    /// rather than fill an unspecified height. Horizontal flow remains fixed-size.
    pub fn content_sized(mut self) -> Self {
        self.content_sized = true;
        self
    }

    /// Opt into uniform windowing on either axis or measured vertical windowing.
    /// Measured requires `content_sized`; incompatible policies fall back to eager.
    /// Templates run only for viewport/buffer items and may still borrow local
    /// data. Durable item state belongs in host-owned models or entities.
    pub fn virtualization(mut self, policy: ListBoxVirtualization) -> Self {
        self.virtualization = policy;
        self
    }

    /// Accessible name of the list surface. Defaults to its ID.
    pub fn aria_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    /// Content centered over an empty viewport.
    pub fn empty(mut self, content: impl IntoElement) -> Self {
        self.empty = Some(content.into_any_element());
        self
    }

    /// Compatibility composition API for raw host-owned state and custom previews.
    /// Prefer `from_control` with SDK drag/drop configuration and named callbacks.
    /// This connects lists owned by the same host entity, which serves
    /// as their shared scope. The SDK owns session admission and lifecycle guards;
    /// `on_drop` validates/mutates the host data and completes the proposal.
    /// Preview construction happens only once a native drag actually starts.
    /// Unanswered synchronous proposals are rejected through `on_event`.
    pub fn drag_and_drop<S: Clone + 'static, P: Render + 'static>(
        mut self,
        list: S,
        on_drop: fn(&mut M, DropProposal<S, K>, &mut Window, &mut Context<M>),
        on_event: fn(&mut M, DragDropEvent<S, K>, &mut Context<M>),
        preview: impl Fn(&T, &[K], &mut Window, &mut App) -> Entity<P> + 'static,
    ) -> Self
    where
        T: Clone,
    {
        let state = self.state;
        let source = list.clone();
        let preview = std::rc::Rc::new(preview);
        let scroll = self.scroll;
        let target_events = on_event;
        // Read scope and theme at build time so fluent option order is immaterial.
        self.drag_drop = Some(DragDropBindings {
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
        });
        self
    }

    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(self, policy: crate::interaction::PointerFocusPolicy) -> Self {
        self.binding.set_pointer_focus_policy(policy);
        self
    }

    /// Configure wheel eligibility without changing boundary behavior.
    pub fn wheel_scroll_policy(self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.scroll.set_wheel_scroll_policy(policy);
        self
    }

    /// Override this retained list's interaction settings.
    pub fn scroll_boundary_policy(self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.scroll.set_scroll_boundary_policy(policy);
        self
    }

    /// Override this retained list's interaction settings.
    pub fn wheel_focus_scope(self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.scroll.set_wheel_focus_scope(policy);
        self
    }

    /// Assemble one frame using the existing SDK bindings and scroll handle.
    /// The supplied template builds only look-specific surfaces and chrome;
    /// mounted-row composition and all input/scroll/drop wiring stay here.
    pub fn build(mut self, template: &dyn ListBoxTemplate, window: &mut Window, cx: &mut Context<M>) -> Surface {
        self.scroll.set_wheel_focus_owner(self.binding.focus_handle().clone());
        let content_sized = self.content_sized && self.axis == ListBoxAxis::Vertical;
        let scope = cx.entity_id();
        let config = self.motion_config.take().unwrap_or(&mut self.raw_motion);
        if let Some(enabled) = self.motion_enabled {
            config.set_enabled(enabled);
        }
        let motion_enabled = config.enabled();
        let profile = config.inherit_with(|| template.motion_defaults(), self.motion_edits);
        self.scroll.set_motion(profile.scroll.enabled(motion_enabled));
        if let Some(drop) = &self.drop_layout {
            drop.configure_transition(&profile.drop_layout);
        }
        let drag_look = template.drag_look();
        let highlight = drag_look.marker_color;
        let preview_style = drag_look.preview;
        let insertion_target = |before: Option<K>, edge: DropEdge, cx: &mut Context<M>| {
            let zone = DropZone {
                axis: match self.axis {
                    ListBoxAxis::Vertical => Axis::Vertical,
                    ListBoxAxis::Horizontal => Axis::Horizontal,
                },
                edge,
                item_extent: px(match self.axis {
                    ListBoxAxis::Vertical => self.layout.item_height,
                    ListBoxAxis::Horizontal => self.item_width,
                }),
                following_gap: px(if before.is_some() { self.layout.spacing } else { 0.0 }),
                marker_width: px(drag_look.marker_width),
            };
            let target = div().id(if edge == DropEdge::Before {
                "insert-before"
            } else {
                "insert-after"
            });
            let mut surface = if content_sized {
                zone.apply_measured(target)
            } else {
                zone.apply(target)
            }
            .border_color(gpui::transparent_black());
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.highlight)(surface, scope, highlight);
                surface = (drag_drop.target)(surface, before, cx);
            }
            surface
        };
        let virtual_window = if content_sized {
            let (estimate, buffer) = match self.virtualization {
                ListBoxVirtualization::Measured { estimated_height, overscan } => (estimated_height, Some(overscan)),
                _ => (48.0, None),
            };
            Some(self.scroll.content_window(self.state, estimate, self.layout.spacing, buffer))
        } else {
            match self.virtualization {
                ListBoxVirtualization::Eager | ListBoxVirtualization::Measured { .. } => None,
                ListBoxVirtualization::Uniform { overscan } => self.scroll.virtual_window(
                    self.state,
                    self.axis,
                    match self.axis {
                        ListBoxAxis::Vertical => self.layout.item_height,
                        ListBoxAxis::Horizontal => self.item_width,
                    },
                    self.layout.spacing,
                    overscan,
                ),
            }
        };
        let range = virtual_window.as_ref().map_or(0..self.state.visible_items().len(), |window| window.range.clone());
        // `nth` on the projection's mapped slice iterator skips directly to the
        // range; off-screen items do not construct models, templates, or bindings.
        let items = self.state.visible_items().skip(range.start).take(range.len());
        let spacer = |extent: f32| {
            div()
                .flex_shrink_0()
                .when(self.axis == ListBoxAxis::Vertical, |space| space.h(px(extent)).w_full())
                .when(self.axis == ListBoxAxis::Horizontal, |space| space.w(px(extent)).h(px(self.layout.item_height)))
        };
        let mut rows = Vec::with_capacity(range.len() + 2);
        if let Some(extent) = virtual_window.as_ref().and_then(|window| window.leading_space) {
            rows.push(spacer(extent).into_any_element());
        }
        for item in items {
            let key = item.key.clone();
            let visible_index = item.visible_index;
            let surface = template.render_row(
                &ListBoxRowRenderModel {
                    id: (self.item_id)(&key),
                    state: item.state,
                    focus_visible: self.binding.focus_visible(window),
                    content_sized,
                },
                cx,
            );
            let mut surface = self.binding.bind_row(surface, key.clone(), item.state, cx, self.on_input);
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.row)(surface, item.item, &key, &preview_style, cx);
            }
            // Stable keyed wrappers survive window movement; spacers have no bindings.
            let mut row = div()
                .id((self.item_id)(&key))
                .debug_selector(|| format!("{}-item-{visible_index}", self.id))
                .relative()
                .when(self.axis == ListBoxAxis::Vertical, |row| row.w_full())
                .when(self.axis == ListBoxAxis::Horizontal, |row| row.w(px(self.item_width)))
                .min_w(px(0.0))
                .when(!content_sized, |row| row.h(px(self.layout.item_height)))
                .when(content_sized, |row| row.min_h(px(1.0)))
                .flex_shrink_0()
                .child((self.item)(surface, item, cx));
            if self.drag_drop.is_some() && cx.has_active_drag() {
                row = row.child(insertion_target(Some(key.clone()), DropEdge::Before, cx)).child(insertion_target(
                    self.state.visible_items().nth(visible_index + 1).map(|item| item.key),
                    DropEdge::After,
                    cx,
                ));
            }
            rows.push(if let Some(motion) = &self.drop_layout {
                motion.item(key.clone(), row)
            } else {
                row.into_any_element()
            });
        }
        if let Some(extent) = virtual_window.as_ref().and_then(|window| window.trailing_space) {
            rows.push(spacer(extent).into_any_element());
        }
        let mut viewport = div()
            .flex()
            .flex_col()
            .id(format!("{}-viewport", self.id))
            .debug_selector(|| format!("{}-viewport", self.id))
            .when(self.axis == ListBoxAxis::Vertical, |viewport| viewport.overflow_y_scroll())
            .when(self.axis == ListBoxAxis::Horizontal, |viewport| viewport.flex_row().overflow_x_scroll())
            .w_full()
            .min_w(px(0.0))
            .h(px(self.layout.viewport_height))
            .gap(px(self.layout.spacing))
            .children(rows);
        if let Some(drag_drop) = &self.drag_drop {
            viewport = (drag_drop.viewport)(viewport, scope, self.axis);
        }
        let mut surface = template
            .render(
                &ListBoxRenderModel {
                    id: self.id,
                    label: self.label,
                    layout: self.layout,
                    focused: self.state.is_focused(),
                },
                cx,
            )
            .aria_label(self.label);
        if let Some(drag_drop) = &self.drag_drop {
            if let Some(color) = drag_look.surface_color {
                surface = (drag_drop.highlight)(surface, scope, color);
            }
            surface = (drag_drop.target)(surface, None, cx).cancel_drag_on_escape();
        }
        if self.state.visible_items().len() == 0
            && let Some(empty) = self.empty
        {
            surface = surface.child(div().absolute().inset_0().flex().items_center().justify_center().child(empty));
        }
        let surface = match virtual_window {
            Some(rendered) => self.scroll.bind_virtualized(surface, viewport, self.state, rendered),
            None => self.scroll.bind(surface, viewport, self.state),
        };
        let surface =
            self.binding.bind_root(surface, self.axis, self.state.selection_mode(), window, cx, self.on_input);
        if let Some(motion) = &self.drop_layout {
            motion.set_motion_enabled(motion_enabled);
            return div().id(format!("{}-drop-motion", self.id)).child(self.scroll.drop_layout_scope(motion, surface));
        }
        surface
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "builder_tests.rs"]
mod tests;
