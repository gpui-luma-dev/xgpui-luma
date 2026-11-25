//! The SDK popover on the design surface page (issue #393). `popover.lss` holds only where a popover opens
//! (the offsets below its trigger and its edge margin); the SDK draws no panel of its own, so the app supplies
//! the content. The page's live control is an SDK popover button whose content is an SDK card, opened at
//! the start so the panel shows. The `.lss` has no states, so the matrix is a single cell.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::card::Card;
use xgpui_luma::controls::popover_button::{PopoverButton, PopoverButtonEvent, PopoverDismissPolicy, PopoverPlacement};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::stylesheet::PopoverGeometry;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The trigger's label and the panel's card, with ascenders and descenders.
const TRIGGER: &str = "Delivery options";

fn panel() -> AnyElement {
    div()
        .w(px(260.))
        .child(Card::new().title("Delivery options").child("Pick a day, and we'll keep you posted by email."))
        .into_any_element()
}

/// The popover button and its trigger.
struct PopoverHost {
    popover: Entity<PopoverButton<()>>,
}

impl Render for PopoverHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().debug_selector(|| "workshop-sdk-popover".into()).child(self.popover.clone())
    }
}

pub(super) struct PopoverSurface {
    host: Entity<PopoverHost>,
    trigger: Entity<Button>,
}

impl PopoverSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let trigger = Button::new("workshop-sdk-popover-trigger").content(|_, _| div().child(TRIGGER)).spawn(cx);
        let for_popover = trigger.clone();
        let popover = PopoverButton::new("workshop-sdk-popover")
            .typed(())
            .trigger(move |_, _| for_popover.clone().into_any_element())
            .content(|_, _, _| panel())
            .placement(PopoverPlacement::BelowStart)
            .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
            .trigger_on_pointer_down(false)
            .spawn(cx);
        let target = popover.downgrade();
        cx.subscribe(&trigger, move |_: &mut ControlSide, _, event: &xgpui_luma::controls::button::ButtonEvent, cx| {
            if matches!(event, xgpui_luma::controls::button::ButtonEvent::Click)
                && let Some(popover) = target.upgrade()
            {
                popover.update(cx, |popover, cx| popover.toggle_guarded(cx));
            }
        })
        .detach();
        // Open at the start, so the panel is on the stage.
        popover.update(cx, |popover, cx| popover.open(cx));
        let host = cx.new(|_| PopoverHost { popover });
        Box::new(Self { host, trigger })
    }

    /// The SDK popover button, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn popover(&self, cx: &App) -> Entity<PopoverButton<()>> {
        self.host.read(cx).popover.clone()
    }
}

impl SurfaceControl for PopoverSurface {
    fn title(&self) -> &'static str {
        "Popover"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    /// The popover's `.lss` is placement only: rebind where it opens.
    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        let geometry = PopoverGeometry::from_binding(&StyleBinding::new("popover", ThemeSource::Fixed(theme)));
        let popover = self.host.read(cx).popover.clone();
        popover.update(cx, |popover, cx| popover.set_geometry(geometry, cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.trigger.update(cx, |trigger, cx| trigger.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        let popover = self.host.read(cx).popover.clone();
        cx.subscribe(&popover, move |side: &mut ControlSide, _, event: &PopoverButtonEvent, cx| {
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    /// The panel the popover shows.
    fn cell(&self, _theme: &Theme, _id: SharedString, _states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        panel()
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
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_popover_page_opens_its_panel_and_draws_a_matrix_cell() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "popover");
        let side = cx.update(|_, cx| page.read(cx).side());
        let popover = cx.update(|_, cx| side.read(cx).control_as::<PopoverSurface>().expect("a popover").popover(cx));
        cx.update(|_, cx| assert!(popover.read(cx).is_open(cx), "opens at the start"));

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws the panel");
    }
}
