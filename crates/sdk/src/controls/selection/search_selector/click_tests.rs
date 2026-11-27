//! Dispatch real popup row clicks, including disabled rows and filtered source indices.
use super::*;
use std::{cell::RefCell, rc::Rc};
use gpui::{Focusable, TestAppContext, div, prelude::*};
use super::super::behavior::SelectionItem;

#[test]
fn disabled_rows_preserve_click_targets_after_filtering() {
    for query in ["", "Match"] {
        for (target, selector_id) in [("a", "option-a"), ("c", "option-c")] {
            let mut app = TestAppContext::single();
            let (selector, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                let builder = SearchSelectorBuilder::new(
                    "row-clicks",
                    [
                        SelectionItem::new("other", "Other").enabled(false),
                        SelectionItem::new("a", "Match A"),
                        SelectionItem::new("b", "Match B").enabled(false),
                        SelectionItem::new("d", "Match D").enabled(false),
                        SelectionItem::new("c", "Match C"),
                    ],
                )
                .motion_enabled(false)
                .with_item_template(|model, _| {
                    let id = model.item.id.clone();
                    div()
                        .id(id.clone())
                        .debug_selector(move || format!("option-{id}"))
                        .w_full()
                        .h(px(24.0))
                        .child(model.item.label.clone())
                });
                SearchSelectorControl::from_builder(builder, cx)
            });
            cx.run_until_parked();
            cx.simulate_click(gpui::point(px(25.0), px(15.0)), Default::default());
            cx.run_until_parked();
            if !query.is_empty() {
                cx.simulate_input(query);
                cx.run_until_parked();
            }
            cx.update(|window, app| {
                let selector = selector.read(app);
                assert!(selector.behavior.state.open);
                assert_eq!(
                    selector.behavior.state.filtered,
                    if query.is_empty() {
                        vec![0, 1, 2, 3, 4]
                    } else {
                        vec![1, 2, 3, 4]
                    }
                );
                assert!(selector.popup_search_textfield.read(app).focus_handle(app).is_focused(window));
            });

            let events = Rc::new(RefCell::new(Vec::new()));
            let capture = events.clone();
            let _subscription = cx.update(|_, app| {
                app.subscribe(&selector, move |_, event: &SearchSelectorEvent, _| {
                    capture.borrow_mut().push(match event {
                        SearchSelectorEvent::Select { item_id, label } => format!("select:{item_id}:{label}"),
                        SearchSelectorEvent::Complete { .. } => "complete".into(),
                        SearchSelectorEvent::Clear => "clear".into(),
                        SearchSelectorEvent::OpenChanged { open } => format!("open:{open}"),
                        SearchSelectorEvent::Dismiss => "dismiss".into(),
                        _ => return,
                    });
                })
            });
            for disabled in ["option-b", "option-d"] {
                let bounds = cx.debug_bounds(disabled).unwrap();
                cx.simulate_click(bounds.center(), Default::default());
                cx.run_until_parked();
                cx.update(|_, app| {
                    assert!(selector.read(app).behavior.state.open);
                    assert_eq!(selector.read(app).committed_selection, None);
                });
                assert!(events.borrow().is_empty());
            }
            let bounds = cx.debug_bounds(selector_id).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            cx.update(|window, app| {
                let selector = selector.read(app);
                assert!(!selector.behavior.state.open, "{query:?}, target={target}");
                let selected = selector.committed_selection.unwrap();
                assert_eq!(selector.model.items[selected].id.as_ref(), target);
                assert!(selector.interaction.focus_handle().is_focused(window));
            });
            let label = if target == "a" { "Match A" } else { "Match C" };
            assert_eq!(*events.borrow(), ["open:false".to_string(), format!("select:{target}:{label}")]);
        }
    }
}

/// A selector theme that counts the trigger looks it resolves, around the SDK's.
struct RecordingSelectorTheme(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl crate::controls::selector::SelectorTheme for RecordingSelectorTheme {
    fn resolve(
        &self,
        trigger_style: crate::controls::selector::SelectorTriggerStyle,
        state: crate::theme::InteractionState,
        without_elevation: bool,
    ) -> crate::controls::selector::SelectorPalette {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        crate::controls::selector::default_selector_theme().resolve(trigger_style, state, without_elevation)
    }

    fn metrics(&self) -> crate::theme::MetricTokens {
        crate::controls::selector::default_selector_theme().metrics()
    }
}

/// A text-field template that counts the frames it draws, around the SDK's.
struct RecordingTextField(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl crate::controls::textfield::TextFieldTemplate for RecordingTextField {
    fn render(
        &self,
        model: &crate::controls::textfield::TextFieldRenderModel<'_>,
        handlers: crate::controls::textfield::TextFieldTemplateHandlers,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::Stateful<gpui::Div> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        crate::controls::textfield::default_textfield_template().render(model, handlers, window, cx)
    }
}

/// `set_selector_theme` reaches the trigger the live search selector draws, and `set_textfield_template` the
/// popup's search field once it opens (#397).
#[test]
fn theme_setters_reach_the_drawn_trigger_and_search_field() {
    let mut app = TestAppContext::single();
    let (selector, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        let builder = SearchSelectorBuilder::new("themed", [SelectionItem::new("a", "A")]).motion_enabled(false);
        SearchSelectorControl::from_builder(builder, cx)
    });
    cx.run_until_parked();
    let (trigger, field) = (
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    );
    let (theme, template) = (
        std::sync::Arc::new(RecordingSelectorTheme(trigger.clone())),
        std::sync::Arc::new(RecordingTextField(field.clone())),
    );
    selector.update(cx, |selector, cx| {
        selector.set_selector_theme(theme, cx);
        selector.set_textfield_template(template, cx);
    });
    cx.run_until_parked();
    assert!(trigger.load(std::sync::atomic::Ordering::SeqCst) > 0, "the trigger is drawn with the new theme");
    selector.update(cx, |selector, cx| selector.open_popup_with_all_items(cx));
    cx.run_until_parked();
    assert!(
        field.load(std::sync::atomic::Ordering::SeqCst) > 0,
        "the open popup's search field uses the new template"
    );
}

/// The trigger draws its focus ring when focused, like the selector's; it never did before (#397).
#[test]
fn the_trigger_draws_a_focus_ring_when_focused() {
    let mut app = TestAppContext::single();
    let (selector, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        let builder = SearchSelectorBuilder::new("focus-ring", [SelectionItem::new("a", "A")]).motion_enabled(false);
        SearchSelectorControl::from_builder(builder, cx)
    });
    cx.run_until_parked();
    let ring = crate::controls::selector::default_selector_theme()
        .resolve(
            crate::controls::selector::SelectorTriggerStyle::default(),
            crate::theme::InteractionState { focused: true, ..Default::default() },
            false,
        )
        .trigger_focus_border;
    assert!(ring.a > 0.0, "the SDK selector theme has a focus ring color");
    let ring_painted = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, _| window.painted_quads().iter().any(|quad| quad.border_color == ring))
    };
    assert!(!ring_painted(cx), "no ring before focus");
    cx.update(|window, app| selector.read(app).focus_handle(app).focus(window, app));
    cx.run_until_parked();
    // One focus-ring rule (#433): focus after pointer input draws none, keyboard input draws it.
    assert!(!ring_painted(cx), "no ring for pointer focus");
    cx.simulate_keystrokes("shift");
    cx.run_until_parked();
    assert!(ring_painted(cx), "a ring once focused from the keyboard");
}
