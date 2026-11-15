// Shared headless rendering contract, exercised through each look's public builder.
use std::{cell::RefCell, rc::Rc};
use gpui::{Context, Div, Render, Stateful, TestAppContext, VisualTestContext, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxFlow, ListBoxInput, ListBoxItemTemplate, ListBoxSnapshot, ListBoxState,
    ListBoxVirtualization, SelectionMode,
};

struct RenderHarness {
    list: ListBoxControl<Self, u32, u32>,
    flow: ListBoxFlow,
    rows: Rc<RefCell<Vec<u32>>>,
}
impl RenderHarness {
    fn new(flow: ListBoxFlow, policy: ListBoxVirtualization, cx: &mut Context<Self>) -> Self {
        Self {
            list: ListBoxControl::new(
                ListBoxState::from_snapshot(
                    ListBoxSnapshot::try_with_enabled(1..=1000, |key| *key, |key| *key != 2).unwrap(),
                    SelectionMode::Extended,
                ),
                Self::input,
                |key| ("row", *key).into(),
                |key| key.to_string().into(),
                cx,
            )
            .virtualization(policy),
            flow,
            rows: Rc::default(),
        }
    }
    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.list.apply(input, cx);
    }
}
impl Render for RenderHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.rows.borrow_mut().clear();
        let rows = self.rows.clone();
        let content_sized = matches!(self.flow, ListBoxFlow::VerticalContent { .. });
        let template: ListBoxItemTemplate<'_, u32> = Box::new(move |model, _| {
            rows.borrow_mut().push(*model.item);
            div()
                .w_full()
                .when(content_sized, |row| row.h(px(if model.item % 2 == 0 { 70.0 } else { 40.0 })))
                .child(model.item.to_string())
                .into_any_element()
        });
        render_contract_list(&mut self.list, self.flow, template, window, cx).w(px(250.0))
    }
}
fn settle_render(cx: &mut VisualTestContext) {
    for _ in 0..30 {
        cx.run_until_parked();
        if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
            return;
        }
    }
    panic!("listbox layout did not settle");
}

#[test]
fn shared_renderer_preserves_windows_content_and_scroll_on_updates() {
    let modes = [
        (
            ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 },
            ListBoxVirtualization::Eager,
        ),
        (
            ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 },
            ListBoxVirtualization::Uniform { overscan: 2 },
        ),
        (
            ListBoxFlow::Horizontal { item_width: 100.0, item_height: 36.0, gap: 4.0 },
            ListBoxVirtualization::Uniform { overscan: 2 },
        ),
        (ListBoxFlow::VerticalContent { viewport_height: 240.0, gap: 4.0 }, ListBoxVirtualization::Eager),
        (
            ListBoxFlow::VerticalContent { viewport_height: 240.0, gap: 4.0 },
            ListBoxVirtualization::Measured { estimated_height: 48.0, overscan: 2 },
        ),
    ];
    for (flow, policy) in modes {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| RenderHarness::new(flow, policy, cx));
        settle_render(cx);
        cx.update(|_, app| {
            let host = view.read(app);
            let rows = host.rows.borrow();
            assert!(rows.contains(&1));
            if policy == ListBoxVirtualization::Eager {
                assert_eq!(rows.len(), 1000);
            } else {
                assert!(rows.len() < 15, "off-screen templates were constructed");
            }
        });
        cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center(500, cx)));
        settle_render(cx);
        let viewport = cx.debug_bounds("contract-list-viewport").unwrap();
        let before = cx.debug_bounds("contract-list-item-499").unwrap();
        if matches!(flow, ListBoxFlow::Horizontal { .. }) {
            assert!(before.left() >= viewport.left() && before.right() <= viewport.right());
        } else {
            assert!(before.top() >= viewport.top() && before.bottom() <= viewport.bottom());
        }
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                let update = host.list.state.replace_snapshot(
                    ListBoxSnapshot::try_with_enabled(1..=1001, |key| *key, |key| *key != 2).unwrap(),
                );
                host.list.handle_update(&update, cx);
            })
        });
        settle_render(cx);
        assert_eq!(cx.debug_bounds("contract-list-item-499").unwrap(), before);
        cx.update(|_, app| {
            let host = view.read(app);
            let rows = host.rows.borrow();
            assert!(rows.contains(&500));
            if policy != ListBoxVirtualization::Eager {
                assert!(rows.len() < 15 && !rows.contains(&1));
            }
        });
    }
}

#[test]
fn shared_renderer_retains_focus_and_skips_disabled_rows() {
    for horizontal in [false, true] {
        let flow = if horizontal {
            ListBoxFlow::Horizontal { item_width: 100.0, item_height: 36.0, gap: 4.0 }
        } else {
            ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 }
        };
        let mut app = TestAppContext::single();
        let (view, cx) =
            app.add_window_view(|_, cx| RenderHarness::new(flow, ListBoxVirtualization::Uniform { overscan: 2 }, cx));
        cx.update(|window, _| window.activate_window());
        settle_render(cx);
        let first = cx.debug_bounds("contract-list-item-0").unwrap();
        cx.simulate_click(first.center(), Default::default());
        cx.simulate_keystrokes(if horizontal { "right" } else { "down" });
        settle_render(cx);
        cx.update(|_, app| {
            let host = view.read(app);
            assert!(host.list.state.is_focused());
            assert_eq!(host.list.state.active_key(), Some(&3));
        });
        cx.simulate_keystrokes("shift-end");
        settle_render(cx);
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.active_key(), Some(&1000));
            assert!(!host.list.state.selected_keys().any(|key| *key == 2));
            assert!(host.rows.borrow().contains(&1000));
        });
    }
}
