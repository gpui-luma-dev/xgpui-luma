//! Custom chrome and content use the same shared builder as the look adapters.
use std::{cell::RefCell, rc::Rc};
use gpui::{Render, TestAppContext, VisualTestContext};
use super::*;
use crate::controls::listbox::{ListBoxDragLook, ListBoxItemState, ListBoxMotion, ListBoxSnapshot, SelectionMode};

struct Template;
impl ListBoxTemplate for Template {
    fn render(&self, model: &ListBoxRenderModel, _: &mut App) -> Surface {
        div().id(model.id).debug_selector(|| "custom-surface".into()).relative().p(px(7.0))
    }
    fn render_row(&self, model: &ListBoxRowRenderModel, _: &mut App) -> Surface {
        div().id(model.id.clone()).size_full().pl(px(13.0))
    }
    fn drag_look(&self) -> ListBoxDragLook {
        ListBoxDragLook {
            marker_color: gpui::white(),
            marker_width: 3.0,
            surface_color: None,
            preview: TooltipLook::compact(gpui::black(), gpui::white(), 3.0),
        }
    }
    fn motion_defaults(&self) -> ListBoxMotion {
        ListBoxMotion::default()
    }
}
struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    rendered: Rc<RefCell<Vec<(u32, ListBoxItemState)>>>,
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
            rendered: Rc::default(),
        }
    }
    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.list.apply(input, cx);
    }
}
impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.rendered.borrow_mut().clear();
        let rendered = self.rendered.clone();
        let layout = ListBoxFlow::Vertical { visible_items: 3, item_height: 36.0, gap: 4.0 }.layout((0.0, 0.0));
        ListBoxBuilder::from_control("custom-list", &mut self.list, layout, move |surface, item, _| {
            rendered.borrow_mut().push((item.key, item.state));
            surface.child(div().debug_selector(|| format!("custom-content-{}", item.key)).size(px(10.0)))
        })
        .empty(div().debug_selector(|| "custom-empty".into()).size(px(20.0)))
        .build(&Template, window, cx)
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
    panic!("custom listbox layout did not settle");
}
#[test]
fn shared_builder_preserves_custom_surface_row_content_and_empty_hooks() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness::new(cx));
    settle(cx);
    let surface = cx.debug_bounds("custom-surface").unwrap();
    let viewport = cx.debug_bounds("custom-list-viewport").unwrap();
    let row = cx.debug_bounds("custom-list-item-0").unwrap();
    let content = cx.debug_bounds("custom-content-1").unwrap();
    assert_eq!(viewport.left() - surface.left(), px(7.0));
    assert_eq!(content.left() - row.left(), px(13.0));
    cx.update(|_, app| assert!(view.read(app).rendered.borrow().len() <= 5));
    cx.update(|_, app| view.update(app, |host, cx| host.list.apply(ListBoxInput::Select(1), cx)));
    settle(cx);
    cx.update(|_, app| {
        assert!(view.read(app).rendered.borrow().iter().any(|(key, state)| *key == 1 && state.selected));
        view.update(app, |host, cx| {
            host.list.set_projection([], cx).unwrap();
        });
    });
    settle(cx);
    let surface = cx.debug_bounds("custom-surface").unwrap();
    let empty = cx.debug_bounds("custom-empty").unwrap();
    assert_eq!(surface.center(), empty.center());
    cx.update(|_, app| {
        assert!(view.read(app).rendered.borrow().is_empty());
        assert_eq!(view.read(app).list.state.snapshot().items().len(), 100);
    });
}
