use std::sync::Arc;

use gpui::{AnyElement, AppContext, Context, Entity, Pixels, ScrollEvent, SharedString, div, prelude::*, px};

use crate::controls::scroll_container::{ScrollContainer, ScrollbarAutoHideActivate, ScrollbarVisibility};
use crate::controls::scrollbar::{Scrollbar, ScrollbarTemplate};

#[derive(Clone)]
pub struct PopupScrollSurface {
    container: ScrollContainer,
    viewport_height: Pixels,
    row_height: Pixels,
    content_top_padding: Pixels,
    item_count: usize,
    scrolling_enabled: bool,
    snap_to_rows: bool,
}

impl PopupScrollSurface {
    pub fn new(
        id: impl Into<SharedString>,
        scrollbar_template: Arc<dyn ScrollbarTemplate>,
        cx: &mut impl AppContext,
    ) -> Self {
        Self {
            container: ScrollContainer::new(id, scrollbar_template, cx),
            viewport_height: px(crate::theme::stylesheet::SharedGeometry::sdk().popup_viewport_height),
            row_height: px(crate::theme::stylesheet::SharedGeometry::sdk().popup_row_height),
            content_top_padding: px(0.0),
            item_count: 0,
            scrolling_enabled: true,
            snap_to_rows: false,
        }
    }

    pub fn set_scrolling_enabled(&mut self, enabled: bool) {
        self.scrolling_enabled = enabled;
    }

    /// Choose when scrollbar chrome is visible; scrolling remains enabled independently.
    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.container = self.container.visibility(visibility);
        self
    }

    /// Choose hover, timed scroll activity, or both for auto-hiding chrome.
    pub fn scrollbar_auto_hide_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.container = self.container.auto_hide_activate(activate);
        self
    }

    pub fn set_snap_to_rows(&mut self, enabled: bool) {
        self.snap_to_rows = enabled;
    }

    /// Current viewport offset, independent of delayed scrollbar geometry updates.
    pub fn vertical_offset(&self) -> Pixels {
        self.container.vertical_offset()
    }

    pub fn scrollbar(&self) -> Entity<Scrollbar> {
        self.container.scrollbar()
    }

    pub fn set_vertical_offset<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        if !self.scrolling_enabled {
            return;
        }
        self.container.set_vertical_offset(self.quantize_offset(value), cx);
    }

    pub fn configure(
        &mut self,
        item_count: usize,
        row_height: Pixels,
        content_top_padding: Pixels,
        viewport_height: Pixels,
    ) {
        self.item_count = item_count;
        self.row_height = row_height.max(px(1.0));
        self.content_top_padding = content_top_padding.max(px(0.0));
        self.viewport_height = viewport_height.max(px(1.0));
    }

    pub fn ensure_item_visible<T: 'static>(&self, item_index: usize, cx: &mut Context<T>) {
        if !self.scrolling_enabled || self.item_count == 0 {
            return;
        }

        let index = item_index.min(self.item_count.saturating_sub(1));
        let top = self.content_top_padding + (self.row_height * index as f32);
        let bottom = top + self.row_height;
        self.container.ensure_vertical_bounds_visible(top, bottom, cx);
    }

    pub fn sync<T: 'static>(&self, cx: &mut Context<T>) {
        if !self.scrolling_enabled {
            return;
        }
        self.container.sync_scrollbar(cx);
    }

    pub fn visible_row_count(&self) -> usize {
        let content_height = (self.viewport_height - (self.content_top_padding * 2.0)).max(px(1.0));
        ((content_height.as_f32() / self.row_height.as_f32()).floor() as usize).max(1)
    }

    pub fn scroll_wheel<T: 'static>(&mut self, event: &ScrollEvent, cx: &mut Context<T>) -> bool {
        if !self.scrolling_enabled {
            return false;
        }

        let delta = event.delta.pixel_delta(self.row_height).y.as_f32();
        if !delta.is_finite() || delta.abs() <= f32::EPSILON {
            return false;
        }

        self.container.reveal_on_scroll(cx);

        let current = self.container.vertical_offset().as_f32();
        let max = self.container.max_vertical_offset().as_f32();
        let target = (current - delta).clamp(0.0, max);
        // Keep exact boundaries reachable and do not quantize trackpad/fractional input.
        let target = if event.delta.precise() || target == 0.0 || target == max {
            target
        } else {
            self.quantize_offset(target).clamp(0.0, max)
        };
        if (target - current).abs() <= f32::EPSILON {
            return false;
        }

        self.container.set_vertical_offset(target, cx);
        true
    }

    /// Pointer-scrolling composition without an owner wheel callback.
    pub fn render(&self, content: AnyElement) -> AnyElement {
        if !self.scrolling_enabled {
            return div().w_full().child(content).into_any_element();
        }
        div().h(self.viewport_height).w_full().child(self.container.render(content)).into_any_element()
    }

    /// Clip and position content while the composite handles wheel input once.
    pub fn render_without_wheel(&self, content: AnyElement) -> AnyElement {
        if !self.scrolling_enabled {
            return div().w_full().child(content).into_any_element();
        }
        if !self.scrolling_enabled {
            return div().w_full().child(content).into_any_element();
        }

        div()
            .h(self.viewport_height)
            .w_full()
            .child(self.container.render_without_wheel(content))
            .into_any_element()
    }

    /// Bind the owner's callback inside the popup hitbox, including anchored popups.
    /// The outer composite must not install the same callback a second time.
    pub fn render_with_scroll_wheel(
        &self,
        content: AnyElement,
        handler: impl Fn(&ScrollEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> AnyElement {
        div().w_full().on_scroll(handler).child(self.render_without_wheel(content)).into_any_element()
    }

    fn quantize_offset(&self, value: f32) -> f32 {
        if !self.snap_to_rows || self.row_height.as_f32() <= f32::EPSILON {
            return value;
        }

        let row_height = self.row_height.as_f32();
        (value / row_height).round() * row_height
    }
}
