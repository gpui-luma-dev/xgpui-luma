use super::*;
use gpui::{TestAppContext, TestDispatcher, TextDirection, size};
use xgpui_luma::theme::ThemeMode;

#[test]
fn locale_resources_format_samples_and_keep_routes_stable() {
    let app = TestAppContext::single();
    for (tag, language, title, number) in [
        ("en", "en", "Localization", "1,234,567.50"),
        ("de", "de", "Lokalisierung", "1.234.567,50"),
        ("ja", "ja", "ローカライズ", "1,234,567.50"),
        ("ar-u-nu-arab", "ar", "الترجمة والتوطين", "١٬٢٣٤٬٥٦٧٫٥٠"),
        ("fr", "fr", "Localization", "1\u{202f}234\u{202f}567,50"),
    ] {
        app.update(|cx| {
            cx.set_global(localizer(tag.parse().unwrap()).unwrap());
            assert_eq!(tr(cx, messages::msg_l10n_title()), title);
            let sample = samples(cx).unwrap();
            assert_eq!(sample.rows[1].1, number);
            assert!(sample.saved.contains("\u{2068}שלום.txt\u{2069}"));
            let ui = localized::options(cx, TextKind::Ui);
            assert_eq!(
                ui.direction,
                if tag.starts_with("ar") {
                    TextDirection::RightToLeft
                } else {
                    TextDirection::LeftToRight
                }
            );
            assert_eq!(localized::options(cx, TextKind::Data).direction, TextDirection::Auto);
            assert_eq!(ui.language.unwrap().as_str(), language);
            assert_eq!(localized::options(cx, TextKind::Data).language, ui.language);
        });
    }
    assert_eq!(crate::app::Route::resolve("Localization"), crate::app::Route::resolve("xgpui-parley/Localization"));
}

#[test]
fn picker_updates_translations_numbers_and_direction_without_recreating_the_page() {
    let mut app = TestAppContext::single();
    app.update(|cx| {
        xgpui_luma::init(cx).unwrap();
        cx.set_global(localizer("en-US".parse().unwrap()).unwrap());
    });
    let (page, cx) = app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), cx));
    cx.run_until_parked();
    let original = page.clone();
    for (tag, greeting) in [("de", "Willkommen"), ("ar", "مرحبًا"), ("en", "Welcome")] {
        let index = cx.update(|_, cx| choices(cx).iter().position(|(id, _)| id == tag).unwrap());
        page.update(cx, |page, cx| page.locale.select(index, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(page, original);
            assert_eq!(cx.global::<Localizer>().locale().language_tag(), tag);
            assert!(samples(cx).unwrap().greeting.starts_with(greeting));
            assert_eq!(page.read(cx).locale.selector.read(cx).selected_id().unwrap(), &index.to_string());
        });
    }
}

#[test]
fn native_and_parley_layouts_keep_slots_and_mirror_preview_rows() {
    for parley in [false, true] {
        let mut app = if parley {
            let backend = crate::parley_lab::application_text_system().unwrap();
            let app = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend.clone());
            app.update(|cx| crate::parley_lab::paragraph_api::install_typeset(backend, cx));
            app
        } else {
            TestAppContext::single()
        };
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        let mode_look = look.clone();
        let (_, cx) = app.add_window_view(|_, cx| Page::new(look, cx));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            mode_look.set_mode(mode);
            for width in [720., 1280.] {
                cx.simulate_resize(size(px(width), px(2400.)));
                for tag in ["en", "ar"] {
                    cx.update(|_, cx| {
                        xgpui_luma_l10n::set_locale(tag.parse().unwrap(), cx);
                    });
                    cx.run_until_parked();
                    for slot in ["l10n-crumb", "l10n-header", "l10n-hero", "l10n-controls", "l10n-note"] {
                        let bounds = cx.debug_bounds(slot).unwrap_or_else(|| panic!("{slot}"));
                        assert!(bounds.size.width > px(0.) && bounds.right() <= px(width), "{slot}: {tag} {bounds:?}");
                    }
                    let key = cx.debug_bounds("l10n-key-0").unwrap();
                    let value = cx.debug_bounds("l10n-value-0").unwrap();
                    if tag == "ar" {
                        assert!(key.left() > value.left());
                    } else {
                        assert!(key.left() < value.left());
                    }
                }
            }
        }
    }
}
