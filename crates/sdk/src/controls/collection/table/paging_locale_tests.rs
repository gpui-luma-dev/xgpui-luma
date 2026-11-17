use super::*;
use std::{cell::Cell, rc::Rc, sync::Mutex};
use gpui::TestAppContext;
use crate::{
    controls::table::{TableScrollMode, TableSelectionMode},
    infra::locale::{SdkStrings, set_locale_context},
};

struct Harness {
    table: PagingTable<i32>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.table.clone())
    }
}

#[test]
fn footer_uses_live_locale_selection_and_item_counts_without_paging_events() {
    let mut app = TestAppContext::single();
    let (table, cx) = app.add_window_view(|_, cx| {
        let table = PagingTableBuilder::new(
            TableBuilder::new_typed("localized-table")
                .items([1, 2, 3])
                .row_label(|row: &i32| row.to_string())
                .selection_mode(TableSelectionMode::Multiple)
                .selected_indices([0])
                .scroll_mode(TableScrollMode::Paged { page_size: 2 }),
            PagerBuilder::new("table-footer"),
        )
        .spawn(cx);
        Harness { table }
    });
    cx.run_until_parked();
    let (list, pager) = cx.update(|_, cx| {
        let table = table.read(cx).table.read(cx);
        (table.list().clone(), table.pager().clone())
    });
    let events = Rc::new(Cell::new(0));
    let capture = events.clone();
    let _subscription =
        cx.update(|_, cx| cx.subscribe(&pager, move |_, _: &PagerEvent, _| capture.set(capture.get() + 1)));
    for prefix in ["Ausgewählt", "المحدد"] {
        let rendered = Arc::new(Mutex::new(Vec::new()));
        let capture = rendered.clone();
        cx.update(|_, cx| {
            set_locale_context(
                LocaleContext {
                    strings: Arc::new(SdkStrings {
                        rows_per_page: "localized rows".into(),
                        selection_summary: Arc::new(move |selected, total, _| {
                            let text = format!("{prefix} {selected}/{total}");
                            capture.lock().unwrap().push(text.clone());
                            text.into()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
            )
        });
        cx.run_until_parked();
        assert_eq!(rendered.lock().unwrap().last().unwrap(), &format!("{prefix} 1/3"));
        list.update(cx, |view, cx| view.set_selected_indices([0, 1], cx));
        cx.run_until_parked();
        assert_eq!(rendered.lock().unwrap().last().unwrap(), &format!("{prefix} 2/3"));
        list.update(cx, |view, cx| view.set_items([1, 2, 3, 4], cx).unwrap());
        cx.run_until_parked();
        assert_eq!(rendered.lock().unwrap().last().unwrap(), &format!("{prefix} 2/4"));
        cx.update(|_, cx| {
            assert_eq!((pager.read(cx).current_page(), pager.read(cx).page_size()), (0, 2));
            assert_eq!(list.read(cx).selected_indices(), &[0, 1]);
        });
        let rendered_count = rendered.lock().unwrap().len();
        pager.update(cx, |view, cx| view.set_info_text(Some("Runtime app footer"), cx));
        cx.run_until_parked();
        assert_eq!(rendered.lock().unwrap().len(), rendered_count);
        pager.update(cx, |view, cx| view.set_info_text(None::<String>, cx));
        cx.run_until_parked();
        assert!(rendered.lock().unwrap().len() > rendered_count);
        // Restore the input state for the next locale without touching the locale.
        list.update(cx, |view, cx| {
            view.set_items([1, 2, 3], cx).unwrap();
            view.set_selected_indices([0], cx);
        });
        cx.run_until_parked();
    }
    assert_eq!(events.get(), 0);
}

#[test]
fn app_footer_text_and_slots_take_precedence_over_default_summary() {
    for slot in [false, true] {
        let mut app = TestAppContext::single();
        let calls = Arc::new(Mutex::new(0));
        let capture = calls.clone();
        app.update(|cx| {
            set_locale_context(
                LocaleContext {
                    strings: Arc::new(SdkStrings {
                        selection_summary: Arc::new(move |_, _, _| {
                            *capture.lock().unwrap() += 1;
                            "Unexpected default".into()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
            )
        });
        let (table, cx) = app.add_window_view(|_, cx| {
            let mut pager = PagerBuilder::new("custom-footer").page_size_label("");
            if slot {
                pager = pager.info_slot(Arc::new(|_, _, _| div().child("App slot").into_any_element()));
            } else {
                pager = pager.info_text("App info");
            }
            let table = PagingTableBuilder::new(
                TableBuilder::new_typed("custom-table").items([1, 2]).row_label(|row: &i32| row.to_string()),
                pager,
            )
            .spawn(cx);
            Harness { table }
        });
        cx.run_until_parked();
        let list = cx.update(|_, cx| table.read(cx).table.read(cx).list().clone());
        list.update(cx, |view, cx| view.set_selected_indices([0], cx));
        cx.run_until_parked();
        assert_eq!(*calls.lock().unwrap(), 0);
    }
}
