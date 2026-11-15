//! Shadcn composition of SDK ListBox state, bindings, and optional DnD.
//!
//! Keep the state, binding, and scroll handle for the lifetime of the host. Build
//! the SDK Frame each render for outer decoration and inset. Focus, selection,
//! and scrolling stay in the ListBox bindings; rebuilding must not reset them.
//! Item content and drop commits remain application choices. This is deliberately
//! a borrowed `.build(window, cx)` API, not a second entity owning the collection.

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_builder_tests.rs"]
mod tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_virtualization_tests.rs"]
mod virtualization_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_measured_tests.rs"]
mod measured_tests;

use std::hash::Hash;
#[cfg(all(test, feature = "test-support"))]
use gpui::div;
use gpui::{App, Context, Div, ElementId, Entity, Render, Stateful, Window, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxBuilder as SdkListBoxBuilder, ListBoxBinding, ListBoxControl, ListBoxFlow, ListBoxInput,
    ListBoxItemRenderModel, ListBoxLayout, ListBoxScrollHandle, ListBoxState, ListBoxVisibleItem,
    ListBoxVirtualization,
};
use xgpui_luma::infra::drag_drop::{DragDropEvent, DropProposal};
use crate::look::{ShadcnLook, resolve_look_from};

type Surface = Stateful<Div>;
type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);
const CORNER_RADIUS: f32 = 8.0;

fn default_layout() -> ListBoxLayout {
    ListBoxLayout { item_height: 36.0, spacing: 4.0, viewport_height: 196.0, inset_x: 8.0, inset_y: 8.0 }
}

/// Shadcn appearance over the SDK's borrowed listbox builder. Keep state and
/// handles on the host between renders; building never resets them.
pub struct ListBoxBuilder<'a, M: 'static, T, K> {
    look: Option<ShadcnLook>,
    builder: SdkListBoxBuilder<'a, M, T, K>,
}
impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    xgpui_luma::forward_motion!(xgpui_luma::controls::listbox::ListBoxMotion, builder builder);

    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    /// Reuse the host's existing SDK state and handles. Stable item IDs must be
    /// unique within this list and remain the same when items move or reorder.
    pub fn new<E: IntoElement>(
        id: &'static str,
        state: &'a ListBoxState<T, K>,
        binding: &'a mut ListBoxBinding,
        scroll: &'a ListBoxScrollHandle<K>,
        on_input: InputHandler<M, K>,
        item_id: fn(&K) -> ElementId,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self {
            look: None,
            builder: SdkListBoxBuilder::new(id, state, binding, scroll, on_input, item_id, default_layout(), item),
        }
    }

    /// Compose a persistent SDK control, including its registered drag/drop
    /// callbacks and configuration. No render-time callback registration occurs.
    pub fn from_control<E: IntoElement>(
        id: &'static str,
        control: &'a mut ListBoxControl<M, T, K>,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self { look: None, builder: SdkListBoxBuilder::from_control(id, control, default_layout(), item) }
    }

    /// Bind an explicit look, including live theme changes.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Reuse SDK-owned drop motion for raw/host-owned composition.
    pub fn drop_layout_motion(mut self, motion: xgpui_luma::motion::DropLayoutMotion<K>) -> Self {
        self.builder = self.builder.drop_layout_motion(motion);
        self
    }
    /// Set SDK geometry; appearance remains owned by this look.
    pub fn layout(mut self, layout: ListBoxLayout) -> Self {
        self.builder = self.builder.layout(layout);
        self
    }
    /// Fixed-width cards in an available-width horizontal viewport.
    pub fn horizontal(mut self, item_width: f32) -> Self {
        self.builder = self.builder.horizontal(item_width);
        self
    }
    /// Let vertical row templates determine their height at the available width.
    /// Use an explicit viewport height in `layout`. Templates must size naturally
    /// rather than fill an unspecified height. Horizontal flow remains fixed-size.
    pub fn content_sized(mut self) -> Self {
        self.builder = self.builder.content_sized();
        self
    }
    /// Opt into uniform windowing on either axis or measured vertical windowing.
    /// Measured requires `content_sized`; incompatible policies fall back to eager.
    /// Templates run only for viewport/buffer items and may still borrow local
    /// data. Durable item state belongs in host-owned models or entities.
    pub fn virtualization(mut self, policy: ListBoxVirtualization) -> Self {
        self.builder = self.builder.virtualization(policy);
        self
    }
    /// Accessible name of the list surface. Defaults to its ID.
    pub fn aria_label(mut self, label: &'static str) -> Self {
        self.builder = self.builder.aria_label(label);
        self
    }
    /// Content centered over an empty viewport.
    pub fn empty(mut self, content: impl IntoElement) -> Self {
        self.builder = self.builder.empty(content);
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
        self.builder = self.builder.drag_and_drop(list, on_drop, on_event, preview);
        self
    }

    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(mut self, policy: xgpui_luma::interaction::PointerFocusPolicy) -> Self {
        self.builder = self.builder.pointer_focus_policy(policy);
        self
    }
    /// Configure wheel eligibility without changing boundary behavior.
    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.builder = self.builder.wheel_scroll_policy(policy);
        self
    }
    /// Override this retained list's interaction settings.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.builder = self.builder.scroll_boundary_policy(policy);
        self
    }
    /// Override this retained list's interaction settings.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.builder = self.builder.wheel_focus_scope(policy);
        self
    }

    /// Assemble one frame using the existing SDK bindings and scroll handle.
    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Surface {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.builder.build(look.listbox_template().as_ref(), window, cx)
    }
}

