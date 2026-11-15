//! Shared listbox rendering pipeline. Appearance stays with the look.

use std::hash::Hash;
use gpui::{AnyElement, App, Axis, Context, Div, ElementId, Stateful, Window, div, prelude::*, px};
use crate::infra::drag_drop::{DragDropElementExt, DropEdge, DropZone};
use super::{
    ListBoxBinding, ListBoxDragDropRenderBindings, ListBoxInputHandler, ListBoxRenderModel, ListBoxRowRenderModel,
    ListBoxTemplate, ListBoxVisibleItem,
};
use super::{ListBoxAxis, ListBoxScrollHandle, ListBoxState, ListBoxVirtualization, ListBoxVirtualWindow};

/// Flow geometry used by both looks to select render windows and create spacers.
/// Visual chrome remains look-owned.
#[derive(Clone, Copy, Debug)]
pub struct ListBoxRenderGeometry {
    pub axis: ListBoxAxis,
    /// Fixed row extent on the scrolling axis.
    pub item_extent: f32,
    pub spacing: f32,
    /// Content sizing is supported only for vertical flow.
    pub content_sized: bool,
}

impl ListBoxRenderGeometry {
    /// Select a window using persistent scroll state. `None` means eager fixed
    /// rows. Eager content-sized rows still use measurement/reveal bookkeeping.
    pub fn window<T, K: Clone + Eq + Hash + 'static>(
        self,
        state: &ListBoxState<T, K>,
        scroll: &ListBoxScrollHandle<K>,
        policy: ListBoxVirtualization,
    ) -> Option<ListBoxVirtualWindow> {
        if self.content_sized && self.axis == ListBoxAxis::Vertical {
            let (estimate, buffer) = match policy {
                ListBoxVirtualization::Measured { estimated_height, overscan } => (estimated_height, Some(overscan)),
                _ => (48.0, None),
            };
            Some(scroll.content_window(state, estimate, self.spacing, buffer))
        } else {
            match policy {
                ListBoxVirtualization::Uniform { overscan } => {
                    scroll.virtual_window(state, self.axis, self.item_extent, self.spacing, overscan)
                }
                _ => None,
            }
        }
    }

    /// Create an inert spacer on the scrolling axis. It must be a direct
    /// viewport child so stack gaps match the window's geometry.
    pub fn spacer(self, extent: f32) -> Div {
        div()
            .flex_shrink_0()
            .when(self.axis == ListBoxAxis::Horizontal, |space| space.w(px(extent)))
            .when(self.axis == ListBoxAxis::Vertical, |space| space.h(px(extent)))
    }
}

/// Persistent input, scrolling, and optional drag/motion adapters. Rendering
/// borrows these objects; it never replaces state or registers host callbacks.
pub struct ListBoxRenderBindings<'a, M: 'static, T, K: 'static> {
    pub state: &'a ListBoxState<T, K>,
    pub binding: &'a mut ListBoxBinding,
    pub scroll: &'a ListBoxScrollHandle<K>,
    pub on_input: ListBoxInputHandler<M, K>,
    pub item_id: fn(&K) -> ElementId,
    pub virtualization: ListBoxVirtualization,
    pub drag_drop: Option<ListBoxDragDropRenderBindings<'a, M, T, K>>,
    pub drop_layout: Option<crate::motion::DropLayoutMotion<K>>,
    pub motion_enabled: bool,
}

