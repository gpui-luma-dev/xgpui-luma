use super::*;

#[test]
fn defaults_are_lower_priority_than_app_resources_before_and_after_preparation() {
    let mut localizer = Localizer::new("de".parse().unwrap());
    localizer
        .add_resource(&"de".parse().unwrap(), "sdk-next = App Weiter\n-brand = App\nsdk-clear = { -brand } löschen")
        .unwrap();
    prepare(&mut localizer).unwrap();
    let snapshot = context(&localizer);
    assert_eq!(snapshot.strings.next, "App Weiter");
    assert_eq!(snapshot.strings.clear, "App löschen");
    assert_eq!(snapshot.strings.cancel, "Abbrechen");
    localizer.add_resource(&"de".parse().unwrap(), "sdk-next = Neuer Text").unwrap();
    prepare(&mut localizer).unwrap();
    assert_eq!(context(&localizer).strings.next, "Neuer Text");
    localizer.set_locale("fr".parse().unwrap());
    assert_eq!(context(&localizer).strings.next, "Next");
    assert_eq!(context(&localizer).language.as_deref(), Some("fr"));
}

#[test]
fn sdk_defaults_do_not_overwrite_app_overrides_in_fallback_locales() {
    let mut localizer = Localizer::new("ar-EG-u-nu-latn".parse().unwrap());
    localizer.add_resource(&"en".parse().unwrap(), "sdk-cancel = Custom Cancel").unwrap();
    localizer.add_resource(&"ar".parse().unwrap(), "sdk-next = تطبيق").unwrap();
    prepare(&mut localizer).unwrap();
    assert_eq!(context(&localizer).strings.next, "تطبيق");
    assert_eq!(context(&localizer).strings.cancel, "إلغاء");
    assert_eq!(context(&localizer).direction, SdkDirection::Rtl);
    assert_eq!(context(&localizer).language.as_deref(), Some("ar-EG-u-nu-latn"));
    localizer.set_locale("fr".parse().unwrap());
    assert_eq!(context(&localizer).strings.cancel, "Custom Cancel");
}

#[test]
fn icu_hooks_preserve_unsigned_precision_and_handle_nonfinite_values() {
    let de = IcuNumbers("de".parse().unwrap());
    assert_eq!(de.integer(u64::MAX), "18.446.744.073.709.551.615");
    assert_eq!(de.number(1234.5), "1.234,5");
    assert_eq!(de.percent(0.125), "12,5\u{a0}%");
    assert_eq!(de.range(-1.5, 2.5), "-1,5–2,5");
    assert_eq!(de.number(1.1234567), "1,123457");
    let ar = IcuNumbers("ar-u-nu-arab".parse().unwrap());
    assert_eq!(ar.integer(1234), "١٬٢٣٤");
    assert_eq!(IcuNumbers("ar-u-nu-latn".parse().unwrap()).integer(1234), "1,234");
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(de.number(value), PlainNumberFormat.number(value));
        assert_eq!(de.percent(value), PlainNumberFormat.percent(value));
    }
}

#[cfg(feature = "build-support")]
#[test]
fn embedded_sdk_catalogs_have_identical_ids_and_arguments() {
    use crate::build_support::Catalog;
    let english = Catalog::from_sources([RESOURCES[0]]).unwrap();
    for (tag, source) in RESOURCES {
        let translation = Catalog::from_sources([(*tag, *source)]).unwrap();
        assert!(english.compare(&translation).is_empty(), "{tag}");
    }
}

