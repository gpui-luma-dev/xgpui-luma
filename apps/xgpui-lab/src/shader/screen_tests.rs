//! Headless regressions for the first shader LabScreen conversions.
use super::{Action, Parameter, ShaderPage};
use gpui::{TestAppContext, px, size};
use xgpui_luma::{
    controls::{button::ButtonEvent, switch::SwitchEvent, textfield::TextFieldEvent},
    theme::ThemeMode,
};

#[test]
fn core_shader_screens_keep_slot_order_and_fit_both_themes() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let look = crate::theme::default_look();
        look.set_mode(mode);
        let (page, cx) = app.add_window_view(|_, cx| ShaderPage::new(look, cx));
        for (tab, slots) in [
            (
                "Gradients",
                [
                    "shader-gradients-crumb",
                    "shader-gradients-header",
                    "shader-gradients-hero",
                    "shader-gradients-caption",
                    "shader-gradients-controls",
                    "shader-gradients-panels",
                ],
            ),
            (
                "Gradient Paints",
                [
                    "shader-gradient-paints-crumb",
                    "shader-gradient-paints-header",
                    "shader-gradient-paints-hero",
                    "shader-gradient-paints-caption",
                    "shader-gradient-paints-controls",
                    "shader-gradient-paints-panels",
                ],
            ),
            (
                "WGSL Paints",
                [
                    "shader-wgsl-crumb",
                    "shader-wgsl-header",
                    "shader-wgsl-hero",
                    "shader-wgsl-caption",
                    "shader-wgsl-controls",
                    "shader-wgsl-panels",
                ],
            ),
            (
                "Instancing",
                [
                    "shader-instancing-crumb",
                    "shader-instancing-header",
                    "shader-instancing-hero",
                    "shader-instancing-caption",
                    "shader-instancing-controls",
                    "shader-instancing-panels",
                ],
            ),
        ] {
            page.update(cx, |page, cx| page.activate(tab.into(), cx));
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(2200.)));
                cx.run_until_parked();
                let bounds = slots.map(|slot| cx.debug_bounds(slot).unwrap_or_else(|| panic!("{tab}: {slot}")));
                assert!(bounds.windows(2).all(|pair| pair[0].top() < pair[1].top()), "{tab}: fixed slot order");
                assert!(bounds[2].right() <= px(width), "{tab}: hero fits window");
                assert!(bounds[4].right() <= px(width), "{tab}: controls fit window");
                assert!(bounds[5].right() <= px(width), "{tab}: panels fit window");
            }
        }
    }
}

#[test]
fn shader_fields_keep_sorted_stop_selection_and_switches_follow_edits() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| ShaderPage::new(crate::theme::default_look(), cx));
    let (offset, animate) = cx.update(|_, cx| {
        let page = page.read(cx);
        (
            page.fields.iter().find(|(id, _)| *id == Parameter::Offset).unwrap().1.clone(),
            page.core_switches.iter().find(|(id, _)| *id == Action::Animate).unwrap().1.clone(),
        )
    });
    animate.update(cx, |switch, cx| {
        switch.set_data(true, cx);
        cx.emit(SwitchEvent::Change { on: true });
    });
    cx.run_until_parked();
    offset.update(cx, |field, cx| {
        field.set_value("0.82", cx);
        cx.emit(TextFieldEvent::Change { value: "0.82".into() });
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let page = page.read(cx);
        let settings = &page.settings[page.effect_index];
        assert_eq!(settings.selected, 3, "the moved stop remains selected after reordering");
        assert_eq!(settings.stops[3].offset, 0.82);
        assert!(!page.animated);
        assert!(!*animate.read(cx).data());
        assert_eq!(offset.read(cx).value(), "0.82", "typing is not reformatted before submit");
    });
    for invalid in ["NaN", "-1", "2"] {
        offset.update(cx, |field, cx| {
            field.set_value(invalid, cx);
            cx.emit(TextFieldEvent::Change { value: invalid.into() });
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(page.read(cx).value(Parameter::Offset), 0.82));
    }
    offset.update(cx, |_, cx| cx.emit(TextFieldEvent::Submit { value: "2".into() }));
    cx.update(|_, cx| assert_eq!(offset.read(cx).value(), "0.820"));
    let radius =
        cx.update(|_, cx| page.read(cx).sliders.iter().find(|(id, _)| *id == Parameter::RadiusX).unwrap().1.clone());
    radius.update(cx, |slider, cx| slider.set_thumb_position(slider.primary_thumb_id(), 0.5, cx));
    cx.run_until_parked();
    cx.update(|_, cx| {
        let page = page.read(cx);
        let field = &page.fields.iter().find(|(id, _)| *id == Parameter::RadiusX).unwrap().1;
        assert_eq!(field.read(cx).value(), format!("{:.3}", page.value(Parameter::RadiusX)));
    });
    let reset = cx.update(|_, cx| page.read(cx).buttons.iter().find(|(id, _)| *id == Action::Reset).unwrap().1.clone());
    reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(offset.read(cx).value(), "0.000"));
}

#[test]
fn instancing_choices_and_wgsl_fields_keep_their_source_workloads() {
    let mut app = TestAppContext::single();
    app.update(|cx| xgpui_luma::init(cx).unwrap());
    let (page, cx) = app.add_window_view(|_, cx| ShaderPage::new(crate::theme::default_look(), cx));
    page.update(cx, |page, cx| page.activate("Instancing".into(), cx));
    for (mode, expected) in [(0, 100), (1, 10000), (2, 3)] {
        let (example, count) = cx.update(|_, cx| {
            let page = page.read(cx);
            (
                page.choices.iter().find(|(id, _)| *id == Action::Mode).unwrap().1.clone(),
                page.choices.iter().find(|(id, _)| *id == Action::Count).unwrap().1.clone(),
            )
        });
        cx.update(|_, cx| {
            example.select(mode, cx);
            count.select(3, cx);
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(page.read(cx).instance_count(), expected));
    }
    page.update(cx, |page, cx| page.activate("WGSL Paints".into(), cx));
    let alpha =
        cx.update(|_, cx| page.read(cx).fields.iter().find(|(id, _)| *id == Parameter::ShaderAlpha).unwrap().1.clone());
    alpha.update(cx, |field, cx| {
        field.set_value("0.25", cx);
        cx.emit(TextFieldEvent::Change { value: "0.25".into() });
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert_eq!(page.read(cx).shader_alpha, 0.25);
        assert!(page.read(cx).wgsl().is_ok());
    });
    page.update(cx, |page, cx| {
        page.activate("Gradients".into(), cx);
        page.activate("WGSL Paints".into(), cx);
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(alpha.read(cx).value(), "0.250"));
}
