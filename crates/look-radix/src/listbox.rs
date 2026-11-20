//! Radix listbox composition over persistent SDK state, input, and scrolling.

use std::{hash::Hash, sync::Arc};
use gpui::{App, Context, SharedString, Div, ElementId, Stateful, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxBuilder as SdkListBoxBuilder, ListBoxControl, ListBoxFlow, ListBoxTemplate, ListBoxRenderModel,
    ListBoxRowRenderModel, ListBoxDragLook, ListBoxMotion, ListBoxItemRenderModel, ListBoxItemState,
    ListBoxItemTemplate, StyledListBox, make_listbox_item_template,
};
use crate::{Look, SemanticRole};
use crate::look::resolve_look;

/// The listbox chrome from `.lss` (`style/listbox.lss`).
pub fn styled_listbox(look: &Look) -> StyledListBox {
    StyledListBox::new(xgpui_luma::style::StyleBinding::new("listbox", crate::textfield::live_source(look)))
}

/// Radix row surface. The SDK binding attaches selection and keyboard input.
pub fn listbox_row(
    look: &Look,
    id: impl Into<ElementId>,
    state: ListBoxItemState,
    focus_visible: bool,
) -> Stateful<Div> {
    styled_listbox(look).paint_row(id, state, focus_visible).flex().items_center()
}

/// Default label content for a Radix listbox. The builder supplies the themed
/// selection surface; callers can also provide any SDK item-content template.
pub fn listbox_template<T: 'static>(label: impl Fn(&T) -> SharedString + 'static) -> ListBoxItemTemplate<'static, T> {
    make_listbox_item_template(move |model: &ListBoxItemRenderModel<'_, T>, _: &mut App| div().child(label(model.item)))
}

/// Borrowed list builder with Radix chrome over the shared SDK pipeline.
pub struct ListBoxBuilder<'a, M: 'static, T, K: 'static> {
    builder: SdkListBoxBuilder<'a, M, T, K>,
    look: Option<Look>,
}
impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    xgpui_luma::forward_motion!(ListBoxMotion, builder builder);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }
    pub fn new(
        id: &'static str,
        control: &'a mut ListBoxControl<M, T, K>,
        template: ListBoxItemTemplate<'a, T>,
    ) -> Self {
        let parts = control.render_parts();
        let label = parts.item_label;
        let flow = ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 };
        Self {
            look: None,
            builder: SdkListBoxBuilder::from_parts(id, parts, flow.layout((0.0, 0.0)), move |surface, item, cx| {
                let model = ListBoxItemRenderModel {
                    item: item.item,
                    selected: item.state.selected,
                    active: item.state.active,
                    enabled: item.state.enabled,
                };
                surface.aria_label(label(item.item)).child(template(&model, cx))
            }),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn flow(mut self, flow: ListBoxFlow) -> Self {
        self.builder = self.builder.flow(flow, (0.0, 0.0));
        self
    }
    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Stateful<Div> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        self.builder.build(listbox_surface_template(&look).as_ref(), window, cx).w_full()
    }
}

struct RadixListBoxTemplate {
    look: Look,
}
impl ListBoxTemplate for RadixListBoxTemplate {
    fn render(&self, model: &ListBoxRenderModel, _: &mut App) -> Stateful<Div> {
        let surface = styled_listbox(&self.look).surface(true, model.focused);
        div()
            .bg(surface.background)
            .border(px(surface.border_width))
            .border_color(surface.border)
            .rounded(px(surface.radius))
            .overflow_hidden()
            .p_1()
            .id(model.id)
            .w_full()
            .min_w_0()
    }
    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Stateful<Div> {
        listbox_row(&self.look, model.id.clone(), model.state, model.focus_visible)
            .w_full()
            .when(!model.content_sized, |row| row.h_full())
    }
    fn drag_look(&self) -> ListBoxDragLook {
        ListBoxDragLook {
            marker_color: self.look.resolve_role(SemanticRole::Focus).hsla(),
            marker_width: 2.0,
            surface_color: None,
            preview: crate::tooltip_theme(&self.look).resolve(),
        }
    }
    fn motion_defaults(&self) -> ListBoxMotion {
        self.look.motion_defaults().listbox
    }
}

/// Radix surface and row chrome for the SDK's shared listbox builder.
/// This is separate from `listbox_template`, which supplies item content.
pub fn listbox_surface_template(look: &Look) -> Arc<dyn ListBoxTemplate> {
    Arc::new(RadixListBoxTemplate { look: look.clone() })
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
#[path = "listbox_virtualization_tests.rs"]
mod virtualization_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_measured_tests.rs"]
mod measured_tests;

#[cfg(test)]
mod lss_tests {
    use xgpui_luma::controls::listbox::ListBoxItemState;

    use crate::control_group::assert_close;
    use crate::{Look, ScaleFamily, SemanticRole};

    #[test]
    fn lss_rows_match_the_native_rows() {
        let look = Look::built_in();
        let styled = super::styled_listbox(&look);
        let step = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
        let role = |r| look.resolve_role(r).hsla();
        for selected in [false, true] {
            for enabled in [true, false] {
                let state = ListBoxItemState { selected, active: true, enabled };
                let row = styled.row(state, false, false, false);
                let fill = if selected { step(4) } else { gpui::transparent_black() };
                assert_close(row.background, fill);
                let text = if enabled {
                    SemanticRole::Foreground
                } else {
                    SemanticRole::MutedForeground
                };
                assert_close(row.foreground, role(text));
                assert_close(styled.row(state, true, false, false).border, role(SemanticRole::Focus));
                if enabled {
                    assert_close(styled.row(state, false, true, false).background, step(5));
                    assert_close(styled.row(state, false, false, true).background, step(6));
                }
                assert_eq!((row.radius, row.padding_x, row.font_size), (4.0, 8.0, 14.0));
            }
        }
        let surface = styled.surface(true, true);
        assert_close(surface.background, look.resolve_step(ScaleFamily::Gray, 1).hsla());
        assert_close(surface.border, look.resolve_step(ScaleFamily::Gray, 6).hsla());
        assert_eq!(surface.radius, 8.0);
    }
}
