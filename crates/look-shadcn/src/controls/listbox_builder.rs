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

use gpui::{AnyElement, App, Context, Div, ElementId, Entity, Render, Stateful, Window, div, prelude::*, px};
use xgpui_luma::controls::frame::FrameBuilder;
use xgpui_luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxControl, ListBoxFlow, ListBoxInput, ListBoxItemRenderModel, ListBoxLayout,
    ListBoxDragDropRenderBindings, ListBoxRenderBindings, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxTemplate,
    ListBoxScrollHandle, ListBoxState, ListBoxVisibleItem, ListBoxVirtualization, render_listbox,
};
use xgpui_luma::infra::drag_drop::{DragDropEvent, DropProposal};
use crate::look::{ShadcnLook, resolve_look_from};

const CORNER_RADIUS: f32 = 8.0;

use xgpui_luma::controls::tooltip::TooltipLook;

type Surface = Stateful<Div>;
type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);
type ItemRenderer<'a, T, K> = Box<dyn Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> AnyElement + 'a>;
/// Look-owned fixed-size or content-sized list composition. Reuse SDK state and handles between renders.
/// No DnD handlers or gap overlays are installed unless `drag_and_drop` is used.
pub struct ListBoxBuilder<'a, M: 'static, T, K> {
    motion_edits: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::listbox::ListBoxMotion>,
    motion_enabled: Option<bool>,
    raw_motion: xgpui_luma::motion::MotionConfig<xgpui_luma::controls::listbox::ListBoxMotion>,
    id: &'static str,
    look: Option<ShadcnLook>,
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
    drag_drop: Option<ListBoxDragDropRenderBindings<'a, M, T, K>>,
    drop_layout: Option<xgpui_luma::motion::DropLayoutMotion<K>>,
    motion_config: Option<&'a mut xgpui_luma::motion::MotionConfig<xgpui_luma::controls::listbox::ListBoxMotion>>,
}

impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    xgpui_luma::forward_motion!(xgpui_luma::controls::listbox::ListBoxMotion, edits motion_edits);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = Some(enabled);
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
            item_width: 136.0,
            content_sized: false,
            virtualization: ListBoxVirtualization::default(),
            look: None,
            layout: ListBoxLayout {
                item_height: 36.0,
                spacing: 4.0,
                viewport_height: 196.0,
                inset_x: 8.0,
                inset_y: 8.0,
            },
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
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        let parts = control.render_parts();
        Self::new(id, parts.state, parts.binding, parts.scroll, parts.on_input, parts.item_id, item)
            .virtualization(parts.virtualization)
            .motion_config(parts.motion_config)
            .drop_layout_motion(parts.drop_layout)
            .control_drag_drop(parts.drag_drop, parts.item_label)
    }

    fn motion_config(
        mut self,
        config: &'a mut xgpui_luma::motion::MotionConfig<xgpui_luma::controls::listbox::ListBoxMotion>,
    ) -> Self {
        self.motion_config = Some(config);
        self
    }

    /// Reuse SDK-owned drop motion for raw/host-owned composition.
    pub fn drop_layout_motion(mut self, motion: xgpui_luma::motion::DropLayoutMotion<K>) -> Self {
        self.drop_layout = Some(motion);
        self
    }

    fn control_drag_drop(
        mut self,
        binding: Option<xgpui_luma::controls::listbox::ListBoxDragDropBinding<M, K>>,
        label: fn(&T) -> gpui::SharedString,
    ) -> Self {
        self.drag_drop =
            binding.map(|binding| ListBoxDragDropRenderBindings::from_control(binding, self.state, self.scroll, label));
        self
    }

    /// Bind an explicit look, including live theme changes.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set SDK geometry; appearance remains owned by this look.
    pub fn layout(mut self, layout: ListBoxLayout) -> Self {
        self.layout = layout;
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
        self.drag_drop =
            Some(ListBoxDragDropRenderBindings::custom(self.state, self.scroll, list, on_drop, on_event, preview));
        self
    }

    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(self, policy: xgpui_luma::interaction::PointerFocusPolicy) -> Self {
        self.binding.set_pointer_focus_policy(policy);
        self
    }

    /// Configure wheel eligibility without changing boundary behavior.
    pub fn wheel_scroll_policy(self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.scroll.set_wheel_scroll_policy(policy);
        self
    }

    /// Override this retained list's interaction settings.
    pub fn scroll_boundary_policy(self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.scroll.set_scroll_boundary_policy(policy);
        self
    }

    /// Override this retained list's interaction settings.
    pub fn wheel_focus_scope(self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.scroll.set_wheel_focus_scope(policy);
        self
    }

    /// Assemble one frame using the existing SDK bindings and scroll handle.
    pub fn build(mut self, window: &mut Window, cx: &mut Context<M>) -> Surface {
        let look = resolve_look_from(self.look.as_ref(), cx);
        let config = self.motion_config.take().unwrap_or(&mut self.raw_motion);
        if let Some(enabled) = self.motion_enabled {
            config.set_enabled(enabled);
        }
        let motion_enabled = config.enabled();
        let profile = config.inherit_with(|| look.motion_defaults().listbox, self.motion_edits);
        self.scroll.set_motion(profile.scroll.enabled(motion_enabled));
        if let Some(drop) = &self.drop_layout {
            drop.configure_transition(&profile.drop_layout);
        }
        let model = ListBoxRenderModel {
            id: self.id,
            label: self.label,
            layout: self.layout,
            axis: self.axis,
            item_width: self.item_width,
            content_sized: self.content_sized,
            focused: self.state.is_focused(),
        };
        let item_id = self.item_id;
        render_listbox(
            &model,
            ListBoxRenderBindings {
                state: self.state,
                binding: self.binding,
                scroll: self.scroll,
                on_input: self.on_input,
                item_id,
                virtualization: self.virtualization,
                drag_drop: self.drag_drop,
                drop_layout: self.drop_layout,
                motion_enabled,
            },
            &ShadcnListBoxTemplate { look },
            |surface, item, cx| div().id(item_id(&item.key)).child((self.item)(surface, item, cx)),
            self.empty,
            window,
            cx,
        )
    }
}

