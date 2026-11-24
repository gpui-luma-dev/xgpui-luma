//! The SDK scrollbar on the design surface page (issue #383). Its `.lss` (`scrollbar`) reads hover,
//! pressed and disabled.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, prelude::*};
use xgpui_luma::controls::scrollbar::{
    Scrollbar, ScrollbarBuilder, ScrollbarEvent, ScrollbarRenderModel, ScrollbarTemplate, ScrollbarTemplateHandlers,
    StyledScrollbarTheme, ThemedScrollbarTemplate,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, MetricTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The `.lss` states the scrollbar theme reads.
const STATES: [&str; 3] = ["hover", "pressed", "disabled"];

/// The live bar's length; each State matrix cell's bar is the same, its thumb a fifth of it and a third
/// of the way along.
const LENGTH: f32 = 240.;
const CELL_THUMB_FRACTION: f32 = 0.2;
const CELL_PERCENTAGE: f32 = 0.33;

pub(super) struct ScrollbarSurface {
    scrollbar: Entity<Scrollbar>,
}

impl ScrollbarSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let scrollbar = ScrollbarBuilder::new("workshop-sdk-scrollbar")
            .viewport(0., 1000., 0., 200.)
            .length(LENGTH)
            .spawn(cx);
        Box::new(Self { scrollbar })
    }

    /// The SDK scrollbar, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn scrollbar(&self) -> Entity<Scrollbar> {
        self.scrollbar.clone()
    }
}

/// The scrollbar template for the `scrollbar` control in `theme`.
fn template(theme: Theme) -> ThemedScrollbarTemplate {
    let binding = StyleBinding::new("scrollbar", ThemeSource::Fixed(theme));
    ThemedScrollbarTemplate::new(Arc::new(StyledScrollbarTheme::new(binding, MetricTokens::default())))
}

/// The picked states as the scrollbar's interaction state.
fn pinned_state(states: &StateSet) -> InteractionState {
    InteractionState { focused: false, invalid: false, ..states.interaction() }
}

/// Draws the base template with the picked states (Design mode), or as the scrollbar is.
struct PinnedScrollbarTemplate {
    base: ThemedScrollbarTemplate,
    pinned: Option<StateSet>,
}

impl ScrollbarTemplate for PinnedScrollbarTemplate {
    fn render(
        &self,
        model: &ScrollbarRenderModel<'_>,
        handlers: ScrollbarTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let Some(states) = &self.pinned else {
            return self.base.render(model, handlers, window, cx);
        };
        let pinned = ScrollbarRenderModel {
            enabled: !states.has("disabled"),
            state: pinned_state(states),
            id: model.id,
            ..*model
        };
        self.base.render(&pinned, handlers, window, cx)
    }

    fn min_thumb_length(
        &self,
        orientation: xgpui_luma::controls::scrollbar::ScrollbarOrientation,
        style: xgpui_luma::controls::scrollbar::ScrollbarStyle,
    ) -> f32 {
        self.base.min_thumb_length(orientation, style)
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> ScrollbarTemplateHandlers {
    ScrollbarTemplateHandlers {
        track_bounds: Box::new(|_, _, _| {}),
        thumb_bounds: Box::new(|_, _, _| {}),
        hover: Box::new(|_, _, _| {}),
        mouse_down: Box::new(|_, _, _| {}),
        mouse_up: Box::new(|_, _, _| {}),
        mouse_up_out: Box::new(|_, _, _| {}),
        drag_move: Box::new(|_, _, _| {}),
        scroll_wheel: Box::new(|_, _, _| {}),
    }
}

impl SurfaceControl for ScrollbarSurface {
    fn title(&self) -> &'static str {
        "Scrollbar"
    }

    fn view(&self) -> AnyView {
        self.scrollbar.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedScrollbarTemplate { base: template(theme), pinned: pinned.cloned() };
        self.scrollbar.update(cx, |scrollbar, cx| scrollbar.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.scrollbar.update(cx, |scrollbar, cx| scrollbar.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.scrollbar, move |side: &mut ControlSide, _, event: &ScrollbarEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, ScrollbarEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                ScrollbarEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                ScrollbarEvent::DragStart => Some(("pressed", true)),
                ScrollbarEvent::DragEnd { .. } => Some(("pressed", false)),
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
        let live = self.scrollbar.read(cx);
        let model = ScrollbarRenderModel {
            id: &id,
            orientation: live.orientation(),
            range: live.range(),
            step: live.step(),
            page_step: live.page_step(),
            value: live.value(),
            percentage: CELL_PERCENTAGE,
            thumb_fraction: CELL_THUMB_FRACTION,
            length: Some(LENGTH),
            style: live.style(),
            enabled: !states.has("disabled"),
            state: pinned_state(states),
        };
        template(theme.clone()).render(&model, inert_handlers(), window, cx).into_any_element()
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
    fn the_scrollbar_page_scrolls_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "scrollbar");
        let side = cx.update(|_, cx| page.read(cx).side());
        let bar = cx.update(|_, cx| side.read(cx).control_as::<ScrollbarSurface>().expect("a scrollbar").scrollbar());

        cx.update(|window, cx| bar.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(bar.read(cx).value(), 800.));

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw scrollbars");
    }
}
