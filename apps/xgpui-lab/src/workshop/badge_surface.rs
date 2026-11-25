//! The SDK badge on the design surface page (issue #383). Its `.lss` (`badge`) has no states: a badge is a
//! passive label, so the page offers its stylesheet and a single matrix cell.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::badge::{Badge, StyledBadge};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The badges' labels, with ascenders and descenders.
const LABELS: [&str; 3] = ["Beta", "Upgrade", "Legacy"];

/// The badges in the bound theme's `badge` control.
fn badges(theme: Option<&Theme>) -> gpui::Div {
    div().flex().gap(px(8.)).children(LABELS.map(|label| {
        let badge = Badge::new(label);
        match theme {
            Some(theme) => {
                badge.styled(StyledBadge::new(StyleBinding::new("badge", ThemeSource::Fixed(theme.clone()))))
            }
            None => badge,
        }
    }))
}

/// Draws the badges; a badge is an element, not a view, so the page needs one to host them.
struct BadgeHost {
    theme: Option<Theme>,
}

impl Render for BadgeHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        badges(self.theme.as_ref()).debug_selector(|| "workshop-sdk-badges".into())
    }
}

pub(super) struct BadgeSurface {
    host: Entity<BadgeHost>,
}

impl BadgeSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { host: cx.new(|_| BadgeHost { theme: None }) })
    }
}

impl SurfaceControl for BadgeSurface {
    fn title(&self) -> &'static str {
        "Badge"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.theme = Some(theme);
            cx.notify();
        });
    }

    /// A badge takes no input.
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A badge emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.host, |_, _, _| {})
    }

    fn cell(&self, theme: &Theme, _id: SharedString, _states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        badges(Some(theme)).into_any_element()
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
    fn the_badge_page_draws_badges_and_a_matrix_cell() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "badge");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<BadgeSurface>().is_some()));
        assert!(cx.debug_bounds("workshop-sdk-badges").is_some(), "the badges are drawn");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws badges");
    }
}
