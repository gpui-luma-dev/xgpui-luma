//! Headless TestWindow checks; these never launch Studio or a platform window.
use std::{cell::Cell, rc::Rc};

use gpui::{Bounds, Pixels, ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, VisualTestContext, div, point};
use xgpui_luma::controls::listbox::{ListBoxEvent, ListBoxSnapshot, SelectionMode};

use std::sync::Arc;
use gpui::{App, Context, Div, Window, prelude::*, px};
use xgpui_luma::controls::listbox::{ListBoxControl, ListBoxInput, ListBoxItemRenderModel, ListBoxState, SelectionPolicy};
use xgpui_luma_look_shadcn::ShadcnLook;
use super::super::{horizontal, vertical};
use super::super::super::event_stream::ControlEventStream;

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector).unwrap_or_else(|| panic!("missing {selector}"))
}

#[test]
fn exposition_geometry_matches_inspector_metrics() {
    let mut app = TestAppContext::single();
    let (_, cx) = app.add_window_view(|_, cx| {
        let look = Arc::new(ShadcnLook::built_in());
        let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "events", ""));
        vertical::VerticalListExample::new(look, events, cx)
    });
    let surface = bounds(cx, "listbox-vertical-surface");
    let viewport = bounds(cx, "listbox-vertical-viewport");
    let first = bounds(cx, "listbox-vertical-item-0");
    let second = bounds(cx, "listbox-vertical-item-1");
    assert_eq!(surface.size.width, px(super::super::VERTICAL_LIST_WIDTH));
    assert_eq!(
        surface.size.height,
        px(vertical::LAYOUT.viewport_height + 2.0 * (vertical::LAYOUT.inset_y + vertical::LAYOUT.border)),
    );
    assert_eq!(viewport.size.height, px(vertical::LAYOUT.viewport_height));
    assert_eq!(first.size.height, px(vertical::LAYOUT.item_height));
    assert_eq!(second.origin.y - first.origin.y - first.size.height, px(vertical::LAYOUT.spacing));
    assert_eq!(viewport.origin.x - surface.origin.x, px(vertical::LAYOUT.inset_x + vertical::LAYOUT.border));
    assert_eq!(viewport.origin.y - surface.origin.y, px(vertical::LAYOUT.inset_y + vertical::LAYOUT.border));

    let mut app = TestAppContext::single();
    let (_, cx) = app.add_window_view(|_, cx| {
        let look = Arc::new(ShadcnLook::built_in());
        let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "events", ""));
        horizontal::HorizontalListExample::new(look, events, cx)
    });
    assert_eq!(
        bounds(cx, "listbox-horizontal-surface").size.height,
        px(horizontal::LAYOUT.viewport_height + 2.0 * (horizontal::LAYOUT.inset_y + horizontal::LAYOUT.border)),
    );
    let first = bounds(cx, "listbox-horizontal-item-0");
    let second = bounds(cx, "listbox-horizontal-item-1");
    assert_eq!(first.size.width, px(horizontal::CARD_WIDTH));
    assert_eq!(first.size.height, px(horizontal::LAYOUT.item_height));
    assert_eq!(second.origin.x - first.origin.x - first.size.width, px(horizontal::LAYOUT.spacing));
    assert_eq!(bounds(cx, "listbox-horizontal-viewport").size.height, px(horizontal::LAYOUT.viewport_height));
}

struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    look: Arc<ShadcnLook>,
    horizontal: bool,
    events: Vec<ListBoxEvent<u32>>,
    parent_wheels: Rc<Cell<usize>>,
}

impl Harness {
    fn new(horizontal: bool, cx: &mut Context<Self>) -> Self {
        let snapshot = ListBoxSnapshot::try_with_enabled(1..=20, |n| *n, |n| *n != 7).unwrap();
        Self {
            list: ListBoxControl::new(
                ListBoxState::from_snapshot(snapshot, SelectionMode::Extended),
                Self::input,
                |key| ("item", *key).into(),
                |item| format!("Item {item}").into(),
                cx,
            )
            .require_focus_for_scroll(true),
            look: Arc::new(ShadcnLook::built_in()),
            horizontal,
            events: Vec::new(),
            parent_wheels: Rc::default(),
        }
    }

    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.events.extend(self.list.apply(input, cx).events);
    }
}

fn named_template(model: &ListBoxItemRenderModel<'_, u32>, _: &mut App) -> Div {
    div().size_full().child(format!("{} {}", model.item, model.selected))
}

