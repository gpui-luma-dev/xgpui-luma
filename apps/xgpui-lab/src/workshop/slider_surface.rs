//! The SDK slider on the design surface page (issue #383). Its `.lss` (`slider`) reads hover, pressed,
//! focused and disabled. The focus ring itself is drawn outside the template, so pinning `:focused` shows
//! only what the `.lss` styles for it. The shape option draws it with any of the SDK's three templates:
//! linear, dial and ring, each with the track's outline.

use std::cell::{Cell, RefCell};
use std::f32::consts::{FRAC_PI_2, PI};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::slider::{
    self, Slider, SliderEvent, SliderInputStrategy, SliderRenderModel, SliderTemplate, SliderTemplateHandlers,
    SliderThumbPolicy, SliderThumbRole, SliderThumbValue, StyledSliderTheme, ThemedAngularDialTemplate,
    ThemedCircularRingTemplate, ThemedSliderTemplate, ThumbId, TrackPresentation, build_track_segments,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::InteractionState;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The `.lss` states the slider theme reads.
const STATES: [&str; 4] = ["hover", "pressed", "focused", "disabled"];

/// The live slider's starting value, and where each State matrix cell's thumb sits (0–1).
const VALUE: f32 = 50.;
const CELL_POSITION: f32 = 0.5;
/// A slider has no width of its own: the stage gives it one (scaled with the zoom), and so does each
/// matrix cell.
const STAGE_WIDTH: f32 = 240.;
const CELL_WIDTH: f32 = 200.;

/// The SDK's three slider templates, chosen with the page's shape option.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    Linear,
    /// A 270° arc open at the bottom.
    Dial,
    /// A full circle starting at the top.
    Ring,
}

impl Shape {
    const ALL: [Shape; 3] = [Shape::Linear, Shape::Dial, Shape::Ring];

    fn id(self) -> &'static str {
        match self {
            Shape::Linear => "linear",
            Shape::Dial => "dial",
            Shape::Ring => "ring",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Shape::Linear => "Linear",
            Shape::Dial => "Dial",
            Shape::Ring => "Ring",
        }
    }

    /// How pointer input maps to a value: along the track, or around the arc.
    fn strategy(self) -> SliderInputStrategy {
        match self {
            Shape::Linear => SliderInputStrategy::Horizontal,
            Shape::Dial => SliderInputStrategy::Angular { min_angle: 0.75 * PI, max_angle: 2.25 * PI },
            Shape::Ring => SliderInputStrategy::Angular { min_angle: -FRAC_PI_2, max_angle: 1.5 * PI },
        }
    }

    /// The template for the `slider` control in `theme`.
    fn template(self, theme: Theme) -> Arc<dyn SliderTemplate> {
        let theme = Arc::new(StyledSliderTheme::new(StyleBinding::new("slider", ThemeSource::Fixed(theme))));
        match self {
            Shape::Linear => Arc::new(ThemedSliderTemplate::new(theme)),
            Shape::Dial => Arc::new(ThemedAngularDialTemplate::new(theme)),
            Shape::Ring => Arc::new(ThemedCircularRingTemplate::new(theme)),
        }
    }
}

/// The shape and what `bind` last drew with, so a shape change can redraw.
struct Binding {
    shape: Cell<Shape>,
    last: RefCell<Option<(Theme, Option<StateSet>)>>,
}

impl Binding {
    /// Give the slider the current shape's input and template, in the theme and pinned states last bound.
    fn apply(&self, slider: &Slider, cx: &mut App) {
        let shape = self.shape.get();
        slider.update(cx, |slider, cx| slider.set_strategy(shape.strategy(), cx));
        let Some((theme, pinned)) = self.last.borrow().clone() else {
            return;
        };
        let template = PinnedSliderTemplate { base: shape.template(theme), pinned };
        slider.update(cx, |slider, cx| slider.set_template(Arc::new(template), cx));
    }
}

pub(super) struct SliderSurface {
    slider: Slider,
    binding: Rc<Binding>,
    /// Radix toggle group: Linear · Dial · Ring.
    shape_picker: ControlGroup<ControlGroupItem>,
}

impl SliderSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let slider = slider::new("workshop-sdk-slider").value(VALUE).spawn(cx);
        let binding = Rc::new(Binding { shape: Cell::new(Shape::Linear), last: RefCell::default() });
        let shape_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-surface-slider-shape",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items(Shape::ALL.map(|shape| ControlGroupItem::new(shape.id()).label(shape.label())))
        .selected_ids([Shape::Linear.id()])
        .spawn(cx);
        let (target, shared) = (slider.clone(), binding.clone());
        cx.subscribe(&shape_picker, move |_: &mut ControlSide, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some(shape) =
                    selected_ids.first().and_then(|id| Shape::ALL.into_iter().find(|shape| shape.id() == id.as_ref()))
            {
                shared.shape.set(shape);
                shared.apply(&target, cx);
                // The stage gives a linear slider its width; a dial or ring sizes itself.
                cx.notify();
            }
        })
        .detach();
        Box::new(Self { slider, binding, shape_picker })
    }

    /// The SDK slider, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn slider(&self) -> Slider {
        self.slider.clone()
    }

    /// The shape option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn shape_picker(&self) -> ControlGroup<ControlGroupItem> {
        self.shape_picker.clone()
    }
}

