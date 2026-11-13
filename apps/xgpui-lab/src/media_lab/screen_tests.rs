//! Headless screen and control regressions; no native application is launched.
use super::*;
use gpui::{TestAppContext, px, size};
use xgpui_luma::{
    controls::{button::ButtonEvent, textfield::TextFieldEvent},
    theme::ThemeMode,
};

#[test]
fn media_screens_render_in_both_themes_and_resize_without_losing_the_camera() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        look.set_mode(mode);
        let (page, cx) = app.add_window_view(|_, cx| MediaPage::new(look, cx));
        page.update(cx, |page, _| page.fixtures = Some(Fixtures::new(None).unwrap()));
        for (tab, hero, panels) in [
            ("Images", "media-images-hero", "media-images-panels"),
            ("Textures & Mipmaps", "media-mipmaps-hero", "media-mipmaps-panels"),
            ("Linear Adjustments", "media-linear-hero", "media-linear-panels"),
            ("Float / P3", "media-float-hero", "media-float-panels"),
            ("Resource Limits", "media-resources-hero", "media-resources-panels"),
            ("Video Frames", "media-video-hero", "media-video-panels"),
        ] {
            page.update(cx, |page, cx| {
                page.activate(tab.into(), cx);
                // Exercise the empty/stopped source layout without allocating native video surfaces.
                if tab == "Video Frames" {
                    page.video_stopped = true;
                }
            });
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(1800.)));
                cx.run_until_parked();
                let bounds = cx.debug_bounds(hero).unwrap_or_else(|| panic!("{tab}: hero"));
                assert!(bounds.size.width > px(0.) && bounds.size.width <= px(width), "{tab}: bounded hero");
                assert!(cx.debug_bounds(panels).is_some(), "{tab}: panels");
                if matches!(tab, "Images" | "Textures & Mipmaps" | "Linear Adjustments" | "Float / P3") {
                    page.update(cx, |page, cx| page.zoom_at(gpui::point(px(0.), px(0.)), 1.1, cx));
                    cx.run_until_parked();
                    cx.update(|_, cx| {
                        let page = page.read(cx);
                        let zoom = &page.filter_fields.iter().find(|(id, _)| *id == Parameter::Zoom).unwrap().1;
                        assert_eq!(zoom.read(cx).value(), Parameter::Zoom.format(page.values[0]));
                    });
                }
            }
        }
    }
}

#[test]
fn linear_fields_switches_and_reset_keep_retained_tab_values_in_sync() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| MediaPage::new(crate::theme::default_look(), cx));
    page.update(cx, |page, cx| {
        page.fixtures = Some(Fixtures::new(None).unwrap());
        page.activate("Linear Adjustments".into(), cx);
    });
    let exposure = cx.update(|_, cx| {
        page.read(cx).filter_fields.iter().find(|(id, _)| *id == Parameter::Exposure).unwrap().1.clone()
    });
    exposure.update(cx, |field, cx| {
        field.set_value("1.25", cx);
        cx.emit(TextFieldEvent::Change { value: "1.25".into() });
    });
    cx.run_until_parked();
    page.update(cx, |page, cx| {
        assert_eq!(page.values[Parameter::Exposure as usize], 1.25);
        page.activate("Images".into(), cx);
        page.activate("Linear Adjustments".into(), cx);
        assert_eq!(page.values[Parameter::Exposure as usize], 1.25);
        page.action(Action::Animate, cx);
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let page = page.read(cx);
        assert!(*page.state_switches.iter().find(|(id, _)| *id == Action::Animate).unwrap().1.read(cx).data());
    });
    let reset = cx.update(|_, cx| page.read(cx).buttons.iter().find(|(id, _)| *id == Action::Reset).unwrap().1.clone());
    reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    cx.update(|_, cx| {
        let page = page.read(cx);
        assert_eq!(page.values[Parameter::Exposure as usize], 0.);
        assert_eq!(exposure.read(cx).value(), "0.00");
        assert!(!*page.state_switches.iter().find(|(id, _)| *id == Action::Animate).unwrap().1.read(cx).data());
    });
}