impl gpui::Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = if self.horizontal {
            listbox! { window, cx;
                id = "test-list";
                control = &mut self.list;
                look = &self.look;
                width = 250.0;
                padding_x = 16.0;
                padding_y = 8.0;
                scroll_view! { horizontal;
                    hstack! {
                        gap = 8.0;
                        item_width = 136.0;
                        item_height = 36.0;
                        item_template = named_template;
                    }
                }
            }
        } else {
            // A local, borrowed, non-Send capture works: markup doesn't retain templates.
            let prefix = Rc::new(String::from("Item"));
            listbox! { window, cx;
                id = "test-list";
                control = &mut self.list;
                look = &self.look;
                width = 250.0;
                padding_x = 16.0;
                padding_y = 8.0;
                scroll_view! { vertical;
                    visible_items = 5;
                    vstack! {
                        gap = 4.0;
                        item_height = 36.0;
                        item_template = |model, _cx| {
                            div().size_full().child(format!("{} {}", prefix, model.item))
                        };
                    }
                }
            }
        };
        let parent_wheels = self.parent_wheels.clone();
        div().id("parent").child(surface).on_scroll(move |_, _, _| {
            parent_wheels.set(parent_wheels.get() + 1);
        })
    }
}

#[test]
fn inline_and_named_templates_preserve_selection_focus_and_reveal_on_both_axes() {
    for horizontal in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, cx));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        cx.simulate_keystrokes(if horizontal { "shift-right" } else { "shift-down" });
        cx.update(|_, app| {
            let host = view.read(app);
            assert!(host.list.state.is_focused());
            assert_eq!(host.list.state.selected_keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        });
        cx.simulate_keystrokes("end");
        let viewport = bounds(cx, "test-list-viewport");
        let last = bounds(cx, "test-list-item-19");
        if horizontal {
            assert!(last.right() <= viewport.right());
            assert!(last.origin.x >= viewport.origin.x);
        } else {
            assert!(last.bottom() <= viewport.bottom());
            assert!(last.origin.y >= viewport.origin.y);
        }
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                assert_eq!(host.list.state.active_key(), Some(&20));
                let update = host.list.set_selection_policy(SelectionPolicy::new(SelectionMode::None), cx);
                assert!(update.changed);
                assert_eq!(host.list.state.selected_keys().count(), 0);
            })
        });
        cx.simulate_keystrokes("home");
        cx.simulate_keystrokes(if horizontal {
            "right right right right right right"
        } else {
            "down down down down down down"
        });
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.active_key(), Some(&8)); // Disabled seventh item is skipped.
            assert_eq!(host.list.state.selected_keys().count(), 0);
        });
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                let update =
                    host.list.state.replace_snapshot(ListBoxSnapshot::try_new([], |item: &u32| *item).unwrap());
                host.list.handle_update(&update, cx);
            })
        });
        assert_eq!(bounds(cx, "test-list-viewport").size, viewport.size);
    }
}

fn wheel(cx: &mut VisualTestContext) {
    cx.simulate_event(ScrollEvent {
        position: point(px(30.0), px(25.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-60.0))),
        modifiers: Default::default(),
        source: gpui::ScrollSource::Pointer,
        touch_phase: TouchPhase::Moved,
    });
}

#[test]
fn wheel_routing_and_scroll_position_survive_markup_rebuilds() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness::new(false, cx));
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    let before = bounds(cx, "test-list-item-0");
    wheel(cx);
    assert_eq!(bounds(cx, "test-list-item-0").origin, before.origin);
    cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
    wheel(cx);
    let scrolled = bounds(cx, "test-list-item-0");
    assert!(scrolled.origin.y < before.origin.y);
    cx.update(|_, app| view.update(app, |_, cx| cx.notify()));
    assert_eq!(bounds(cx, "test-list-item-0").origin, scrolled.origin);
    cx.update(|window, app| {
        assert_eq!(view.read(app).parent_wheels.get(), 1);
        let elsewhere = app.focus_handle();
        elsewhere.focus(window, app);
    });
    cx.run_until_parked();
    wheel(cx);
    cx.update(|_, app| {
        assert!(!view.read(app).list.state.is_focused());
        assert_eq!(view.read(app).parent_wheels.get(), 2);
    });
    assert_eq!(bounds(cx, "test-list-item-0").origin, scrolled.origin);
}