/// The picked states as the slider's interaction state.
fn pinned_state(states: &StateSet) -> InteractionState {
    InteractionState { invalid: false, ..states.interaction() }
}

/// Draws the base template with the picked states (Design mode), or as the slider is.
struct PinnedSliderTemplate {
    base: Arc<dyn SliderTemplate>,
    pinned: Option<StateSet>,
}

impl SliderTemplate for PinnedSliderTemplate {
    fn resolve_style(&self, state: InteractionState) -> Arc<xgpui_luma::style::lss::Resolved> {
        match &self.pinned {
            Some(states) => self.base.resolve_style(pinned_state(states)),
            None => self.base.resolve_style(state),
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

    /// A linear slider fills the width it is given and has none of its own; a dial or ring sizes itself.
    fn stage_width(&self) -> Option<f32> {
        (self.binding.shape.get() == Shape::Linear).then_some(STAGE_WIDTH)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        *self.binding.last.borrow_mut() = Some((theme, pinned.cloned()));
        self.binding.apply(&self.slider, cx);
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
        let shape = self.binding.shape.get();
        let model = SliderRenderModel {
            id: &id,
            strategy: shape.strategy(),
            orientation: shape.strategy().orientation(),
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
            .when(shape == Shape::Linear, |cell| cell.w(px(CELL_WIDTH)))
            .child(shape.template(theme.clone()).render(&model, inert_handlers(), thumb, window, cx))
            .into_any_element()
    }

    fn options(&self) -> Option<AnyElement> {
        Some(self.shape_picker.clone().into_any_element())
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
    use xgpui_luma::controls::slider::SliderTheme;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    /// The shape option switches the live slider between the linear, dial and ring templates and their
    /// input, and the SDK theme outlines the track they draw (#397).
    #[test]
    fn the_shape_option_switches_between_linear_dial_and_ring() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "slider");
        let side = cx.update(|_, cx| page.read(cx).side());
        let (slider, picker) = cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<SliderSurface>().expect("a slider");
            (surface.slider(), surface.shape_picker())
        });
        assert!(cx.debug_bounds("workshop-surface-options").is_some(), "the shape option is shown");
        let choose = |shape: Shape, cx: &mut gpui::VisualTestContext| {
            picker.update(cx, |_, cx| {
                cx.emit(ControlGroupEvent::Change {
                    changed_id: shape.id().into(),
                    selected: true,
                    selected_ids: vec![shape.id().into()],
                })
            });
            cx.run_until_parked();
        };

        for shape in [Shape::Dial, Shape::Ring] {
            choose(shape, cx);
            cx.update(|_, cx| assert_eq!(slider.read(cx).strategy(), shape.strategy(), "{shape:?} input"));
            let drawn = cx.debug_bounds("workshop-surface-control").expect("the control").size;
            assert!(drawn.width > px(0.) && drawn.width == drawn.height, "{shape:?} draws a square: {drawn:?}");
            assert_ne!(drawn.width, px(STAGE_WIDTH), "{shape:?} sizes itself");
        }
        // Input follows the arc: a press at the dial's right-middle (angle 0, 5/6 of its 270° sweep) sets ~83.
        choose(Shape::Dial, cx);
        let dial = cx.debug_bounds("workshop-surface-control").expect("the dial");
        let right_middle = gpui::point(dial.right() - px(4.), dial.center().y);
        cx.simulate_mouse_down(right_middle, gpui::MouseButton::Left, Default::default());
        cx.simulate_mouse_up(right_middle, gpui::MouseButton::Left, Default::default());
        cx.run_until_parked();
        let value = cx.update(|_, cx| slider.read(cx).value());
        assert!((value - 83.).abs() <= 2., "pressing on the arc sets the value: {value}");
        choose(Shape::Linear, cx);
        cx.update(|_, cx| assert_eq!(slider.read(cx).strategy(), SliderInputStrategy::Horizontal));
        let control = cx.debug_bounds("workshop-surface-control").expect("the control");
        assert_eq!(control.size.width, px(STAGE_WIDTH), "linear takes the stage's width again");

        // The track the templates outline has a border in the SDK theme.
        let look = StyledSliderTheme::sdk().resolve(InteractionState::default());
        assert!(look.track_border_width > 0.0, "the SDK slider track has an outline");
    }

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
