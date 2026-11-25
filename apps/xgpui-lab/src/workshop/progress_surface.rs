//! The SDK progress on the design surface page (issue #383). Its `.lss` (`progress`) has one state,
//! `:disabled`. Progress is passive, so the page's options drive it: a Bar · Ring shape and a value.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Div, SharedString, Stateful, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::progress::{
    self, CircularProgressTemplate, LinearProgressTemplate, Progress, ProgressDirection, ProgressRenderModel,
    ProgressTemplate, StyledProgressTheme,
};
use xgpui_luma::controls::slider::Slider;
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

const VALUE: f32 = 40.;
/// The shape option's items: id, label, and whether it is the ring.
const SHAPES: [(&str, &str, bool); 2] = [("bar", "Bar", false), ("ring", "Ring", true)];

/// What `bind` last drew with, so a shape change can redraw.
#[derive(Default)]
struct Binding {
    ring: Cell<bool>,
    last: RefCell<Option<(Theme, Option<bool>)>>,
}

impl Binding {
    /// Sets the progress's template for the current shape, theme and pinned enabled state.
    fn apply(&self, progress: &Progress, cx: &mut App) {
        let Some((theme, enabled)) = self.last.borrow().clone() else {
            return;
        };
        let template = PinnedProgressTemplate { base: template(theme, self.ring.get()), enabled };
        progress.update(cx, |progress, cx| progress.set_template(Arc::new(template), cx));
    }
}

pub(super) struct ProgressSurface {
    progress: Progress,
    binding: Rc<Binding>,
    /// Radix toggle group: Bar · Ring.
    shape_picker: ControlGroup<ControlGroupItem>,
    /// Radix slider for the value.
    value_slider: Slider,
}

impl ProgressSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let progress = progress::ProgressBuilder::new("workshop-sdk-progress").value(VALUE).spawn(cx);
        let binding = Rc::new(Binding::default());
        let shape_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-surface-progress-shape",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items(SHAPES.map(|(id, label, _)| ControlGroupItem::new(id).label(label)))
        .selected_ids(["bar"])
        .spawn(cx);
        let (target, shared) = (progress.clone(), binding.clone());
        cx.subscribe(&shape_picker, move |_: &mut ControlSide, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some((_, _, ring)) =
                    selected_ids.first().and_then(|id| SHAPES.iter().find(|(item, _, _)| *item == id.as_ref()))
            {
                shared.ring.set(*ring);
                shared.apply(&target, cx);
            }
        })
        .detach();
        let target = progress.clone();
        let value_slider = xgpui_luma_look_radix::Slider::new("workshop-surface-progress-value")
            .look(look.as_ref())
            .size(xgpui_luma_look_radix::SliderSize::One)
            .range(0.0..100.0)
            .value(VALUE)
            .on_value_changed(cx, move |_: &mut ControlSide, _, value, cx| {
                target.update(cx, |progress, cx| progress.set_value(value, cx));
            })
            .spawn(cx);
        Box::new(Self { progress, binding, shape_picker, value_slider })
    }

    /// The SDK progress, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn progress(&self) -> Progress {
        self.progress.clone()
    }

    /// The shape option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn shape_picker(&self) -> ControlGroup<ControlGroupItem> {
        self.shape_picker.clone()
    }
}

/// The progress template for the `progress` control in `theme`, as a bar or a ring.
fn template(theme: Theme, ring: bool) -> Arc<dyn ProgressTemplate> {
    let styled = Arc::new(StyledProgressTheme::new(StyleBinding::new("progress", ThemeSource::Fixed(theme))));
    if ring {
        Arc::new(CircularProgressTemplate::new(styled))
    } else {
        Arc::new(LinearProgressTemplate::new(styled))
    }
}

/// Draws the base template enabled as Design mode picks it, or as the progress is.
struct PinnedProgressTemplate {
    base: Arc<dyn ProgressTemplate>,
    enabled: Option<bool>,
}

impl ProgressTemplate for PinnedProgressTemplate {
    fn render(&self, model: &ProgressRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let Some(enabled) = self.enabled else {
            return self.base.render(model, window, cx);
        };
        let pinned = ProgressRenderModel { enabled, ..*model };
        self.base.render(&pinned, window, cx)
    }
}

impl SurfaceControl for ProgressSurface {
    fn title(&self) -> &'static str {
        "Progress"
    }

    fn view(&self) -> AnyView {
        self.progress.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        *self.binding.last.borrow_mut() = Some((theme, pinned.map(|states| !states.has("disabled"))));
        self.binding.apply(&self.progress, cx);
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.progress.update(cx, |progress, cx| progress.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    /// Progress emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.progress, |_, _, _| {})
    }

    /// The progress in the live shape, at its starting value.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let range = ControlRange::default();
        let model = ProgressRenderModel {
            id: &id,
            range,
            value: VALUE,
            percentage: range.percentage(VALUE),
            enabled: !states.has("disabled"),
            direction: ProgressDirection::default(),
            show_thumb: false,
            indeterminate: false,
            phase: 0.,
        };
        let ring = self.binding.ring.get();
        div()
            .when(!ring, |cell| cell.w(px(200.)))
            .child(template(theme.clone(), ring).render(&model, window, cx))
            .into_any_element()
    }

    fn options(&self) -> Option<AnyElement> {
        Some(
            div()
                .flex()
                .items_center()
                .gap(px(16.))
                .child(self.shape_picker.clone())
                .child(div().w(px(160.)).child(self.value_slider.clone()))
                .into_any_element(),
        )
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_progress_page_changes_shape_and_pins_disabled() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "progress");
        let side = cx.update(|_, cx| page.read(cx).side());
        let (progress, picker) = cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<ProgressSurface>().expect("a progress");
            (surface.progress(), surface.shape_picker())
        });
        assert!(cx.debug_bounds("workshop-surface-options").is_some(), "the options are shown");
        cx.update(|_, cx| assert_eq!(progress.read(cx).value(), VALUE));

        picker.update(cx, |_, cx| {
            cx.emit(ControlGroupEvent::Change {
                changed_id: "ring".into(),
                selected: true,
                selected_ids: vec!["ring".into()],
            })
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<ProgressSurface>().expect("a progress");
            assert!(surface.binding.ring.get(), "the ring shape is chosen");
        });

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert_eq!(pinned, StateSet::new(["disabled"]));

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw progress");
    }
}