#[cfg(feature = "test-support")]
mod headless {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn installation_switching_and_resource_sync_keep_sdk_and_engine_in_step() {
        let app = TestAppContext::single();
        app.update(|cx| {
            crate::install(Localizer::new("en".parse().unwrap()), cx).unwrap();
            let old_page_formatter = LocaleContext::current(cx).strings.page_of.clone();
            assert_eq!(LocaleContext::current(cx).strings.next, "Next");
            assert!(crate::set_locale("de".parse().unwrap(), cx));
            let sdk = LocaleContext::current(cx);
            assert_eq!(sdk.language.as_deref(), Some("de"));
            assert_eq!(sdk.strings.next, "Weiter");
            assert_eq!(sdk.numbers.number(1234.5), "1.234,5");
            assert_eq!(old_page_formatter(2, 8, cx), "Seite \u{2068}2\u{2069} von \u{2068}8\u{2069}");
            assert!(crate::set_locale("ar-u-nu-arab".parse().unwrap(), cx));
            let sdk = LocaleContext::current(cx);
            assert_eq!(sdk.direction, SdkDirection::Rtl);
            assert_eq!(sdk.numbers.integer(1234), "١٬٢٣٤");
            assert_eq!(sdk.paragraph_options().direction, gpui::TextDirection::RightToLeft);
            assert_eq!(sdk.data_paragraph_options().direction, gpui::TextDirection::Auto);
            assert_eq!(cx.global::<Localizer>().locale().language_tag(), "ar-u-nu-arab");
            cx.global_mut::<Localizer>().add_resource(&"ar".parse().unwrap(), "sdk-next = جديد").unwrap();
            assert!(sync(cx).unwrap());
            assert_eq!(LocaleContext::current(cx).strings.next, "جديد");
        });
    }

    #[test]
    fn table_strings_follow_locale_and_app_overrides_with_exact_isolated_counts() {
        let app = TestAppContext::single();
        app.update(|cx| {
            crate::install(Localizer::new("en".parse().unwrap()), cx).unwrap();
            let summary = LocaleContext::current(cx).strings.selection_summary.clone();
            assert_eq!(summary(0, 1, cx), "Selected rows: \u{2068}0\u{2069} of \u{2068}1\u{2069}");
            for (tag, label, word) in [
                ("de", "Zeilen pro Seite", "Ausgewählte Zeilen"),
                ("ja", "ページあたりの行数", "選択した行"),
                ("ar-u-nu-arab", "صفوف في الصفحة", "الصفوف المحددة"),
            ] {
                assert!(crate::set_locale(tag.parse().unwrap(), cx));
                let sdk = LocaleContext::current(cx);
                assert_eq!(sdk.strings.rows_per_page, label);
                let output = summary(1, usize::MAX, cx);
                assert!(output.contains(word));
                let total = sdk.numbers.integer(usize::MAX as u64);
                assert!(output.contains(&format!("\u{2068}{total}\u{2069}")));
            }
            cx.global_mut::<Localizer>()
                .add_resource(
                    &"ar".parse().unwrap(),
                    "sdk-rows-per-page = App rows\nsdk-selection-summary = { $total } / { $selected }",
                )
                .unwrap();
            sync(cx).unwrap();
            assert_eq!(LocaleContext::current(cx).strings.rows_per_page, "App rows");
            assert_eq!(summary(1, 3, cx), "\u{2068}٣\u{2069} / \u{2068}١\u{2069}");
        });
    }