impl ShadcnLook {
    /// Render a persistent SDK control with this look and an arbitrary content
    /// template. The callback may borrow render-local data; it is not retained.
    /// Keep the control across renders so focus and scroll position remain stable.
    #[allow(clippy::too_many_arguments)]
    pub fn render_listbox<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static, E: IntoElement>(
        &self,
        control: &mut ListBoxControl<M, T, K>,
        id: &'static str,
        flow: ListBoxFlow,
        padding: (f32, f32),
        template: impl Fn(&ListBoxItemRenderModel<'_, T>, &mut App) -> E,
        window: &mut Window,
        cx: &mut Context<M>,
    ) -> Stateful<Div> {
        let parts = control.render_parts();
        let layout = flow.layout(padding);
        let content_sized = matches!(flow, ListBoxFlow::VerticalContent { .. });
        let item_label = parts.item_label;
        let mode = parts.state.selection_mode();
        let builder = SdkListBoxBuilder::from_parts(id, parts, layout, |surface, item, cx| {
            let model = ListBoxItemRenderModel {
                item: item.item,
                selected: item.state.selected,
                active: item.state.active,
                enabled: item.state.enabled,
            };
            surface
                .w_full()
                .when(!content_sized, |surface| surface.h_full())
                .rounded(px(CORNER_RADIUS))
                .overflow_hidden()
                .aria_label(item_label(item.item))
                .when(!item.state.enabled, |surface| surface.aria_description("Disabled"))
                .child(template(&model, cx))
        })
        .flow(flow, padding);
        builder
            .build(self.listbox_template().as_ref(), window, cx)
            .aria_description(format!("Selection mode: {mode:?}"))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod persistent_drag_tests {
    fn render_drag_list(
        side: usize,
        list: &mut xgpui_luma::controls::listbox::ListBoxControl<DragHarness, u32, u32>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<DragHarness>,
    ) -> gpui::Stateful<gpui::Div> {
        super::ListBoxBuilder::from_control(
            if side == 0 { "sdk-left" } else { "sdk-right" },
            list,
            |surface, item, _| surface.child(item.item.to_string()),
        )
        .build(window, cx)
    }

    #[test]
    fn retained_list_captures_typed_look_roles_once() {
        use xgpui_luma::motion::{Transition, Easing};
        use std::time::Duration;
        let make_look = |ms| {
            let source =
                format!("[common.motion.listbox.drop_layout]\nkind = 'timed'\nduration_ms = {ms}\neasing = 'linear'");
            let sheet = crate::stylesheet::StylesheetConfig::parse(&source).unwrap();
            crate::ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, sheet).unwrap()
        };
        let expected = Transition::timed(Duration::from_millis(350), Easing::Linear).description();
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            cx.set_global(make_look(350));
            DragHarness::new(cx)
        });
        cx.run_until_parked();
        view.update(cx, |host, cx| {
            assert_eq!(host.lists[0].motion_profile().drop_layout.description(), expected);
            cx.set_global(make_look(800));
            host.lists[0].set_motion_enabled(false);
            cx.notify();
        });
        cx.run_until_parked();
        view.update(cx, |host, cx| {
            assert_eq!(host.lists[0].motion_profile().drop_layout.description(), expected);
            host.lists[0].set_motion_enabled(true);
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|_, app| assert_eq!(view.read(app).lists[0].motion_profile().drop_layout.description(), expected));
    }
    include!("../../../sdk/tests/support/listbox_drag.rs");
}
