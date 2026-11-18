use xgpui_luma_l10n::{Error, LayoutDirection, LocaleRequest, LocaleSource, Localizer};

#[test]
fn explicit_precedence_ignores_lower_priority_invalid_input_and_os() {
    let request = LocaleRequest { command_line: Some("AR-eg"), setting: Some("invalid setting!") };
    let system = std::iter::from_fn(|| -> Option<&str> { panic!("OS locales must not be consumed") });
    let selected = request.resolve_with_system(system).unwrap();
    assert_eq!(selected.locale.language_tag(), "ar-EG");
    assert_eq!(selected.source, LocaleSource::CommandLine);
    assert_eq!(request.resolve().unwrap(), selected);

    let request = LocaleRequest { command_line: None, setting: Some("ja-JP") };
    let selected = request.resolve_with_system(["de"]).unwrap();
    assert_eq!(selected.locale.language_tag(), "ja-JP");
    assert_eq!(selected.source, LocaleSource::Setting);
}

#[test]
fn explicit_invalid_values_are_actionable_errors_not_silent_fallbacks() {
    for tag in ["", " ", "not a locale!"] {
        let request = LocaleRequest { command_line: Some(tag), setting: Some("de") };
        let error = request.resolve_with_system(["ja"]).unwrap_err();
        assert!(
            matches!(error, Error::InvalidLocaleSource { origin: LocaleSource::CommandLine, ref value } if value == tag)
        );
        assert!(error.to_string().contains("command-line"));

        let request = LocaleRequest { command_line: None, setting: Some(tag) };
        let error = request.resolve_with_system(["ar"]).unwrap_err();
        assert!(
            matches!(error, Error::InvalidLocaleSource { origin: LocaleSource::Setting, ref value } if value == tag)
        );
        assert!(error.to_string().contains("saved-setting"));
    }
}

#[test]
fn os_preferences_keep_order_and_skip_unusable_entries() {
    let selected = LocaleRequest::default().resolve_with_system(["", "C", "POSIX", "invalid!", "ja-JP", "de"]).unwrap();
    assert_eq!(selected.locale.language_tag(), "ja-JP");
    assert_eq!(selected.source, LocaleSource::System);

    let selected = LocaleRequest::default().resolve_with_system([String::from("ar-u-nu-latn")]).unwrap();
    assert_eq!(selected.locale.language_tag(), "ar-u-nu-latn");
    assert_eq!(selected.locale.direction(), LayoutDirection::Rtl);
}

#[test]
fn absent_or_unusable_system_locales_default_to_english() {
    for system in [vec![], vec!["C", "POSIX", "bad!"]] {
        let selected = LocaleRequest::default().resolve_with_system(system).unwrap();
        assert_eq!(selected.locale.language_tag(), "en");
        assert_eq!(selected.source, LocaleSource::Default);
    }
}

#[test]
fn startup_identity_is_independent_of_available_translations() {
    let selected = LocaleRequest::default().resolve_with_system(["ar-EG"]).unwrap();
    let mut localizer = Localizer::new(selected.locale);
    localizer.add_resource(&"en".parse().unwrap(), "title = English fallback").unwrap();
    assert_eq!(localizer.tr("title"), "English fallback");
    assert_eq!(localizer.locale().language_tag(), "ar-EG");
    assert_eq!(localizer.locale().direction(), LayoutDirection::Rtl);
}

#[test]
fn posix_and_undetermined_tags_are_not_ui_locales() {
    // POSIX parses as a five-letter BCP 47 language but names no language.
    for tag in ["POSIX", "posix", "posix-US"] {
        assert!(tag.parse::<xgpui_luma_l10n::Locale>().is_err(), "{tag}");
    }
    // `und` stays a valid Locale value, but never selects the UI language.
    assert!("und".parse::<xgpui_luma_l10n::Locale>().is_ok());
    for tag in ["POSIX", "und", "und-Latn", "UND"] {
        for (request, origin) in [
            (LocaleRequest { command_line: Some(tag), setting: Some("de") }, LocaleSource::CommandLine),
            (LocaleRequest { command_line: None, setting: Some(tag) }, LocaleSource::Setting),
        ] {
            let error = request.resolve_with_system(["ja"]).unwrap_err();
            assert!(
                matches!(error, Error::InvalidLocaleSource { origin: o, ref value } if o == origin && value == tag),
                "{tag}: {error:?}"
            );
        }
    }
    let selected = LocaleRequest::default().resolve_with_system(["und", "und-Latn", "posix-US", "de-AT"]).unwrap();
    assert_eq!(selected.locale.language_tag(), "de-AT");
    assert_eq!(selected.source, LocaleSource::System);
    let selected = LocaleRequest::default().resolve_with_system(["und"]).unwrap();
    assert_eq!(selected.locale.language_tag(), "en");
    assert_eq!(selected.source, LocaleSource::Default);
}
