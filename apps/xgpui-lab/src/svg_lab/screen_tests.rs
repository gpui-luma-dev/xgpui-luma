use super::*;
use gpui::{TestAppContext, px};
use xgpui_luma::{
    theme::ThemeMode,
    controls::{button::ButtonEvent, switch::SwitchEvent, textfield::TextFieldEvent},
};
#[test]
fn four_pages_keep_slots_and_fit_both_themes() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        look.set_mode(mode);
        let (page, cx) = app.add_window_view(|_, cx| SvgPage::new(look, cx));
        for (tab, id) in [
            ("Rendering", "svg-rendering"),
            ("Scaling", "svg-scaling"),
            ("Path Effects", "svg-paths"),
            ("Scheduling & Cache", "svg-cache"),
        ] {
            page.update(cx, |p, cx| p.activate(tab.into(), cx));
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(2400.)));
                cx.run_until_parked();
                let selectors = match id {
                    "svg-rendering" => [
                        "svg-rendering-crumb",
                        "svg-rendering-header",
                        "svg-rendering-hero",
                        "svg-rendering-caption",
                        "svg-rendering-controls",
                    ],
                    "svg-scaling" => [
                        "svg-scaling-crumb",
                        "svg-scaling-header",
                        "svg-scaling-hero",
                        "svg-scaling-caption",
                        "svg-scaling-controls",
                    ],
                    "svg-paths" => [
                        "svg-paths-crumb",
                        "svg-paths-header",
                        "svg-paths-hero",
                        "svg-paths-caption",
                        "svg-paths-controls",
                    ],
                    "svg-cache" => [
                        "svg-cache-crumb",
                        "svg-cache-header",
                        "svg-cache-hero",
                        "svg-cache-caption",
                        "svg-cache-controls",
                    ],
                    _ => unreachable!(),
                };
                let bounds = selectors.map(|slot| cx.debug_bounds(slot).unwrap_or_else(|| panic!("{tab}: {slot}")));
                assert!(bounds.windows(2).all(|p| p[0].top() < p[1].top()), "{tab}");
                assert!(bounds.iter().all(|b| b.right() <= px(width)), "{tab}: fits window");
            }
        }
    }
}
#[test]
fn fields_switches_and_reset_keep_camera_and_path_models_independent() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| SvgPage::new(crate::theme::default_look(), cx));
    page.update(cx, |p, cx| p.activate("Scaling".into(), cx));
    let zoom = cx.update(|_, cx| page.read(cx).numbers[0].field.clone());
    zoom.update(cx, |f, cx| {
        f.set_value("4", cx);
        cx.emit(TextFieldEvent::Change { value: "4".into() });
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(page.read(cx).camera.zoom, 4.));
    page.update(cx, |p, cx| p.activate("Path Effects".into(), cx));
    let animate =
        cx.update(|_, cx| page.read(cx).switches.iter().find(|(a, _)| *a == Action::Animate).unwrap().1.clone());
    animate.update(cx, |_, cx| cx.emit(SwitchEvent::Change { on: true }));
    cx.run_until_parked();
    let trim = cx.update(|_, cx| page.read(cx).numbers[3].field.clone());
    trim.update(cx, |f, cx| {
        f.set_value("0.25", cx);
        cx.emit(TextFieldEvent::Change { value: "0.25".into() });
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let p = page.read(cx);
        assert!(!p.animate);
        assert_eq!(p.values[3], 0.25);
        assert_eq!(p.camera.zoom, 4.);
    });
    let reset = cx.update(|_, cx| page.read(cx).buttons.iter().find(|(a, _)| *a == Action::Reset).unwrap().1.clone());
    reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert_eq!(trim.read(cx).value(), "0.680");
        assert_eq!(page.read(cx).camera.zoom, 4.);
    });
}

#[test]
fn rendering_footer_stays_put_while_artwork_status_restarts() {
    use gpui::{VisualTestContext, Render, Window, point, prelude::*};
    use crate::screen::LabScreen;
    struct Probe {
        page: Entity<SvgPage>,
    }
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let page = self.page.read(cx);
            LabScreen::new("svg-status-probe", page.look.clone())
                .section_panel(page.diagnostics())
                .note("Footer below live diagnostics")
        }
    }
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        for width in [1280., 720.] {
            let mut app = TestAppContext::single();
            app.update(|cx| xgpui_luma::init(cx).unwrap());
            let look = crate::theme::default_look();
            look.set_mode(mode);
            let window = app.open_window(size(px(width), px(2400.)), |_, cx| SvgPage::new(look, cx));
            let page = window.root(&mut app).unwrap();
            let mut cx = VisualTestContext::from_window(window.into(), &app);
            cx.run_until_parked();
            let ready = cx.update(|_, cx| page.read(cx).rendering_status.clone().expect("mounted SVG status"));
            // Draw only the diagnostics to observe the pending frame directly:
            // drawing the SVG itself would immediately deliver a new status.
            let probe = cx.update(|_, cx| cx.new(|_| Probe { page: page.clone() }));
            let draw = |cx: &mut VisualTestContext, status: Option<SvgStatus>| {
                page.update(cx, |page, _| page.rendering_status = status);
                probe.update(cx, |_, cx| cx.notify());
                cx.draw(point(px(0.), px(0.)), size(px(width), px(2400.)), |_, _| {
                    gpui::div().size_full().child(probe.clone())
                });
            };
            draw(&mut cx, Some(ready.clone()));
            let footer = cx.debug_bounds("svg-status-probe-note").unwrap();
            for action in [Action::Shadow, Action::Clip, Action::Shadow] {
                page.update(&mut cx, |p, cx| {
                    p.action(action, cx);
                    assert!(p.rendering_status.is_none());
                });
                draw(&mut cx, None);
                assert_eq!(
                    cx.debug_bounds("svg-status-probe-note").unwrap(),
                    footer,
                    "pending status must retain panel height"
                );
                draw(&mut cx, Some(ready.clone()));
                assert_eq!(
                    cx.debug_bounds("svg-status-probe-note").unwrap(),
                    footer,
                    "reported status must retain panel height"
                );
            }
        }
    }
}
