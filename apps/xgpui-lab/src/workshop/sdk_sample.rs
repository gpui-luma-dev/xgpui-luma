//! Scope the legacy color-control theme to SDK defaults without changing Lab chrome.
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    Window,
};
use xgpui_luma::theme::ThemeTokens;
use xgpui_luma_color::style::{ColorControlTheme, active_color_control_theme, set_active_color_control_theme};

struct RestoreTheme(ColorControlTheme);
impl Drop for RestoreTheme {
    fn drop(&mut self) {
        set_active_color_control_theme(self.0);
    }
}

pub(super) fn with_sdk_defaults<T>(build: impl FnOnce() -> T) -> T {
    let restore = RestoreTheme(active_color_control_theme());
    let tokens = ThemeTokens::default();
    let background = tokens.palette.surface.panel.background;
    set_active_color_control_theme(ColorControlTheme::new(
        tokens.palette.border.default,
        background,
        background.l < 0.5,
    ));
    let result = build();
    drop(restore);
    result
}

pub(super) fn isolated(child: impl IntoElement) -> SdkSample {
    SdkSample { child: child.into_any_element() }
}

pub(super) struct SdkSample {
    child: AnyElement,
}
impl IntoElement for SdkSample {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for SdkSample {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (with_sdk_defaults(|| self.child.request_layout(window, cx)), ())
    }
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        with_sdk_defaults(|| {
            self.child.prepaint(window, cx);
        });
    }
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        with_sdk_defaults(|| self.child.paint(window, cx));
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{AppContext, Context, Entity, Render, TestAppContext, Styled, canvas, div, ParentElement};
    use std::sync::{Arc, Mutex};

    #[test]
    fn sdk_construction_restores_ambient_theme_even_when_unwinding() {
        let _restore = RestoreTheme(active_color_control_theme());
        let ambient = ColorControlTheme::new(gpui::rgb(0xff00ff).into(), gpui::rgb(0x00ffff).into(), false);
        set_active_color_control_theme(ambient);
        let tokens = ThemeTokens::default();
        let context = with_sdk_defaults(|| {
            xgpui_luma_color::color_arc::ColorArcBuilder::hue("isolated-arc", 90., 0.5, 0.5).track_context()
        });
        assert_eq!(context.border_color, tokens.palette.border.default);
        assert_eq!(active_color_control_theme().border, ambient.border);
        let result = std::panic::catch_unwind(|| with_sdk_defaults(|| panic!("test scoped restoration")));
        assert!(result.is_err());
        assert_eq!(active_color_control_theme().border, ambient.border);
        assert_eq!(active_color_control_theme().background, ambient.background);
    }

    type Observations = Arc<Mutex<Vec<(&'static str, gpui::Hsla, gpui::Hsla)>>>;
    fn observe(observations: &Observations, phase: &'static str) {
        let theme = active_color_control_theme();
        observations.lock().unwrap().push((phase, theme.border, theme.background));
    }
    struct Probe {
        observations: Observations,
    }
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            observe(&self.observations, "layout");
            let prepaint = self.observations.clone();
            let paint = prepaint.clone();
            canvas(move |_, _, _| observe(&prepaint, "prepaint"), move |_, _, _, _| observe(&paint, "paint"))
                .size_full()
        }
    }
    struct Host {
        probe: Entity<Probe>,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(isolated(self.probe.clone()))
        }
    }
    #[test]
    fn sdk_rendering_uses_default_theme_in_every_phase_without_leaking_to_lab() {
        let _restore = RestoreTheme(active_color_control_theme());
        let ambient = ColorControlTheme::new(gpui::rgb(0xff00ff).into(), gpui::rgb(0x00ffff).into(), false);
        set_active_color_control_theme(ambient);
        let observations = Observations::default();
        let target = observations.clone();
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| Host { probe: cx.new(|_| Probe { observations: target }) });
        cx.run_until_parked();
        let tokens = ThemeTokens::default();
        let observations = observations.lock().unwrap();
        for phase in ["layout", "prepaint", "paint"] {
            assert!(observations.iter().any(|entry| entry.0 == phase), "missing {phase}");
        }
        for (_, border, background) in observations.iter() {
            assert_eq!(*border, tokens.palette.border.default);
            assert_eq!(*background, tokens.palette.surface.panel.background);
        }
        assert_eq!(active_color_control_theme().border, ambient.border);
        assert_eq!(active_color_control_theme().background, ambient.background);
    }
}
