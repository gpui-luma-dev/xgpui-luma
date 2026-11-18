use xgpui_luma_l10n::{FluentArgs, Locale, Localizer};

fn locale(tag: &str) -> Locale {
    tag.parse().unwrap()
}

#[test]
fn canonical_tags_extensions_and_fallback_chains() {
    assert_eq!(locale("FR-ca").language_tag(), "fr-CA");
    assert_eq!(locale("th-u-nu-thai").language_tag(), "th-u-nu-thai");
    assert!("not a locale!".parse::<Locale>().is_err());
    assert_eq!(locale("fr-CA").fallback_chain(), ["fr-CA", "fr", "en"]);
    assert_eq!(locale("en-US").fallback_chain(), ["en-US", "en"]);
    assert_eq!(locale("zh-Hant-TW").fallback_chain(), ["zh-Hant-TW", "zh-Hant", "zh", "en"]);
    assert_eq!(locale("th-u-nu-thai").fallback_chain(), ["th-u-nu-thai", "th", "en"]);
}

#[test]
fn per_message_fallback_overrides_and_runtime_switching() {
    let mut l10n = Localizer::new(locale("fr-CA"));
    l10n.add_resource(&locale("en"), "title = English\nonly-en = Fallback\n").unwrap();
    l10n.add_resource(&locale("fr"), "title = Français\n").unwrap();
    l10n.add_resource(&locale("fr-CA"), "regional = Québécois\n").unwrap();
    assert_eq!(l10n.tr("title"), "Français");
    assert_eq!(l10n.tr("only-en"), "Fallback");
    assert_eq!(l10n.tr("regional"), "Québécois");
    l10n.add_resource(&locale("fr"), "title = App override\n").unwrap();
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
fn fluent_plurals_without_isolation_marks() {
    let mut l10n = Localizer::new(locale("en"));
    l10n.add_resource(
        &locale("en"),
        "saved = Saved { $file }\ncount = { $n ->\n    [one] One item\n   *[other] { $n } items\n}\n",
    )
    .unwrap();
    let mut args = FluentArgs::new();
    args.set("file", "report.txt");
    assert_eq!(l10n.tr_args("saved", &args), "Saved report.txt");
    args.set("n", 1);
    assert_eq!(l10n.tr_args("count", &args), "One item");
    args.set("n", 3);
    assert_eq!(l10n.tr_args("count", &args), "3 items");
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