    #[test]
    fn missing_localizer_is_safe_and_existing_sdk_context_is_untouched() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let context = LocaleContext { language: Some("custom".into()), ..Default::default() };
            cx.set_global(context);
            assert!(!sync(cx).unwrap());
            assert!(!crate::set_locale("de".parse().unwrap(), cx));
            assert_eq!(LocaleContext::current(cx).language.as_deref(), Some("custom"));
            assert_eq!(page_of(2, 8, cx), "Page 2 of 8");
            assert_eq!(selection_summary(1, 3, cx), "Selected rows: 1 of 3");
        });
    }

    #[test]
    fn global_observers_see_matching_engine_and_sdk_snapshots() {
        use std::{cell::Cell, rc::Rc};
        let app = TestAppContext::single();
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let _subscription = app.update(|cx| {
            crate::install(Localizer::new("en".parse().unwrap()), cx).unwrap();
            cx.observe_global::<Localizer>(move |cx| {
                let engine = cx.global::<Localizer>();
                let sdk = LocaleContext::current(cx);
                assert_eq!(sdk.language.as_deref(), Some(engine.locale().language_tag().as_str()));
                assert_eq!(sdk.strings.next.as_ref(), engine.tr("sdk-next"));
                if engine.locale().language_tag() == "de" {
                    assert_eq!(sdk.numbers.number(1234.5), "1.234,5");
                    observed.set(observed.get() + 1);
                }
            })
        });
        app.update(|cx| {
            assert!(crate::set_locale("de".parse().unwrap(), cx));
        });
        assert!(calls.get() > 0);
    }

    #[test]
    fn repeated_installation_keeps_app_overrides_and_exact_page_values() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let mut localizer = Localizer::new("ja".parse().unwrap());
            localizer
                .add_resource(&"ja".parse().unwrap(), "sdk-page-of = { $page } / { $pages }\nsdk-next = Custom")
                .unwrap();
            crate::install(localizer, cx).unwrap();
            assert_eq!(LocaleContext::current(cx).strings.next, "Custom");
            let output = (LocaleContext::current(cx).strings.page_of)(usize::MAX, usize::MAX, cx);
            let expected = LocaleContext::current(cx).numbers.integer(usize::MAX as u64);
            assert_eq!(output, format!("\u{2068}{expected}\u{2069} / \u{2068}{expected}\u{2069}"));
            crate::install(Localizer::new("de".parse().unwrap()), cx).unwrap();
            assert_eq!(LocaleContext::current(cx).strings.next, "Weiter");
        });
    }
}

#[test]
fn english_resources_match_every_static_sdk_default() {
    let mut localizer = Localizer::new("en".parse().unwrap());
    prepare(&mut localizer).unwrap();
    let snapshot = context(&localizer);
    let actual = &snapshot.strings;
    let expected = SdkStrings::default();
    for (actual, expected) in [
        (&actual.cancel, &expected.cancel),
        (&actual.clear, &expected.clear),
        (&actual.first, &expected.first),
        (&actual.previous, &expected.previous),
        (&actual.next, &expected.next),
        (&actual.last, &expected.last),
        (&actual.no_matches, &expected.no_matches),
        (&actual.select_placeholder, &expected.select_placeholder),
        (&actual.search_placeholder, &expected.search_placeholder),
        (&actual.selection_search, &expected.selection_search),
        (&actual.rows_per_page, &expected.rows_per_page),
    ] {
        assert_eq!(actual, expected);
    }
}

#[test]
fn malformed_default_resource_set_does_not_partially_replace_app_bundles() {
    let mut localizer = Localizer::new("en".parse().unwrap());
    localizer.add_resource(&"en".parse().unwrap(), "sdk-next = Existing").unwrap();
    assert!(localizer.prepend_defaults(&[("en", "sdk-cancel = Added"), ("de", "sdk-next = {")]).is_err());
    assert_eq!(localizer.tr("sdk-next"), "Existing");
    let expected = if cfg!(debug_assertions) {
        "⟦sdk-cancel⟧"
    } else {
        "sdk-cancel"
    };
    assert_eq!(localizer.tr("sdk-cancel"), expected);
}

#[cfg(feature = "test-support")]
#[test]
fn autocomplete_copy_uses_live_locale_and_isolates_app_data() {
    let app = gpui::TestAppContext::single();
    app.update(|cx| {
        crate::install(Localizer::new("en".parse().unwrap()), cx).unwrap();
        let formatter = LocaleContext::current(cx).strings.autocomplete_status.clone();
        crate::set_locale("de".parse().unwrap(), cx);
        let (status, detail) = formatter("searching", Some("שלום"), 1234, "input", "left", cx);
        assert!(status.contains("Suche"));
        assert!(detail.contains("1.234"));
        assert!(detail.contains("\u{2068}שלום\u{2069}"));
        crate::set_locale("ja".parse().unwrap(), cx);
        let (status, detail) = formatter("idle", None, 0, "clear", "escape", cx);
        assert!(status.contains("待機中"));
        assert!(detail.contains("なし"));
    });
}
