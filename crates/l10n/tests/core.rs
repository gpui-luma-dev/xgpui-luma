use xgpui_luma_l10n::{FluentArgs, LayoutDirection, Locale, Localizer};

fn locale(tag: &str) -> Locale {
    tag.parse().unwrap()
}

#[test]
fn canonical_tags_extensions_and_script_direction() {
    assert_eq!(locale("AR-eg").language_tag(), "ar-EG");
    assert_eq!(locale("ar-u-nu-latn").language_tag(), "ar-u-nu-latn");
    for tag in ["ar", "he", "fa", "ur", "en-Arab", "ar-XB"] {
        assert_eq!(locale(tag).direction(), LayoutDirection::Rtl, "{tag}");
    }
    for tag in ["en", "ja", "ar-Latn", "en-XA"] {
        assert_eq!(locale(tag).direction(), LayoutDirection::Ltr, "{tag}");
    }
    assert!("not a locale!".parse::<Locale>().is_err());
    assert_eq!(locale("ar-EG").fallback_chain(), ["ar-EG", "ar", "en"]);
    assert_eq!(locale("en-US").fallback_chain(), ["en-US", "en"]);
    assert_eq!(locale("zh-Hant-TW").fallback_chain(), ["zh-Hant-TW", "zh-Hant", "zh", "en"]);
    assert_eq!(locale("ar-u-nu-latn").fallback_chain(), ["ar-u-nu-latn", "ar", "en"]);
}

#[test]
fn per_message_fallback_overrides_and_runtime_switching() {
    let mut l10n = Localizer::new(locale("ar-EG"));
    l10n.add_resource(&locale("en"), "title = English\nonly-en = Fallback\n").unwrap();
    l10n.add_resource(&locale("ar"), "title = العربية\n").unwrap();
    l10n.add_resource(&locale("ar-EG"), "regional = مصري\n").unwrap();
    assert_eq!(l10n.tr("title"), "العربية");
    assert_eq!(l10n.tr("only-en"), "Fallback");
    assert_eq!(l10n.tr("regional"), "مصري");
    l10n.add_resource(&locale("ar"), "title = App override\n").unwrap();
    assert_eq!(l10n.tr("title"), "App override");
    l10n.set_locale(locale("ja"));
    assert_eq!(l10n.tr("title"), "English");
    let expected = if cfg!(debug_assertions) {
        "⟦missing⟧"
    } else {
        "missing"
    };
    assert_eq!(l10n.tr("missing"), expected);
}

#[test]
fn fluent_plurals_and_bidi_isolation() {
    let mut l10n = Localizer::new(locale("en"));
    l10n.add_resource(
        &locale("en"),
        "saved = Saved { $file }\ncount = { $n ->\n    [one] One item\n   *[other] { $n } items\n}\n",
    )
    .unwrap();
    let mut args = FluentArgs::new();
    args.set("file", "שלום.txt");
    assert_eq!(l10n.tr_args("saved", &args), "Saved \u{2068}שלום.txt\u{2069}");
    args.set("n", 1);
    assert_eq!(l10n.tr_args("count", &args), "One item");
    args.set("n", 3);
    assert_eq!(l10n.tr_args("count", &args), "\u{2068}3\u{2069} items");
}

#[test]
fn malformed_resources_are_atomic_and_format_errors_fall_back() {
    let mut l10n = Localizer::new(locale("de"));
    l10n.add_resource(&locale("en"), "title = Original\n").unwrap();
    assert!(l10n.add_resource(&locale("en"), "title = Changed\nbroken = {\n").is_err());
    l10n.add_resource(&locale("de"), "title = { $missing }\n").unwrap();
    assert_eq!(l10n.tr("title"), "Original");
}

#[test]
fn localizer_is_send_and_sync_for_gpui_global() {
    fn check<T: Send + Sync>() {}
    check::<Localizer>();
}
