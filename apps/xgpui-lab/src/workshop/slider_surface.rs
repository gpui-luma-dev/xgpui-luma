//! The SDK slider on the design surface page (issue #383). Its `.lss` (`slider`) reads hover, pressed and
//! disabled. Its focus ring is drawn outside the template, so `:focused` can't be pinned.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::slider::{
    self, Slider, SliderEvent, SliderInputStrategy, SliderLook, SliderRenderModel, SliderTemplate,
    SliderTemplateHandlers, SliderThumbPolicy, SliderThumbRole, SliderThumbValue, StyledSliderTheme,
    ThemedSliderTemplate, ThumbId, TrackPresentation, build_track_segments,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::InteractionState;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The `.lss` states the slider theme reads.
const STATES: [&str; 3] = ["hover", "pressed", "disabled"];

/// The live slider's starting value, and where each State matrix cell's thumb sits (0–1).
const VALUE: f32 = 50.;
const CELL_POSITION: f32 = 0.5;
/// A slider has no width of its own: the stage gives it one (scaled with the zoom), and so does each
/// matrix cell.
const STAGE_WIDTH: f32 = 240.;
const CELL_WIDTH: f32 = 200.;

pub(super) struct SliderSurface {
    slider: Slider,
}

impl SliderSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let slider = slider::new("workshop-sdk-slider").value(VALUE).spawn(cx);
        Box::new(Self { slider })
    }

    /// The SDK slider, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn slider(&self) -> Slider {
        self.slider.clone()
    }
}

/// The slider template for the `slider` control in `theme`.
fn template(theme: Theme) -> ThemedSliderTemplate {
    let theme = StyledSliderTheme::new(StyleBinding::new("slider", ThemeSource::Fixed(theme)));
    ThemedSliderTemplate::new(Arc::new(theme))
}

/// The picked states as the slider's interaction state.
fn pinned_state(states: &StateSet) -> InteractionState {
    InteractionState { focused: false, invalid: false, ..states.interaction() }
}

/// Draws the base template with the picked states (Design mode), or as the slider is.
struct PinnedSliderTemplate {
    base: ThemedSliderTemplate,
    pinned: Option<StateSet>,
}

impl SliderTemplate for PinnedSliderTemplate {
    fn resolve_look(&self, state: InteractionState) -> SliderLook {
        match &self.pinned {
            Some(states) => self.base.resolve_look(pinned_state(states)),
            None => self.base.resolve_look(state),
        }
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let Some(states) = &self.pinned else {
            return self.base.render(model, handlers, primary_thumb_id, window, cx);
        };
        let pinned = SliderRenderModel {
            id: model.id,
            strategy: model.strategy,
            orientation: model.orientation,
            presentation: model.presentation,
            range: model.range,
            step: model.step,
            thumbs: model.thumbs,
            track_segments: model.track_segments.clone(),
            reversed: model.reversed,
            wrapping: model.wrapping,
            enabled: !states.has("disabled"),
            corner_radius: model.corner_radius,
            thumb_radius: model.thumb_radius,
            thumb_policy: model.thumb_policy,
            active_thumb_id: model.active_thumb_id,
            state: pinned_state(states),
            domain_track: model.domain_track.clone(),
        };
        self.base.render(&pinned, handlers, primary_thumb_id, window, cx)
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(|_, _, _| {}),
        hover: Box::new(|_, _, _| {}),
        mouse_down: Box::new(|_, _, _| {}),
        mouse_move: Box::new(|_, _, _| {}),
        mouse_up: Box::new(|_, _, _| {}),
        mouse_up_out: Box::new(|_, _, _| {}),
        drag_move: Arc::new(|_, _, _| {}),
        thumb_mouse_down: Arc::new(|_, _, _, _| {}),
    }
}

impl SurfaceControl for SliderSurface {
    fn title(&self) -> &'static str {
        "Slider"
    }

    fn view(&self) -> AnyView {
        self.slider.clone().into()
    }

    /// A slider fills the width it is given and has none of its own.
    fn stage_width(&self) -> Option<f32> {
        Some(STAGE_WIDTH)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedSliderTemplate { base: template(theme), pinned: pinned.cloned() };
        self.slider.update(cx, |slider, cx| slider.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.slider.update(cx, |slider, cx| slider.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.slider, move |side: &mut ControlSide, _, event: &SliderEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, SliderEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                SliderEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                SliderEvent::DragStart { .. } => Some(("pressed", true)),
                SliderEvent::DragEnd { .. } | SliderEvent::Release { .. } => Some(("pressed", false)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let thumb = ThumbId::next();
        let thumbs =
            [SliderThumbValue { id: thumb, position: CELL_POSITION, preview: None, role: SliderThumbRole::Value }];
        let range = (0.0..100.0).into();
        let model = SliderRenderModel {
            id: &id,
            strategy: SliderInputStrategy::Horizontal,
            orientation: SliderInputStrategy::Horizontal.orientation(),
            presentation: TrackPresentation::Fill,
            range,
            step: 1.,
            thumbs: &thumbs,
            track_segments: build_track_segments(TrackPresentation::Fill, CELL_POSITION, &[], range),
            reversed: false,
            wrapping: false,
            enabled: !states.has("disabled"),
            corner_radius: None,
            thumb_radius: None,
            thumb_policy: SliderThumbPolicy::default(),
            active_thumb_id: None,
            state: pinned_state(states),
            domain_track: None,
        };
        div()
            .w(px(CELL_WIDTH))
            .child(template(theme.clone()).render(&model, inert_handlers(), thumb, window, cx))
            .into_any_element()
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
    fn the_slider_page_moves_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "slider");
        let side = cx.update(|_, cx| page.read(cx).side());
        // No width of its own: the stage gives it one.
        let control = cx.debug_bounds("workshop-surface-control").expect("the control");
        assert_eq!(control.size.width, px(STAGE_WIDTH), "the stage gives it a width");
        let slider = cx.update(|_, cx| side.read(cx).control_as::<SliderSurface>().expect("a slider").slider());

        cx.update(|window, cx| slider.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(slider.read(cx).value(), 100.));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Change"), "changes are logged: {text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("pressed"), "{pinned:?}");

        let (columns, _) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw sliders");
    }
}
