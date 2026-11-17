//! Independent integration probes using the shipped pseudo resources, not synthetic SDK strings.
use std::sync::{Arc, Mutex};
use gpui::{Context, Entity, Focusable, Render, TestAppContext, Window, div, prelude::*, px};
use xgpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use xgpui_luma::infra::locale::{LayoutDirection, LocaleContext};

struct Probe {
    selector: Entity<Selector>,
}
impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(360.)).child(self.selector.clone())
    }
}

#[test]
fn shipped_pseudo_resources_update_open_controls_without_selection_events_or_focus_loss() {
    let mut app = TestAppContext::single();
    app.update(|cx| {
        xgpui_luma::init(cx).unwrap();
        xgpui_luma_l10n::install(super::localizer("en".parse().unwrap()).unwrap(), cx).unwrap();
    });
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let capture = snapshots.clone();
    let (host, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        let selector = Selector::new("audit-selector")
            .items([SelectorItem::new("reset").label("Reset canvas"), SelectorItem::new("keep").label("User data")])
            .motion_enabled(false)
            .with_template_modifier(move |root, model| {
                capture.lock().unwrap().push((
                    model.open,
                    model.label.to_string(),
                    model.items[0].label_text().to_string(),
                ));
                root.debug_selector(|| "audit-selector-trigger".into())
            })
            .spawn(cx);
        Probe { selector: super::localize_selector(selector, cx) }
    });
    cx.run_until_parked();
    let selector = cx.update(|window, cx| {
        let selector = host.read(cx).selector.clone();
        selector.read(cx).focus_handle(cx).focus(window, cx);
        selector
    });
    let events = Arc::new(Mutex::new(0));
    let capture = events.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&selector, move |_, event: &SelectorEvent, _| {
            if matches!(event, SelectorEvent::Change { .. } | SelectorEvent::OpenChanged { .. }) {
                *capture.lock().unwrap() += 1;
            }
        })
    });
    let trigger = cx.debug_bounds("audit-selector-trigger").unwrap();
    cx.simulate_click(trigger.center(), Default::default());
    cx.run_until_parked();
    assert!(snapshots.lock().unwrap().last().unwrap().0);
    *events.lock().unwrap() = 0;
    for tag in ["en-XA", "ar-XB", "en"] {
        cx.update(|_, cx| {
            assert!(xgpui_luma_l10n::set_locale(tag.parse().unwrap(), cx));
        });
        cx.run_until_parked();
        let (open, prompt, item) = snapshots.lock().unwrap().last().unwrap().clone();
        assert!(open);
        cx.update(|window, cx| {
            assert_eq!(prompt, LocaleContext::current(cx).strings.select_placeholder);
            assert_eq!(item, super::resolve("Reset canvas", cx));
            assert!(selector.read(cx).selected_id().is_none());
            assert!(selector.read(cx).focus_handle(cx).is_focused(window));
        });
        let label = cx.debug_bounds("selector-trigger-label").unwrap();
        let caret = cx.debug_bounds("selector-trigger-caret").unwrap();
        let shell = cx.debug_bounds("audit-selector-trigger").unwrap();
        let rtl = cx.update(|_, cx| LocaleContext::current(cx).direction == LayoutDirection::Rtl);
        assert_eq!(label.left() < caret.left(), !rtl);
        for bounds in [label, caret] {
            assert!(bounds.left() >= shell.left() && bounds.right() <= shell.right());
        }
        assert_eq!(*events.lock().unwrap(), 0);
    }
    selector.update(cx, |selector, cx| selector.set_label("Application override", cx));
    cx.update(|_, cx| {
        xgpui_luma_l10n::set_locale("ar-XB".parse().unwrap(), cx);
    });
    cx.run_until_parked();
    assert_eq!(snapshots.lock().unwrap().last().unwrap().1, "Application override");
}

#[test]
fn arabic_plural_selectors_keep_numeric_category_separate_from_isolated_display_digits() {
    let localizer = super::localizer("ar-u-nu-arab".parse().unwrap()).unwrap();
    for (count, suffix) in [(0, "من العناصر"), (1, "عنصر"), (2, "عنصران"), (3, "عناصر"), (11, "عنصرًا"), (100, "عنصر")]
    {
        let number = localizer.locale().fmt_number(&count.into()).unwrap();
        let rendered = localizer.format(super::messages::msg_l10n_items(count, number.clone()));
        assert!(rendered.ends_with(suffix), "{count}: {rendered}");
        assert!(rendered.contains(&format!("\u{2068}{number}\u{2069}")), "{rendered}");
    }
}

#[test]
fn fluent_isolates_reach_parley_without_visible_advance_or_out_of_bounds_selection() {
    struct ParagraphProbe;
    impl Render for ParagraphProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }
    let backend = crate::parley_lab::application_text_system().unwrap();
    let mut app = TestAppContext::build_with_text_system(gpui::TestDispatcher::new(0), None, backend);
    app.update(|cx| {
        xgpui_luma_l10n::install(super::localizer("en".parse().unwrap()).unwrap(), cx).unwrap();
    });
    let (_, cx) = app.add_window_view(|_, _| ParagraphProbe);
    cx.update(|window, cx| {
        let text = cx.global::<xgpui_luma_l10n::Localizer>().format(super::messages::msg_l10n_saved("שלום.txt"));
        assert!(text.contains("\u{2068}שלום.txt\u{2069}"));
        let options = LocaleContext::current(cx).paragraph_options();
        assert_eq!(options.language.as_deref(), Some("en"));
        assert_eq!(options.direction, gpui::TextDirection::LeftToRight);
        let system = window.text_system();
        assert_eq!(system.paragraph_support(), gpui::ParagraphSupport::Full);
        let font_id = system.resolve_font(&gpui::font("IBM Plex Sans"));
        let width = |text: &str| {
            system
                .paragraph_content_widths_with_options(
                    text.to_owned().into(),
                    px(18.),
                    &[gpui::FontRun { len: text.len(), font_id }],
                    &options,
                )
                .unwrap()
                .1
        };
        let without_isolates = text.replace(['\u{2068}', '\u{2069}'], "");
        assert!((width(&text) - width(&without_isolates)).abs() < px(1.), "isolate marks must not draw a glyph");
        let layout = system
            .layout_paragraph_with_options(
                text.clone().into(),
                px(18.),
                &[gpui::FontRun { len: text.len(), font_id }],
                &options,
                px(500.),
            )
            .unwrap();
        let bounds = layout
            .selection_bounds(
                gpui::ParagraphPosition { byte_index: 0, affinity: gpui::TextAffinity::Downstream },
                gpui::ParagraphPosition { byte_index: text.len(), affinity: gpui::TextAffinity::Upstream },
            )
            .unwrap();
        assert!(!bounds.is_empty());
        for rect in bounds {
            assert!(rect.left() >= px(0.) && rect.right() <= px(500.));
        }
    });
}
