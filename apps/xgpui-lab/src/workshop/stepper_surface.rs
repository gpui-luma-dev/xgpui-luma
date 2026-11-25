//! The SDK stepper on the design surface page (issue #383). Its `.lss` (`stepper`) has one state,
//! `:disabled`; the finished, current and unstarted step colors are its own attributes. A stepper is
//! passive, so the page's options drive it: Next and Reset.

use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, Context, Div, Entity, SharedString, Stateful, Subscription, Window, div, prelude::*, px,
};
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::progress::ProgressDirection;
use xgpui_luma::controls::stepper::{
    StepState, Stepper, StepperBuilder, StepperContentRenderer, StepperLabelPlacement, StepperRenderModel,
    StepperTemplate, StyledStepperTheme, ThemedStepperTemplate,
};
use xgpui_luma::infra::icon::SelectionStatusIcons;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The steps' labels, with ascenders and descenders.
const LABELS: [&str; 3] = ["Shipping", "Payment", "Display"];

pub(super) struct StepperSurface {
    stepper: Stepper,
    next: Entity<Button>,
    reset: Entity<Button>,
}

impl StepperSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let stepper = StepperBuilder::new("workshop-sdk-stepper", LABELS.len()).labels(LABELS.to_vec()).spawn(cx);
        let target = stepper.clone();
        let next = xgpui_luma_look_radix::Button::new("workshop-surface-stepper-next")
            .look(look.as_ref())
            .soft()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .content(|_, _| div().child("Next"))
            .on_click(cx, move |_: &mut ControlSide, cx| {
                target.update(cx, |stepper, cx| {
                    if stepper.current_step() + 1 < stepper.step_count() {
                        stepper.set_current_step(stepper.current_step() + 1, cx);
                    } else {
                        stepper.complete(cx);
                    }
                });
            })
            .spawn(cx);
        let target = stepper.clone();
        let reset = xgpui_luma_look_radix::Button::new("workshop-surface-stepper-reset")
            .look(look.as_ref())
            .soft()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .content(|_, _| div().child("Reset"))
            .on_click(cx, move |_: &mut ControlSide, cx| {
                target.update(cx, |stepper, cx| stepper.set_current_step(0, cx));
            })
            .spawn(cx);
        Box::new(Self { stepper, next, reset })
    }

    /// The SDK stepper, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn stepper(&self) -> Stepper {
        self.stepper.clone()
    }

    /// The Next option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn next(&self) -> Entity<Button> {
        self.next.clone()
    }
}

/// The stepper template for the `stepper` control in `theme`.
fn template(theme: Theme) -> ThemedStepperTemplate {
    let binding = StyleBinding::new("stepper", ThemeSource::Fixed(theme));
    ThemedStepperTemplate::new(Arc::new(StyledStepperTheme::new(binding)))
}

/// Draws the base template enabled as Design mode picks it, or as the stepper is.
struct PinnedStepperTemplate {
    base: ThemedStepperTemplate,
    enabled: Option<bool>,
}

impl StepperTemplate for PinnedStepperTemplate {
    fn render(&self, model: &StepperRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let Some(enabled) = self.enabled else {
            return self.base.render(model, window, cx);
        };
        let pinned = StepperRenderModel {
            id: model.id,
            step_count: model.step_count,
            current_step: model.current_step,
            display_step: model.display_step,
            transition_progress: model.transition_progress,
            from_step: model.from_step,
            to_step: model.to_step,
            step_states: model.step_states,
            labels: model.labels,
            label_placement: model.label_placement,
            direction: model.direction,
            enabled,
            step_contents: model.step_contents,
            content_height: model.content_height,
            icons: model.icons,
        };
        self.base.render(&pinned, window, cx)
    }
}

impl SurfaceControl for StepperSurface {
    fn title(&self) -> &'static str {
        "Stepper"
    }

    fn view(&self) -> AnyView {
        self.stepper.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let enabled = pinned.map(|states| !states.has("disabled"));
        let template = PinnedStepperTemplate { base: template(theme), enabled };
        self.stepper.update(cx, |stepper, cx| stepper.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.stepper.update(cx, |stepper, cx| stepper.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    /// A stepper emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.stepper, |_, _, _| {})
    }

    /// The stepper on its second step, so a finished, the current and an unstarted step all show.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let labels: Vec<SharedString> = LABELS.map(SharedString::from).to_vec();
        let step_states = [StepState::Complete, StepState::InProgress, StepState::Incomplete];
        let contents: Vec<Option<StepperContentRenderer>> = vec![None; LABELS.len()];
        let icons = SelectionStatusIcons::default();
        let model = StepperRenderModel {
            id: &id,
            step_count: LABELS.len(),
            current_step: 1,
            display_step: 1.,
            transition_progress: 1.,
            from_step: 1.,
            to_step: 1,
            step_states: &step_states,
            labels: &labels,
            label_placement: StepperLabelPlacement::Below,
            direction: ProgressDirection::default(),
            enabled: !states.has("disabled"),
            step_contents: &contents,
            content_height: None,
            icons: &icons,
        };
        div().w(px(320.)).child(template(theme.clone()).render(&model, window, cx)).into_any_element()
    }

    fn options(&self) -> Option<AnyElement> {
        Some(div().flex().gap(px(8.)).child(self.next.clone()).child(self.reset.clone()).into_any_element())
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_stepper_page_advances_completes_and_pins_disabled() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "stepper");
        let side = cx.update(|_, cx| page.read(cx).side());
        let (stepper, next) = cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<StepperSurface>().expect("a stepper");
            (surface.stepper(), surface.next())
        });

        cx.update(|window, cx| next.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space space space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(stepper.read(cx).current_step(), 2);
            assert!(stepper.read(cx).is_complete());
        });

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert_eq!(pinned, StateSet::new(["disabled"]));

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw steppers");
    }
}
