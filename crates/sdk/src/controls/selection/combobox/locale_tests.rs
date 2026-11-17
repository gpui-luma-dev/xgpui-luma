use super::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use gpui::{Focusable, TestAppContext};
use crate::{
    controls::textfield::{ThemedTextFieldTemplate, default_textfield_theme},
    infra::locale::{LocaleContext, SdkStrings, set_locale_context},
};

#[test]
fn locale_defaults_update_the_retained_field_without_editing_query_or_emitting_changes() {
    for explicit in [None, Some("Select…"), Some(""), Some("App prompt")] {
        let mut app = TestAppContext::single();
        let rendered = Arc::new(Mutex::new(String::new()));
        let capture = rendered.clone();
        let (combo, cx) = app.add_window_view(|_, cx| {
            let template = ThemedTextFieldTemplate::new(default_textfield_theme()).with_modifier(move |root, model| {
                *capture.lock().unwrap() = model.placeholder.to_string();
                root
            });
            let mut builder =
                ComboBoxBuilder::new("localized-combo", [super::super::behavior::SelectionItem::new("alpha", "Alpha")])
                    .textfield_template(Arc::new(template))
                    .motion_enabled(false);
            if let Some(text) = explicit {
                builder = builder.placeholder(text);
            }
            ComboBoxControl::from_builder(builder, cx)
        });
        cx.run_until_parked();
        assert_eq!(*rendered.lock().unwrap(), explicit.unwrap_or("Select…"));
        cx.update(|window, cx| combo.read(cx).textfield.read(cx).focus_handle(cx).focus(window, cx));
        cx.simulate_input("alp");
        cx.run_until_parked();
        let before = cx.update(|_, cx| {
            let view = combo.read(cx);
            (view.behavior.state.query.clone(), view.committed_selection, view.textfield.read(cx).value())
        });
        let changes = Rc::new(Cell::new(0));
        let events = changes.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&combo, move |_, event: &ComboBoxEvent, _| {
                if matches!(event, ComboBoxEvent::Change { .. }) {
                    events.set(events.get() + 1);
                }
            })
        });
        for prompt in ["Auswählen…", "اختر…", "Select…"] {
            cx.update(|_, cx| {
                set_locale_context(
                    LocaleContext {
                        strings: Arc::new(SdkStrings { select_placeholder: prompt.into(), ..Default::default() }),
                        ..Default::default()
                    },
                    cx,
                )
            });
            cx.run_until_parked();
            assert_eq!(*rendered.lock().unwrap(), explicit.unwrap_or(prompt));
            cx.update(|window, cx| {
                let view = combo.read(cx);
                assert_eq!(
                    (view.behavior.state.query.clone(), view.committed_selection, view.textfield.read(cx).value()),
                    before
                );
                assert!(view.textfield.read(cx).focus_handle(cx).is_focused(window));
            });
        }
        assert_eq!(changes.get(), 0);
    }
}

#[test]
fn construction_uses_the_already_installed_locale() {
    let mut app = TestAppContext::single();
    app.update(|cx| {
        set_locale_context(
            LocaleContext {
                strings: Arc::new(SdkStrings { select_placeholder: "Auswählen…".into(), ..Default::default() }),
                ..Default::default()
            },
            cx,
        )
    });
    let (view, cx) =
        app.add_window_view(|_, cx| ComboBoxControl::from_builder(ComboBoxBuilder::new("initial", []), cx));
    cx.update(|_, cx| assert_eq!(view.read(cx).model.placeholder, "Auswählen…"));
}

#[test]
fn rtl_affordances_and_editor_insets_switch_without_editing_retained_input() {
    use crate::infra::locale::LayoutDirection;
    let mut app = TestAppContext::single();
    let insets = Arc::new(Mutex::new((0.0, 0.0)));
    let capture = insets.clone();
    let (combo, cx) = app.add_window_view(|_, cx| {
        let template = ThemedTextFieldTemplate::new(default_textfield_theme()).with_modifier(move |root, model| {
            *capture.lock().unwrap() = model.content_insets;
            root
        });
        ComboBoxControl::from_builder(
            ComboBoxBuilder::new(
                "rtl-combo",
                [
                    super::super::behavior::SelectionItem::new("alpha", "Alpha"),
                    super::super::behavior::SelectionItem::new("alpine", "Alpine"),
                ],
            )
            .textfield_template(Arc::new(template))
            .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| combo.read(cx).textfield.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_input("al");
    cx.run_until_parked();
    let before = cx.update(|_, cx| {
        let view = combo.read(cx);
        (view.behavior.state.query.clone(), view.committed_selection, view.textfield.read(cx).value())
    });
    let changes = Rc::new(Cell::new(0));
    let capture = changes.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&combo, move |_, event: &ComboBoxEvent, _| {
            if matches!(event, ComboBoxEvent::Change { .. }) {
                capture.set(capture.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr, LayoutDirection::Rtl] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let arrow = cx.debug_bounds("combobox-arrow").unwrap();
        let clear = cx.debug_bounds("combobox-clear").unwrap();
        assert_eq!(clear.left() < arrow.left(), direction == LayoutDirection::Ltr);
        let viewport = cx.debug_bounds("textfield-editor-viewport").unwrap();
        if direction == LayoutDirection::Rtl {
            assert!(viewport.left() >= clear.right());
        } else {
            assert!(viewport.right() <= clear.left());
        }
        let (left, right) = *insets.lock().unwrap();
        if direction == LayoutDirection::Rtl {
            assert!(left > 0.);
            assert_eq!(right, 0.);
        } else {
            assert!(right > 0.);
            assert_eq!(left, 0.);
        }
        cx.update(|window, cx| {
            let view = combo.read(cx);
            assert_eq!(
                (view.behavior.state.query.clone(), view.committed_selection, view.textfield.read(cx).value()),
                before
            );
            assert!(view.textfield.read(cx).focus_handle(cx).is_focused(window));
        });
    }
    assert_eq!(changes.get(), 0);
    let clear = cx.debug_bounds("combobox-clear").unwrap();
    cx.simulate_click(clear.center(), Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(combo.read(cx).textfield.read(cx).value().is_empty()));
}
