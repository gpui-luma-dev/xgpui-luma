use super::*;
use gpui::{Focusable, TestAppContext};
use std::sync::{Arc, Mutex};
use crate::controls::textfield::{ThemedTextFieldTemplate, default_textfield_theme};
use crate::infra::locale::{LocaleContext, SdkStrings, set_locale_context};
#[test]
fn autocomplete_localizes_implicit_prompts_without_editing_query_or_explicit_copy() {
    for explicit in [None, Some("Search…"), Some(""), Some("App prompt")] {
        let mut app = TestAppContext::single();
        let rendered = Arc::new(Mutex::new(String::new()));
        let capture = rendered.clone();
        let (control, cx) = app.add_window_view(|_, cx| {
            let template = ThemedTextFieldTemplate::new(default_textfield_theme()).with_modifier(move |root, model| {
                *capture.lock().unwrap() = model.placeholder.to_string();
                root
            });
            let mut builder = AutocompleteBuilder::new(
                "localized-autocomplete",
                [super::super::behavior::SelectionItem::new("alpha", "Alpha")],
            )
            .textfield_template(Arc::new(template))
            .motion_enabled(false);
            if let Some(copy) = explicit {
                builder = builder.placeholder(copy);
            }
            AutocompleteControl::from_builder(builder, cx)
        });
        cx.run_until_parked();
        assert_eq!(*rendered.lock().unwrap(), explicit.unwrap_or("Search…"));
        cx.update(|window, cx| control.read(cx).textfield.read(cx).focus_handle(cx).focus(window, cx));
        cx.simulate_input("alp");
        cx.run_until_parked();
        let before = cx.update(|_, cx| control.read(cx).textfield.read(cx).value());
        for prompt in ["Suchen…", "بحث…", "Search…"] {
            cx.update(|_, cx| {
                set_locale_context(
                    LocaleContext {
                        strings: Arc::new(SdkStrings { search_placeholder: prompt.into(), ..Default::default() }),
                        ..Default::default()
                    },
                    cx,
                )
            });
            cx.run_until_parked();
            assert_eq!(*rendered.lock().unwrap(), explicit.unwrap_or(prompt));
            cx.update(|window, cx| {
                assert_eq!(control.read(cx).textfield.read(cx).value(), before);
                assert!(control.read(cx).textfield.read(cx).focus_handle(cx).is_focused(window));
            });
        }
    }
}
