//! Exercise custom template surfaces/content through the public shared renderer.
use std::sync::{Arc, Mutex};
use gpui::{Render, TestAppContext, VisualTestContext};
use super::*;
use crate::controls::listbox::{
    ListBoxControl, ListBoxFlow, ListBoxInput, ListBoxItemState, ListBoxSnapshot, SelectionMode,
};
use crate::controls::tooltip::TooltipLook;

#[derive(Default)]
struct Calls {
    surfaces: usize,
    rows: Vec<(u32, ListBoxItemState)>,
}
struct Template(Arc<Mutex<Calls>>);
impl ListBoxTemplate for Template {
    fn render(&self, model: &ListBoxRenderModel, _: &mut App) -> Stateful<Div> {
        self.0.lock().unwrap().surfaces += 1;
        div().id(model.id).debug_selector(|| "custom-surface".into()).relative().p(px(7.0))
    }
    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Stateful<Div> {
        // The renderer must retain this chrome when composing arbitrary content.
        div().id(model.id.clone()).pl(px(13.0))
    }
    fn drop_color(&self) -> gpui::Hsla {
        gpui::white()
    }
    fn surface_drop_color(&self) -> Option<gpui::Hsla> {
        None
    }
    fn drop_marker_width(&self) -> f32 {
        3.0
    }
    fn drag_preview_style(&self) -> TooltipLook {
        TooltipLook::compact(gpui::black(), gpui::white(), 3.0)
    }
}
struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    calls: Arc<Mutex<Calls>>,
}
impl Harness {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            list: ListBoxControl::new(
                ListBoxState::from_snapshot(
                    ListBoxSnapshot::try_new(1..=100, |key| *key).unwrap(),
                    SelectionMode::Extended,
                ),
                Self::input,
                |key| ("row", *key).into(),
                |key| key.to_string().into(),
                cx,
            )
            .virtualization(ListBoxVirtualization::Uniform { overscan: 1 }),
            calls: Arc::default(),
        }
    }
    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.list.apply(input, cx);
    }
}
impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.calls.lock().unwrap().rows.clear();
        let parts = self.list.render_parts();
        let model = ListBoxRenderModel {
            id: "custom-list",
            label: "Custom collection",
            layout: ListBoxFlow::Vertical { visible_items: 3, item_height: 36.0, gap: 4.0 }.layout((0.0, 0.0)),
            axis: ListBoxAxis::Vertical,
            item_width: 0.0,
            content_sized: false,
            focused: parts.state.is_focused(),
        };
        render_listbox(
            &model,
            ListBoxRenderBindings {
                state: parts.state,
                binding: parts.binding,
                scroll: parts.scroll,
                on_input: parts.on_input,
                item_id: parts.item_id,
                virtualization: parts.virtualization,
                drag_drop: None,
                drop_layout: Some(parts.drop_layout),
                motion_enabled: false,
            },
            &Template(self.calls.clone()),
            |surface, item, _| {
                self.calls.lock().unwrap().rows.push((item.key, item.state));
                surface.child(div().debug_selector(|| format!("custom-content-{}", item.key)).size(px(10.0)))
            },
            Some(div().debug_selector(|| "custom-empty".into()).size(px(20.0)).into_any_element()),
            window,
            cx,
        )
        .w(px(200.0))
    }
}
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..10 {
        cx.run_until_parked();
        if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
            return;
        }
    }
    panic!("custom template did not settle");
}

#[test]
fn shared_template_preserves_surface_row_content_and_empty_hooks() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness::new(cx));
    settle(cx);
    let surface = cx.debug_bounds("custom-surface").unwrap();
    let viewport = cx.debug_bounds("custom-list-viewport").unwrap();
    let row = cx.debug_bounds("custom-list-item-0").unwrap();
    let content = cx.debug_bounds("custom-content-1").unwrap();
    assert_eq!(viewport.left() - surface.left(), px(7.0));
    assert_eq!(content.left() - row.left(), px(13.0));
    cx.update(|_, app| {
        let calls = view.read(app).calls.lock().unwrap();
        assert!(calls.surfaces > 0);
        assert!(calls.rows.len() <= 5, "off-screen custom content was built");
    });
    cx.update(|_, app| view.update(app, |host, cx| host.list.apply(ListBoxInput::Select(1), cx)));
    settle(cx);
    cx.update(|_, app| {
        assert!(view.read(app).calls.lock().unwrap().rows.iter().any(|(key, state)| *key == 1 && state.selected));
        view.update(app, |host, cx| {
            host.list.set_projection([], cx).unwrap();
        });
    });
    settle(cx);
    let surface = cx.debug_bounds("custom-surface").unwrap();
    let empty = cx.debug_bounds("custom-empty").unwrap();
    assert_eq!(surface.center(), empty.center());
    cx.update(|_, app| {
        let host = view.read(app);
        assert!(host.calls.lock().unwrap().rows.is_empty());
        assert_eq!(host.list.state.snapshot().items().len(), 100);
    });
}
