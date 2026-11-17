use super::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use gpui::{AnyElement, App, TestAppContext};
use crate::{
    controls::pager::{PagerStyle, PagerTemplate, default_pager_template, default_pager_theme},
    infra::locale::{LocaleContext, NumberFormat, SdkStrings, set_locale_context},
};

struct Probe {
    indicators: Arc<Mutex<Vec<String>>>,
}
impl PagerTemplate for Probe {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.indicators.lock().unwrap().push(model.page_indicator_with_locale(cx).to_string());
        let look = default_pager_theme().resolve(model.enabled, model.style);
        let sizes = super::super::template::render_page_size_select(model, &look, &handlers, cx);
        div()
            .child(default_pager_template().render(model, handlers, window, cx))
            .child(sizes)
            .into_any_element()
    }
}
struct RecordingNumbers(Arc<Mutex<Vec<u64>>>);
impl NumberFormat for RecordingNumbers {
    fn integer(&self, value: u64) -> SharedString {
        self.0.lock().unwrap().push(value);
        format!("n{value}").into()
    }
    fn number(&self, value: f64) -> SharedString {
        value.to_string().into()
    }
}

#[test]
fn live_locale_formats_indicator_and_numbers_without_changing_paging_state() {
    let mut app = TestAppContext::single();
    let indicators = Arc::new(Mutex::new(Vec::new()));
    let capture = indicators.clone();
    let (pager, cx) = app.add_window_view(|_, cx| {
        PagerControl::from_builder(
            PagerBuilder::new("localized-pager")
                .current_page(1)
                .page_count(4)
                .page_size(25)
                .page_size_options([25, 50])
                .template(Arc::new(Probe { indicators: capture })),
            cx,
        )
    });
    cx.run_until_parked();
    assert_eq!(indicators.lock().unwrap().last().unwrap(), "Page 2 of 4");
    let changes = Rc::new(Cell::new(0));
    let events = changes.clone();
    let _subscription =
        cx.update(|_, cx| cx.subscribe(&pager, move |_, _: &PagerEvent, _| events.set(events.get() + 1)));
    // Exercise the shared page-size popup and its labels as well as both pager presentations.
    pager.update(cx, |view, cx| {
        view.page_size_open = true;
        cx.notify();
    });
    for (style, prefix) in [(PagerStyle::MinimalEdge, "Seite"), (PagerStyle::Numeric, "صفحة")] {
        pager.update(cx, |view, cx| {
            view.model.style = style;
            cx.notify();
        });
        let numbers = Arc::new(Mutex::new(Vec::new()));
        let recorded = numbers.clone();
        cx.update(|_, cx| {
            set_locale_context(
                LocaleContext {
                    strings: Arc::new(SdkStrings {
                        page_of: Arc::new(move |page, pages, _| format!("{prefix} {page}/{pages}").into()),
                        ..Default::default()
                    }),
                    numbers: Arc::new(RecordingNumbers(recorded)),
                    ..Default::default()
                },
                cx,
            )
        });
        cx.run_until_parked();
        assert_eq!(indicators.lock().unwrap().last().unwrap(), &format!("{prefix} 2/4"));
        let calls = numbers.lock().unwrap();
        assert!(calls.contains(&25) && calls.contains(&50));
        if style == PagerStyle::Numeric {
            for page in 1..=4 {
                assert!(calls.contains(&page));
            }
        }
        cx.update(|_, cx| {
            let view = pager.read(cx);
            assert_eq!((view.current_page(), view.page_count(), view.page_size()), (1, 4, 25));
            assert!(view.page_size_open);
        });
    }
    assert_eq!(changes.get(), 0);
}

#[test]
fn app_formatters_and_labels_survive_locale_refresh_and_parameter_replacement() {
    for direct in [false, true] {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            set_locale_context(
                LocaleContext {
                    strings: Arc::new(SdkStrings {
                        page_of: Arc::new(|page, pages, _| format!("localized {page}/{pages}").into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
            )
        });
        let indicators = Arc::new(Mutex::new(Vec::new()));
        let capture = indicators.clone();
        let (pager, cx) = app.add_window_view(|_, cx| {
            let mut params = PagerTemplateParameters::default().first_label("").next_label("App next");
            // Both the public field and builder callback must take precedence.
            let mut builder = PagerBuilder::new("custom-pager").page_count(3);
            if direct {
                params.page_indicator_formatter = Arc::new(|_, _| "App indicator".into());
            }
            builder = builder.template_parameters(params);
            if !direct {
                builder = builder.page_indicator_formatter(|_, _| "App indicator".into());
            }
            PagerControl::from_builder(builder.template(Arc::new(Probe { indicators: capture })), cx)
        });
        cx.run_until_parked();
        assert_eq!(indicators.lock().unwrap().last().unwrap(), "App indicator");
        cx.update(|_, cx| set_locale_context(LocaleContext::default(), cx));
        cx.run_until_parked();
        assert_eq!(indicators.lock().unwrap().last().unwrap(), "App indicator");
        cx.update(|_, cx| {
            let view = pager.read(cx);
            assert_eq!(view.render_model().first_label().unwrap(), "");
            assert_eq!(view.render_model().next_label().unwrap(), "App next");
        });
        pager.update(cx, |view, cx| view.set_template_parameters(PagerTemplateParameters::default(), cx));
        cx.run_until_parked();
        assert_eq!(indicators.lock().unwrap().last().unwrap(), "Page 1 of 3");
    }
}

#[test]
fn pager_order_tracks_direction_and_clicks_keep_page_targets() {
    use crate::infra::locale::LayoutDirection;
    for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
        let mut app = TestAppContext::single();
        let (pager, cx) = app.add_window_view(|_, cx| {
            PagerControl::from_builder(PagerBuilder::new("rtl-pager").style(style).current_page(2).page_count(5), cx)
        });
        cx.run_until_parked();
        let changes = Rc::new(Cell::new(0));
        let observed = changes.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&pager, move |_, event: &PagerEvent, _| {
                if matches!(event, PagerEvent::PageChanged { .. }) {
                    observed.set(observed.get() + 1);
                }
            })
        });
        for (index, direction) in
            [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr].into_iter().enumerate()
        {
            pager.update(cx, |view, cx| view.set_page(2, cx));
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            cx.run_until_parked();
            assert_eq!(changes.get(), index);
            let prev = cx.debug_bounds("pager-nav-prev-1").unwrap();
            let next = cx.debug_bounds("pager-nav-next-3").unwrap();
            assert_eq!(prev.left() < next.left(), direction == LayoutDirection::Ltr);
            if style != PagerStyle::Minimal {
                let first = cx.debug_bounds("pager-nav-first-0").unwrap();
                let last = cx.debug_bounds("pager-nav-last-4").unwrap();
                assert_eq!(first.left() < prev.left(), direction == LayoutDirection::Ltr);
                assert_eq!(next.left() < last.left(), direction == LayoutDirection::Ltr);
            }
            if style == PagerStyle::Numeric {
                let one = cx.debug_bounds("pager-page-0").unwrap();
                let five = cx.debug_bounds("pager-page-4").unwrap();
                assert_eq!(one.left() < five.left(), direction == LayoutDirection::Ltr);
            }
            cx.simulate_click(next.center(), Default::default());
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(pager.read(cx).current_page(), 3));
            assert_eq!(changes.get(), index + 1);
        }
    }
}
