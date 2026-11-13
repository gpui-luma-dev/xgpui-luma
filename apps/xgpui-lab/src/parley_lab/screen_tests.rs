use super::*;
use xgpui_luma::{
    theme::ThemeMode,
    controls::{button::ButtonEvent, textfield::TextFieldEvent},
};
#[::core::prelude::v1::test]
fn seven_pages_keep_screen_slots_and_fit_both_themes() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        look.set_mode(mode);
        let (page, cx) = app.add_window_view(|_, cx| ParleyPage::new(look, cx));
        for (tab, id) in [
            ("Text Shaping", "parley-shaping"),
            ("Line Layout", "parley-line"),
            ("Paragraphs", "parley-paragraph"),
            ("Text Field", "parley-fields"),
            ("Fonts & Fallback", "parley-fonts"),
            ("Rasterization", "parley-raster"),
            ("Scrolling", "parley-scroll"),
        ] {
            page.update(cx, |p, cx| p.activate(tab.into(), cx));
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(2400.)));
                cx.run_until_parked();
                let selectors = match id {
                    "parley-shaping" => [
                        "parley-shaping-crumb",
                        "parley-shaping-header",
                        "parley-shaping-hero",
                        "parley-shaping-caption",
                    ],
                    "parley-line" => {
                        ["parley-line-crumb", "parley-line-header", "parley-line-hero", "parley-line-caption"]
                    }
                    "parley-paragraph" => [
                        "parley-paragraph-crumb",
                        "parley-paragraph-header",
                        "parley-paragraph-hero",
                        "parley-paragraph-caption",
                    ],
                    "parley-fields" => {
                        ["parley-fields-crumb", "parley-fields-header", "parley-fields-hero", "parley-fields-caption"]
                    }
                    "parley-fonts" => {
                        ["parley-fonts-crumb", "parley-fonts-header", "parley-fonts-hero", "parley-fonts-caption"]
                    }
                    "parley-raster" => {
                        ["parley-raster-crumb", "parley-raster-header", "parley-raster-hero", "parley-raster-caption"]
                    }
                    "parley-scroll" => {
                        ["parley-scroll-crumb", "parley-scroll-header", "parley-scroll-hero", "parley-scroll-caption"]
                    }
                    _ => unreachable!(),
                };
                let bounds = selectors.map(|slot| cx.debug_bounds(slot).unwrap_or_else(|| panic!("{tab}: {slot}")));
                assert!(bounds.windows(2).all(|p| p[0].top() < p[1].top()), "{tab}");
                assert!(bounds.iter().all(|b| b.right() <= px(width)), "{tab}: fits window");
            }
        }
    }
}
#[::core::prelude::v1::test]
fn numeric_edits_rebuild_masks_and_reset_the_retained_controls() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| ParleyPage::new(crate::theme::default_look(), cx));
    let field = cx.update(|_, cx| page.read(cx).numbers[0].field.clone());
    field.update(cx, |f, cx| {
        f.set_value("36", cx);
        cx.emit(TextFieldEvent::Change { value: "36".into() });
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let p = page.read(cx);
        assert_eq!(p.font_size, 36.);
        assert_eq!(field.read(cx).value(), "36");
        assert!(p.rows.as_ref().unwrap().is_ok());
    });
    let reset = cx.update(|_, cx| page.read(cx).buttons[1].clone());
    reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert_eq!(page.read(cx).font_size, 28.);
        assert_eq!(field.read(cx).value(), "28.0");
    });
}

#[::core::prelude::v1::test]
fn text_field_variants_fit_with_the_live_parley_backend() {
    use super::text_field::page::{TextFieldPage, Screen};
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let backend = Engine::new().unwrap().backend.clone();
        let mut app = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            cx.set_global(ParleyTextSystemActive);
        });
        let look = crate::theme::default_look();
        look.set_mode(mode);
        let (page, cx) = app.add_window_view(|_, cx| TextFieldPage::new(look, false, true, cx));
        for screen in [Screen::Fields, Screen::RightToLeft, Screen::TextAreas, Screen::Scrolling, Screen::States] {
            page.update(cx, |p, cx| p.set_screen(screen, cx));
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(2000.)));
                cx.run_until_parked();
                for slot in ["parley-fields-hero", "parley-fields-controls"] {
                    let b = cx.debug_bounds(slot).unwrap();
                    assert!(b.right() <= px(width), "{screen:?}: {slot}");
                }
            }
        }
    }
}

#[::core::prelude::v1::test]
fn scrolling_receives_completed_diagnostics_without_restarting_the_list() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| ParleyPage::new(crate::theme::default_look(), cx));
    page.update(cx, |p, cx| p.activate("Scrolling".into(), cx));
    cx.run_until_parked();
    let bounds = cx.debug_bounds("parley-scroll-hero").unwrap();
    page.update(cx, |p, cx| {
        p.scroll_report = Some(Ok("Finished CPU corpus measurement".into()));
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let p = page.read(cx);
        assert_eq!(p.scroll.read(cx).lab_report.as_deref(), Some("Finished CPU corpus measurement"));
    });
    assert_eq!(cx.debug_bounds("parley-scroll-hero").unwrap(), bounds);
}
