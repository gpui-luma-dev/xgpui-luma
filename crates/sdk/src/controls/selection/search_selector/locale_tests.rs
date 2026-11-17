use super::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use gpui::TestAppContext;
use crate::{
    controls::textfield::{ThemedTextFieldTemplate, default_textfield_theme},
    infra::locale::{LocaleContext, SdkStrings, set_locale_context},
};

#[test]
fn trigger_and_retained_search_prompt_follow_locale_but_preserve_app_text_and_input() {
    for explicit in [None, Some("Select…"), Some(""), Some("App prompt")] {
        let mut app = TestAppContext::single();
        let rendered = Arc::new(Mutex::new(String::new()));
        let capture = rendered.clone();
        let (selector, cx) = app.add_window_view(|_, cx| {
            let template = ThemedTextFieldTemplate::new(default_textfield_theme()).with_modifier(move |root, model| {
                *capture.lock().unwrap() = model.placeholder.to_string();
                root
            });
            let mut builder = SearchSelectorBuilder::new(
                "localized-search",
                [super::super::behavior::SelectionItem::new("alpha", "Alpha")],
            )
            .textfield_template(Arc::new(template))
            .motion_enabled(false);
            if let Some(text) = explicit {
                builder = builder.placeholder(text).search_placeholder(text);
            }
            SearchSelectorControl::from_builder(builder, cx)
        });
        cx.run_until_parked();
        cx.simulate_click(gpui::point(px(25.), px(15.)), Default::default());
        cx.run_until_parked();
        cx.simulate_input("alp");
        cx.run_until_parked();
        let before = cx.update(|_, cx| {
            let view = selector.read(cx);
            assert!(view.behavior.state.open);
            assert!(!view.behavior.state.filtered.is_empty());
            (
                view.behavior.state.query.clone(),
                view.committed_selection,
                view.popup_search_textfield.read(cx).value(),
            )
        });
        let changes = Rc::new(Cell::new(0));
        let events = changes.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&selector, move |_, event: &SearchSelectorEvent, _| {
                if matches!(event, SearchSelectorEvent::Change { .. }) {
                    events.set(events.get() + 1);
                }
            })
        });
        for (prompt, search, empty) in
            [("Auswählen…", "Auswahl suchen", "Keine Treffer"), ("اختر…", "بحث", "لا توجد نتائج")]
        {
            cx.update(|_, cx| {
                set_locale_context(
                    LocaleContext {
                        strings: Arc::new(SdkStrings {
                            select_placeholder: prompt.into(),
                            selection_search: search.into(),
                            no_matches: empty.into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    cx,
                )
            });
            cx.run_until_parked();
            assert_eq!(*rendered.lock().unwrap(), explicit.unwrap_or(search));
            cx.update(|window, cx| {
                let view = selector.read(cx);
                assert_eq!(view.model.placeholder, explicit.unwrap_or(prompt));
                assert_eq!(view.model.no_matches, empty);
                assert_eq!(
                    (
                        view.behavior.state.query.clone(),
                        view.committed_selection,
                        view.popup_search_textfield.read(cx).value()
                    ),
                    before
                );
                assert!(view.behavior.state.open && !view.behavior.state.filtered.is_empty());
                assert!(view.popup_search_textfield.read(cx).focus_handle(cx).is_focused(window));
            });
        }
        assert_eq!(changes.get(), 0);
    }
}

#[test]
fn runtime_trigger_override_is_not_replaced_by_locale_refresh() {
    let mut app = TestAppContext::single();
    let (selector, cx) =
        app.add_window_view(|_, cx| SearchSelectorControl::from_builder(SearchSelectorBuilder::new("runtime", []), cx));
    cx.run_until_parked();
    selector.update(cx, |view, cx| view.set_placeholder("Select…", cx));
    cx.update(|_, cx| {
        set_locale_context(
            LocaleContext {
                strings: Arc::new(SdkStrings { select_placeholder: "Auswählen…".into(), ..Default::default() }),
                ..Default::default()
            },
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(selector.read(cx).model.placeholder, "Select…"));
}

#[test]
fn rtl_refresh_mirrors_trigger_and_filtered_rows_without_editing_query_or_selection() {
    use crate::infra::locale::LayoutDirection;
    use super::super::behavior::SelectionItem;
    use lucide_svg_static::Icon;
    let mut app = TestAppContext::single();
    let (selector, cx) = app.add_window_view(|_, cx| {
        SearchSelectorControl::from_builder(
            SearchSelectorBuilder::new(
                "rtl-search",
                [
                    SelectionItem::new("alpha", "Alpha").trailing_icon(Icon::Check),
                    SelectionItem::new("alpine", "Alpine").trailing_icon(Icon::Check),
                    SelectionItem::new("disabled", "Albatross").enabled(false).trailing_icon(Icon::Check),
                ],
            )
            .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.simulate_click(gpui::point(px(25.), px(15.)), Default::default());
    cx.run_until_parked();
    cx.simulate_input("al");
    cx.run_until_parked();
    let before = cx.update(|_, cx| {
        let view = selector.read(cx);
        assert!(view.behavior.state.open);
        (
            view.behavior.state.query.clone(),
            view.committed_selection,
            view.popup_search_textfield.read(cx).value(),
        )
    });
    let changes = Rc::new(Cell::new(0));
    let capture = changes.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&selector, move |_, event: &SearchSelectorEvent, _| {
            if matches!(event, SearchSelectorEvent::Change { .. }) {
                capture.set(capture.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let label = cx.debug_bounds("search-selector-label").unwrap();
        let icon = cx.debug_bounds("search-selector-icon").unwrap();
        assert_eq!(label.left() < icon.left(), direction == LayoutDirection::Ltr);
        let mut bottom = None;
        for (label_id, icon_id, row_id) in [
            ("search-row-label-0", "search-row-icon-0", "search-row-0"),
            ("search-row-label-1", "search-row-icon-1", "search-row-1"),
            ("search-row-label-2", "search-row-icon-2", "search-row-2"),
        ] {
            let label = cx.debug_bounds(label_id).unwrap();
            let icon = cx.debug_bounds(icon_id).unwrap();
            let row = cx.debug_bounds(row_id).unwrap();
            assert_eq!(label.left() < icon.left(), direction == LayoutDirection::Ltr);
            if let Some(previous) = bottom {
                assert!(row.top() >= previous);
            }
            bottom = Some(row.bottom());
        }
        cx.update(|window, cx| {
            let view = selector.read(cx);
            assert!(view.behavior.state.open);
            assert_eq!(
                (
                    view.behavior.state.query.clone(),
                    view.committed_selection,
                    view.popup_search_textfield.read(cx).value()
                ),
                before
            );
            assert!(view.popup_search_textfield.read(cx).focus_handle(cx).is_focused(window));
        });
    }
    assert_eq!(changes.get(), 0);
}
