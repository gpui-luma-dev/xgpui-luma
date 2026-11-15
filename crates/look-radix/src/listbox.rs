//! Radix listbox composition over persistent SDK state, input, and scrolling.

use std::hash::Hash;
use gpui::{App, Context, SharedString, Div, ElementId, Stateful, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxFlow, ListBoxItemRenderModel, ListBoxItemState, ListBoxItemTemplate, ListBoxAxis,
    ListBoxDragDropRenderBindings, ListBoxRenderBindings, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxTemplate,
    render_listbox, make_listbox_item_template,
};
use crate::{Look, ScaleFamily, SemanticRole};
use crate::look::resolve_look;

/// Radix row surface. The SDK binding attaches selection and keyboard input.
pub fn listbox_row(
    look: &Look,
    id: impl Into<ElementId>,
    state: ListBoxItemState,
    focus_visible: bool,
) -> Stateful<Div> {
    let step = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
    div()
        .id(id)
        .rounded(px(4.0))
        .border_1()
        .border_color(if state.active && focus_visible {
            look.resolve_role(SemanticRole::Focus).hsla()
        } else {
            gpui::transparent_black()
        })
        .bg(if state.selected {
            step(4)
        } else {
            gpui::transparent_black()
        })
        .text_color(
            look.resolve_role(if state.enabled {
                SemanticRole::Foreground
            } else {
                SemanticRole::MutedForeground
            })
            .hsla(),
        )
        .font_family(crate::typography::font_family(look))
        .text_sm()
        .px_2()
        .flex()
        .items_center()
        .when(state.enabled, |row| {
            row.cursor_pointer().hover(|style| style.bg(step(5))).active(|style| style.bg(step(6)))
        })
}

/// Default label content for a Radix listbox. The builder supplies the themed
/// selection surface; callers can also provide any SDK item-content template.
pub fn listbox_template<T: 'static>(label: impl Fn(&T) -> SharedString + 'static) -> ListBoxItemTemplate<'static, T> {
    make_listbox_item_template(move |model: &ListBoxItemRenderModel<'_, T>, _: &mut App| div().child(label(model.item)))
}

/// Borrowed list builder. Keep the SDK control on the host between renders;
/// building does not reset focus, selection, or scroll position.
pub struct ListBoxBuilder<'a, M: 'static, T, K: 'static> {
    motion_edits: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::listbox::ListBoxMotion>,
    motion_enabled: Option<bool>,
    id: &'static str,
    control: &'a mut ListBoxControl<M, T, K>,
    look: Option<Look>,
    flow: ListBoxFlow,
    template: ListBoxItemTemplate<'a, T>,
}
impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    xgpui_luma::forward_motion!(xgpui_luma::controls::listbox::ListBoxMotion, edits motion_edits);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = Some(enabled);
        self
    }

    pub fn new(
        id: &'static str,
        control: &'a mut ListBoxControl<M, T, K>,
        template: ListBoxItemTemplate<'a, T>,
    ) -> Self {
        Self {
            motion_edits: Default::default(),
            motion_enabled: None,
            id,
            control,
            look: None,
            flow: ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 },
            template,
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn flow(mut self, flow: ListBoxFlow) -> Self {
        self.flow = flow;
        self
    }
    /// Render with SDK windowing policy retained by the control.
    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Stateful<Div> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        let parts = self.control.render_parts();
        if let Some(enabled) = self.motion_enabled {
            parts.motion_config.set_enabled(enabled);
        }
        let motion_enabled = parts.motion_config.enabled();
        let profile = parts.motion_config.inherit_with(|| look.motion_defaults().listbox, self.motion_edits);
        parts.scroll.set_motion(profile.scroll.enabled(motion_enabled));
        parts.drop_layout.configure_transition(&profile.drop_layout);
        let layout = self.flow.layout((0.0, 0.0));
        let (axis, extent) = match self.flow {
            ListBoxFlow::Horizontal { item_width, .. } => (ListBoxAxis::Horizontal, item_width),
            _ => (ListBoxAxis::Vertical, layout.item_height),
        };
        let model = ListBoxRenderModel {
            id: self.id,
            label: self.id,
            layout,
            axis,
            item_width: extent,
            content_sized: matches!(self.flow, ListBoxFlow::VerticalContent { .. }),
            focused: parts.state.is_focused(),
        };
        let label = parts.item_label;
        let drag_drop = parts
            .drag_drop
            .map(|binding| ListBoxDragDropRenderBindings::from_control(binding, parts.state, parts.scroll, label));
        render_listbox(
            &model,
            ListBoxRenderBindings {
                state: parts.state,
                binding: parts.binding,
                scroll: parts.scroll,
                on_input: parts.on_input,
                item_id: parts.item_id,
                virtualization: parts.virtualization,
                drag_drop,
                drop_layout: Some(parts.drop_layout),
                motion_enabled,
            },
            &RadixListBoxTemplate { look },
            |surface, item, cx| {
                let model = ListBoxItemRenderModel {
                    item: item.item,
                    selected: item.state.selected,
                    active: item.state.active,
                    enabled: item.state.enabled,
                };
                surface.aria_label(label(item.item)).child((self.template)(&model, cx))
            },
            None,
            window,
            cx,
        )
    }
}

struct RadixListBoxTemplate {
    look: Look,
}

impl ListBoxTemplate for RadixListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, _: &mut App) -> Stateful<Div> {
        crate::tree_view_frame(&self.look).rounded(px(8.0)).p_1().id(model.id).w_full().min_w_0()
    }

    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Stateful<Div> {
        listbox_row(&self.look, model.id.clone(), model.state, model.focus_visible)
    }

    fn drop_color(&self) -> gpui::Hsla {
        self.look.resolve_role(SemanticRole::Focus).hsla()
    }

    fn surface_drop_color(&self) -> Option<gpui::Hsla> {
        None
    }

    fn drop_marker_width(&self) -> f32 {
        2.0
    }

    fn drag_preview_style(&self) -> xgpui_luma::controls::tooltip::TooltipLook {
        crate::tooltip_theme(&self.look).resolve()
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
        super::ListBoxBuilder::new(
            if side == 0 { "sdk-left" } else { "sdk-right" },
            list,
            super::listbox_template(|row: &u32| row.to_string().into()),
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
            let look = crate::Look::built_in();
            look.set_common_stylesheet(xgpui_luma::theme::stylesheet::CommonStylesheet::parse(&source).unwrap());
            look
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
    include!("../../sdk/tests/support/listbox_drag.rs");
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
        super::ListBoxBuilder::new("contract-list", list, template).flow(flow).build(window, cx)
    }
    include!("../../sdk/tests/support/listbox_render.rs");
}
