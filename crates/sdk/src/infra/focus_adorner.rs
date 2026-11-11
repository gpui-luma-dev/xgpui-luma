//! Focus paint never participates in layout. Reserve geometry from metrics alone,
//! then paint the same noninteractive overlay with a visible or transparent color.
//!
//! # Contract for templates and looks
//!
//! - Looks resolve face borders and focus colors independently. Transparent focus
//!   paint suppresses the visual indicator without changing geometry.
//! - Templates reserve `extent()` on every side in every interaction state, or
//!   consistently paint inset for dense controls. Never branch reservation on
//!   focus, enabled/invalid/selected state, optional paint, or color alpha.
//! - The target is the control face (or its indicator/track), not its label or
//!   popup content. The overlay has no input handlers and is out of normal flow.
//! - Templates retain their existing focus visibility policy. `visible` gates
//!   paint only. Face borders can still change color without changing thickness.
//!
//! Custom look overrides supply `focus_border` explicitly; no template probes a
//! hypothetical focused appearance or derives focus paint from a face border.

use gpui::{Div, Hsla, div, prelude::*, px, transparent_black};

use crate::theme::FocusMetricTokens;

/// Resolved geometry shared by control templates and look inspection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusAdorner {
    pub width: f32,
    /// Distance from the face's outer edge to the ring's inner edge.
    pub gap: f32,
}

impl FocusAdorner {
    pub fn new(metrics: FocusMetricTokens) -> Self {
        Self { width: metrics.width.max(0.0), gap: (metrics.gap + metrics.offset).max(0.0) }
    }

    /// Space reserved on each side, regardless of focus state or color alpha.
    pub fn extent(self) -> f32 {
        self.gap + self.width
    }

    /// Paint over a root which already reserves [`Self::extent`] on every side.
    /// Inset mode instead paints inside the face, preserving dense footprints.
    pub fn overlay(self, color: Hsla, visible: bool, face_radius: f32, inset: bool) -> Div {
        let radius = if inset {
            face_radius
        } else {
            face_radius + self.extent()
        };
        div()
            .absolute()
            .top_0()
            .right_0()
            .bottom_0()
            .left_0()
            .border(px(self.width))
            .border_color(if visible { color } else { transparent_black() })
            .rounded(px(radius.max(0.0)))
    }
}

/// Shared headless regression harness for SDK and look implementations.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub mod testing;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_uses_all_focus_metrics_and_clamps_negative_lengths() {
        let ring = FocusAdorner::new(FocusMetricTokens { gap: 1.5, width: 2., offset: 0.5 });
        assert_eq!(ring.extent(), 4.);
        let inset = FocusAdorner::new(FocusMetricTokens { gap: 1., width: -2., offset: -5. });
        assert_eq!(inset.extent(), 0.);
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn transparent_and_visible_paint_share_geometry_and_keep_face_border_independent() {
        use crate::controls::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
        use crate::controls::button_family::{default_button_family_theme, compose_button_family_look};
        use crate::theme::{ControlSize, StandardBoxScale};
        use std::sync::Arc;
        for compact in [false, true] {
            let theme = default_button_family_theme();
            let template = DefaultButtonTemplate::new(theme.clone());
            testing::assert_stable_layout(move |state, selected, window, cx| {
                let palette = theme.resolve(Default::default(), ControlSize::Md, state);
                let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
                let mut look = compose_button_family_look(&palette, Default::default(), &scale, 999.);
                // Deliberately vary both face presence and focus alpha independently.
                look.border = selected.then_some(gpui::red());
                look.focus_border = if state.focused {
                    gpui::blue()
                } else {
                    transparent_black()
                };
                let model = ButtonRenderModel::<()> {
                    state,
                    compact,
                    look: Some(Arc::new(move |_| look.clone())),
                    ..Default::default()
                };
                template.render(&model, window, cx).into_any_element()
            });
        }
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn default_input_and_popup_templates_have_stable_focus_geometry() {
        testing::assert_textfield_layout(crate::controls::textfield::default_textfield_theme());
        testing::assert_textarea_layout(crate::controls::textarea::default_textarea_theme());
        testing::assert_selector_layout(crate::controls::selector::default_selector_theme());
        testing::assert_popup_layout(crate::controls::popup_menu::default_popup_menu_theme());
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn overlay_paints_only_the_requested_focus_color() {
        use gpui::{Context, IntoElement, Render, TestAppContext, Window};
        struct Harness {
            visible: bool,
        }
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().relative().w(px(50.)).h(px(30.)).child(
                    FocusAdorner::new(FocusMetricTokens { gap: 1., width: 2., offset: 0. }).overlay(
                        gpui::red(),
                        self.visible,
                        4.,
                        false,
                    ),
                )
            }
        }
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, _| Harness { visible: false });
        for visible in [false, true, false] {
            view.update(cx, |view, cx| {
                view.visible = visible;
                cx.notify();
            });
            cx.run_until_parked();
            cx.update(|window, _| {
                assert_eq!(window.painted_quads().iter().any(|quad| quad.border_color == gpui::red()), visible);
            });
        }
    }
}