/// Render mounted items only, using stable keyed rows and the retained SDK
/// adapters. `item` composes content around the bound row surface, preserving
/// raw host composition as well as the content-only template API. Keep the
/// supplied bound surface in the result; any outer wrapper must use the item's
/// stable `item_id`, rather than its visible index.
#[allow(clippy::too_many_arguments)]
pub fn render_listbox<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static>(
    model: &ListBoxRenderModel,
    parts: ListBoxRenderBindings<'_, M, T, K>,
    template: &dyn ListBoxTemplate,
    item: impl Fn(Stateful<Div>, ListBoxVisibleItem<'_, T, K>, &mut App) -> Stateful<Div>,
    empty: Option<AnyElement>,
    window: &mut Window,
    cx: &mut Context<M>,
) -> Stateful<Div> {
    parts.scroll.set_wheel_focus_owner(parts.binding.focus_handle().clone());
    let content_sized = model.content_sized && model.axis == ListBoxAxis::Vertical;
    let horizontal = model.axis == ListBoxAxis::Horizontal;
    let geometry = ListBoxRenderGeometry {
        axis: model.axis,
        item_extent: if horizontal {
            model.item_width
        } else {
            model.layout.item_height
        },
        spacing: model.layout.spacing,
        content_sized,
    };
    let virtual_window = geometry.window(parts.state, parts.scroll, parts.virtualization);
    let range = virtual_window.as_ref().map_or(0..parts.state.visible_items().len(), |window| window.range.clone());
    let scope = cx.entity_id();
    let preview = parts.drag_drop.as_ref().map(|_| template.drag_preview_style());
    let spacer = |extent| {
        geometry
            .spacer(extent)
            .when(!horizontal, |space| space.w_full())
            .when(horizontal, |space| space.h(px(model.layout.item_height)))
            .into_any_element()
    };
    let insertion_target = |before: Option<K>, edge: DropEdge, cx: &mut Context<M>| {
        let zone = DropZone {
            axis: if horizontal { Axis::Horizontal } else { Axis::Vertical },
            edge,
            item_extent: px(geometry.item_extent),
            following_gap: px(if before.is_some() { geometry.spacing } else { 0.0 }),
            marker_width: px(template.drop_marker_width()),
        };
        let target = div().id(if edge == DropEdge::Before {
            "insert-before"
        } else {
            "insert-after"
        });
        let mut target = if content_sized {
            zone.apply_measured(target)
        } else {
            zone.apply(target)
        }
        .border_color(gpui::transparent_black());
        if let Some(drag_drop) = &parts.drag_drop {
            target = (drag_drop.highlight)(target, scope, template.drop_color());
            target = (drag_drop.target)(target, before, cx);
        }
        target
    };
    let mut rows = Vec::with_capacity(range.len() + 2);
    if let Some(extent) = virtual_window.as_ref().and_then(|window| window.leading_space) {
        rows.push(spacer(extent));
    }
    // Skip the projection directly: off-screen items never construct row chrome,
    // content, input bindings, or drag handlers.
    for visible in parts.state.visible_items().skip(range.start).take(range.len()) {
        let key = visible.key.clone();
        let index = visible.visible_index;
        let row = template.render_row(
            &ListBoxRowRenderModel {
                id: (parts.item_id)(&key),
                state: visible.state,
                focus_visible: parts.binding.focus_visible(window),
            },
            cx,
        );
        let mut row = parts.binding.bind_row(row, key.clone(), visible.state, cx, parts.on_input);
        if let (Some(drag_drop), Some(preview)) = (&parts.drag_drop, &preview) {
            row = (drag_drop.row)(row, visible.item, &key, preview, cx);
        }
        let mut row = item(row, visible, cx)
            .debug_selector(|| format!("{}-item-{index}", model.id))
            .relative()
            .flex_shrink_0()
            .min_w_0()
            .when(horizontal, |row| row.w(px(model.item_width)))
            .when(!horizontal, |row| row.w_full())
            .when(!content_sized, |row| row.h(px(model.layout.item_height)))
            .when(content_sized, |row| row.min_h(px(1.0)));
        if parts.drag_drop.is_some() && cx.has_active_drag() {
            let next = parts.state.visible_items().nth(index + 1).map(|item| item.key);
            row = row.child(insertion_target(Some(key.clone()), DropEdge::Before, cx)).child(insertion_target(
                next,
                DropEdge::After,
                cx,
            ));
        }
        rows.push(match &parts.drop_layout {
            Some(motion) => motion.item(key, row),
            None => row.into_any_element(),
        });
    }
    if let Some(extent) = virtual_window.as_ref().and_then(|window| window.trailing_space) {
        rows.push(spacer(extent));
    }
    let mut viewport = div()
        .id(format!("{}-viewport", model.id))
        .debug_selector(|| format!("{}-viewport", model.id))
        .flex()
        .w_full()
        .min_w_0()
        .h(px(model.layout.viewport_height))
        .gap(px(model.layout.spacing))
        .when(horizontal, |viewport| viewport.overflow_x_scroll())
        .when(!horizontal, |viewport| viewport.flex_col().overflow_y_scroll())
        .children(rows);
    if let Some(drag_drop) = &parts.drag_drop {
        viewport = (drag_drop.viewport)(viewport, scope, model.axis);
    }
    let mut surface = template.render(model, cx).aria_label(model.label);
    if let Some(drag_drop) = &parts.drag_drop {
        if let Some(color) = template.surface_drop_color() {
            surface = (drag_drop.highlight)(surface, scope, color);
        }
        surface = (drag_drop.target)(surface, None, cx).cancel_drag_on_escape();
    }
    if parts.state.visible_items().len() == 0
        && let Some(empty) = empty
    {
        surface = surface.child(div().absolute().inset_0().flex().items_center().justify_center().child(empty));
    }
    let surface = match virtual_window {
        Some(rendered) => parts.scroll.bind_virtualized(surface, viewport, parts.state, rendered),
        None => parts.scroll.bind(surface, viewport, parts.state),
    };
    let surface =
        parts
            .binding
            .bind_root(surface, model.axis, parts.state.selection_mode(), window, cx, parts.on_input);
    if let Some(motion) = &parts.drop_layout {
        motion.set_motion_enabled(parts.motion_enabled);
        return div()
            .id(format!("{}-drop-motion", model.id))
            .w_full()
            .child(parts.scroll.drop_layout_scope(motion, surface));
    }
    surface
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::listbox::{ListBoxSnapshot, SelectionMode};

    #[test]
    fn content_measurement_is_vertical_and_fixed_rows_fall_back_from_measured() {
        let state = ListBoxState::from_snapshot(
            ListBoxSnapshot::try_new(0..10, |item| *item).unwrap(),
            SelectionMode::SingleAllowNone,
        );
        let scroll = ListBoxScrollHandle::default();
        let geometry = ListBoxRenderGeometry {
            axis: ListBoxAxis::Vertical,
            item_extent: 36.0,
            spacing: 4.0,
            content_sized: false,
        };
        let measured = ListBoxVirtualization::Measured { estimated_height: 48.0, overscan: 2 };
        assert!(geometry.window(&state, &scroll, measured).is_none());
        let content = ListBoxRenderGeometry { content_sized: true, ..geometry };
        let eager = content.window(&state, &scroll, ListBoxVirtualization::Eager).unwrap();
        assert_eq!(eager.range, 0..10);
        assert!(eager.measured_items.is_some());
        let horizontal = ListBoxRenderGeometry { axis: ListBoxAxis::Horizontal, ..content };
        assert!(horizontal.window(&state, &scroll, measured).is_none());
        let uniform = horizontal.window(&state, &scroll, ListBoxVirtualization::Uniform { overscan: 2 }).unwrap();
        assert!(uniform.measured_items.is_none());
        assert!(uniform.range.end <= 10);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "render_tests.rs"]
mod render_tests;
