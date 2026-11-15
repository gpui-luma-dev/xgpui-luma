//! Shared geometry stage of listbox rendering. Appearance stays with the look.

use std::hash::Hash;
use gpui::{Div, div, prelude::*, px};
use super::{ListBoxAxis, ListBoxScrollHandle, ListBoxState, ListBoxVirtualization, ListBoxVirtualWindow};

/// Flow geometry used by both looks to select render windows and create spacers.
/// Cross-axis spacer sizing and visual chrome remain look-owned.
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