struct ShadcnListBoxTemplate {
    look: ShadcnLook,
}

impl ListBoxTemplate for ShadcnListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, cx: &mut App) -> Surface {
        let mode = self.look.mode_tokens();
        let palette = super::listbox::listbox_surface_palette(&mode, true);
        let border = if model.focused {
            crate::focus::focus_ring_or_fallback(&mode.catalog)
        } else {
            palette.border
        };
        FrameBuilder::new(format!("{}-surface", model.id))
            // The viewport plus inset determines height, not Frame's full-size default.
            .h_auto()
            // Keep flex parents from shrinking the surface below its fixed viewport.
            .min_h(gpui::Length::Auto)
            .bg(palette.background)
            .border_1()
            .border_color(border)
            .rounded(px(CORNER_RADIUS))
            .px(px(model.layout.inset_x))
            .py(px(model.layout.inset_y))
            .render(cx)
            .debug_selector(|| format!("{}-surface", model.id))
            .relative()
            .aria_label(model.label)
    }

    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Surface {
        self.look.listbox_row(model.id.clone(), model.state, model.focus_visible)
    }

    fn drop_color(&self) -> gpui::Hsla {
        self.look.token_color("ring").unwrap_or(self.look.chrome().border)
    }

    fn surface_drop_color(&self) -> Option<gpui::Hsla> {
        Some(self.drop_color())
    }

    fn drop_marker_width(&self) -> f32 {
        2.0
    }

    fn drag_preview_style(&self) -> TooltipLook {
        crate::tooltip_theme(&self.look).resolve()
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
        let builder = ListBoxBuilder::new(
            id,
            parts.state,
            parts.binding,
            parts.scroll,
            parts.on_input,
            parts.item_id,
            |surface, item, cx| {
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
            },
        )
        .control_drag_drop(parts.drag_drop, parts.item_label)
        .motion_config(parts.motion_config)
        .drop_layout_motion(parts.drop_layout)
        .look(self)
        .layout(layout)
        .virtualization(parts.virtualization);
        let builder = match flow {
            ListBoxFlow::Vertical { .. } => builder,
            ListBoxFlow::VerticalContent { .. } => builder.content_sized(),
            ListBoxFlow::Horizontal { item_width, .. } => builder.horizontal(item_width),
        };
        builder.build(window, cx).aria_description(format!("Selection mode: {mode:?}"))
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

#[cfg(all(test, feature = "test-support"))]
mod shared_render_tests {
    fn render_contract_list(
        list: &mut ListBoxControl<RenderHarness, u32, u32>,
        flow: ListBoxFlow,
        template: ListBoxItemTemplate<'_, u32>,
        window: &mut Window,
        cx: &mut Context<RenderHarness>,
    ) -> Stateful<Div> {
        crate::ShadcnLook::built_in().render_listbox(
            list,
            "contract-list",
            flow,
            (8.0, 8.0),
            move |model, cx| template(model, cx),
            window,
            cx,
        )
    }
    include!("../../../sdk/tests/support/listbox_render.rs");
}
