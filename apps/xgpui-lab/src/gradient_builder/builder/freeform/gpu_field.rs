//! Adapter between editor state and the app's promotable GPU field primitive.
#[cfg(any(target_os = "macos", all(test, feature = "test-support")))]
use super::*;
use crate::gradient_builder::color_field::ColorField;
#[cfg(target_os = "macos")]
use crate::gradient_builder::color_field::Output;
use gpui::{AnyElement, IntoElement, div, prelude::*};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(in super::super) enum Renderer {
    #[default]
    Cpu,
    Wgpu,
    Metal,
}
impl Renderer {
    pub(in super::super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "cpu" => Some(Self::Cpu),
            "wgpu" => Some(Self::Wgpu),
            "metal" if cfg!(target_os = "macos") => Some(Self::Metal),
            _ => None,
        }
    }
    pub(in super::super) fn is_gpu(self) -> bool {
        self != Self::Cpu
    }
}

pub(in super::super) fn renderer_items() -> Vec<xgpui_luma::controls::selector::SelectorItem> {
    use xgpui_luma::controls::selector::SelectorItem;
    vec![
        SelectorItem::new("cpu").label("CPU"),
        SelectorItem::new("wgpu").label("wgpu interop"),
        SelectorItem::new("metal").label("Metal (macOS)").enabled(cfg!(target_os = "macos")),
    ]
}
#[derive(Default)]
pub(in super::super) struct Cache {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    portable: Option<gpui::Entity<super::interop::FieldView>>,
    #[cfg(target_os = "macos")]
    pipeline: Arc<crate::gradient_builder::color_field::PipelineCache>,
    #[cfg(target_os = "macos")]
    cached: Option<(ColorField, Output, Arc<crate::gradient_builder::color_field::Paint>)>,
}
impl Cache {
    pub(in super::super) fn retry(&mut self, cx: &mut gpui::Context<super::super::GradientBuilder>) {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        if self.portable.as_ref().is_some_and(|view| view.read(cx).failed()) {
            self.portable = None;
        }
        let _ = cx;
    }
    pub(in super::super) fn element(
        &mut self,
        field: ColorField,
        wide: bool,
        renderer: Renderer,
        cx: &mut gpui::Context<super::super::GradientBuilder>,
    ) -> AnyElement {
        if renderer == Renderer::Wgpu {
            #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
            {
                let view = self
                    .portable
                    .get_or_insert_with(|| cx.new(|_| super::interop::FieldView::new(field.clone(), wide)));
                view.update(cx, |view, cx| view.set_input(field, wide, cx));
                return view.clone().into_any_element();
            }
        }
        let _ = cx;
        #[cfg(target_os = "macos")]
        {
            use gpui::{Corners, CustomGeometry, canvas};
            let output = if wide { Output::DisplayP3 } else { Output::Srgb };
            if self.cached.as_ref().is_none_or(|(old, mode, _)| old != &field || *mode != output) {
                let paint = match crate::gradient_builder::color_field::Paint::new(
                    field.clone(),
                    output,
                    self.pipeline.clone(),
                ) {
                    Ok(paint) => Arc::new(paint),
                    Err(error) => return div().p_3().child(error.to_string()).into_any_element(),
                };
                self.cached = Some((field, output, paint));
            }
            let paint = self.cached.as_ref().unwrap().2.clone();
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    window.paint_custom(
                        CustomGeometry::RoundedRect { bounds, corner_radii: Corners::default() },
                        paint.clone(),
                    );
                },
            )
            .size_full()
            .into_any_element()
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (field, wide);
            div().p_3().child(crate::l10n::ui("Native Metal is available only on macOS.")).into_any_element()
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    #[test]
    fn renderer_and_mixing_switches_keep_the_same_edited_field() {
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (view, cx) = app.add_window_view(|_, cx| GradientBuilder::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.handle_top_tabs_event(
                &TabsEvent::Activate { tab_id: "freeform".into(), label: "Freeform".into() },
                cx,
            );
            view.point_transforms[0] = [75., 0.4];
            view.point_spreads[0] = 65.;
        });
        let (renderer, mixing) =
            view.update(cx, |view, _| (view.freeform_renderer_selector.clone(), view.freeform_mixing_selector.clone()));
        let original = view.update(cx, |view, _| view.points.clone());
        for method in ["normalized", "ordered"] {
            mixing.update(cx, |_, cx| cx.emit(SelectorEvent::Change { item_id: method.into(), label: method.into() }));
            cx.run_until_parked();
            for id in ["wgpu", "metal", "cpu"] {
                if Renderer::from_id(id).is_none() {
                    continue;
                }
                renderer.update(cx, |_, cx| cx.emit(SelectorEvent::Change { item_id: id.into(), label: id.into() }));
                cx.run_until_parked();
                view.update(cx, |view, cx| {
                    assert_eq!(view.points, original);
                    assert_eq!(view.point_transforms[0], [75., 0.4]);
                    assert_eq!(view.point_spreads[0], 65.);
                    assert_eq!(view.uses_render_preview(cx), id == "cpu");
                    assert_eq!(
                        view.freeform_mixing == crate::gradient_builder::color_field::Mixing::Normalized,
                        method == "normalized"
                    );
                });
            }
        }
    }
}

#[cfg(test)]
mod platform_tests {
    use super::*;
    #[test]
    fn metal_stays_visible_but_is_disabled_off_macos() {
        let items = renderer_items();
        assert_eq!(items.len(), 3);
        let metal = items.iter().find(|item| item.id() == "metal").unwrap();
        assert_eq!(metal.is_enabled(), cfg!(target_os = "macos"));
        assert_eq!(Renderer::from_id("metal").is_some(), metal.is_enabled());
        assert_eq!(Renderer::from_id("wgpu"), Some(Renderer::Wgpu));
    }
}
