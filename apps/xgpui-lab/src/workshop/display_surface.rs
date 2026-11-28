//! SDK display decorations on the design surface page (issue #393): Card, Avatar, Callout and Tooltip (its
//! bubble drawn open by the SDK's own renderer). Each `.lss`
//! has no states, so the page offers its stylesheet and a single matrix cell, like Badge.
//!
//! Card, Avatar and Callout are the SDK's own elements (Avatar and Callout since #397), each drawn from its
//! `.lss` bound to the page's theme.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::controls::avatar::{Avatar, StyledAvatar};
use xgpui_luma::controls::callout::{Callout, StyledCallout};
use xgpui_luma::controls::card::{Card, StyledCard};
use xgpui_luma::controls::tooltip::{TooltipRenderModel, bubble, resolve_tooltip};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

fn binding(control: &'static str, theme: &Theme) -> StyleBinding {
    StyleBinding::new(control, ThemeSource::Fixed(theme.clone()))
}

/// A card with a header and body text, with ascenders and descenders.
fn card(theme: &Theme) -> AnyElement {
    div()
        .w(px(320.))
        .child(
            Card::new()
                .title("Shipping preferences")
                .description("Delivery days and updates")
                .styled(StyledCard::new(binding("card", theme)))
                .child("Pick a delivery day, and we'll keep you posted by email."),
        )
        .into_any_element()
}

/// SDK avatars with one- and two-letter and icon fallbacks.
fn avatars(theme: &Theme) -> AnyElement {
    let styled = || StyledAvatar::new(binding("avatar", theme));
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(Avatar::new("Q").styled(styled()))
        .child(Avatar::new("JG").styled(styled()))
        .child(Avatar::new("").icon(LucideIcon::User).styled(styled()))
        .into_any_element()
}

/// An SDK callout with a leading icon.
fn callout(theme: &Theme) -> AnyElement {
    div()
        .w(px(320.))
        .child(
            Callout::new("Shipping delays may apply to your region this week.")
                .icon(LucideIcon::Info)
                .styled(StyledCallout::new(binding("callout", theme))),
        )
        .into_any_element()
}

/// A tooltip bubble drawn open by the SDK's own renderer, with a shortcut after its text.
fn tooltip(theme: &Theme) -> AnyElement {
    bubble(&TooltipRenderModel::from_resolved(
        &resolve_tooltip(&binding("tooltip", theme)),
        "Copy styling".into(),
        Some("Ctrl+Shift+C".into()),
    ))
}

/// Draws a passive decoration from the theme last bound; decorations are elements, so the page needs a view.
struct DecorationHost {
    theme: Option<Theme>,
    draw: fn(&Theme) -> AnyElement,
    selector: &'static str,
}

impl Render for DecorationHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selector = self.selector;
        div().debug_selector(move || selector.into()).children(self.theme.as_ref().map(self.draw))
    }
}

/// A passive decoration page: no states, no input, no events.
pub(super) struct DecorationSurface {
    title: &'static str,
    host: Entity<DecorationHost>,
    draw: fn(&Theme) -> AnyElement,
}

impl DecorationSurface {
    fn make(
        title: &'static str,
        selector: &'static str,
        draw: fn(&Theme) -> AnyElement,
        cx: &mut Context<ControlSide>,
    ) -> Box<dyn SurfaceControl> {
        let host = cx.new(|_| DecorationHost { theme: None, draw, selector });
        Box::new(Self { title, host, draw })
    }

    pub(super) fn make_card(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make("Card", "workshop-sdk-card", card, cx)
    }

    pub(super) fn make_avatar(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make("Avatar", "workshop-sdk-avatar", avatars, cx)
    }

    pub(super) fn make_tooltip(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make("Tooltip", "workshop-sdk-tooltip", tooltip, cx)
    }

    pub(super) fn make_callout(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Self::make("Callout", "workshop-sdk-callout", callout, cx)
    }
}

impl SurfaceControl for DecorationSurface {
    fn title(&self) -> &'static str {
        self.title
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

    /// A decoration takes no input.
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A decoration emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.host, |_, _, _| {})
    }

    fn cell(&self, theme: &Theme, _id: SharedString, _states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        (self.draw)(theme)
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
    fn the_decoration_pages_draw_and_draw_a_matrix_cell() {
        for (id, selector) in [
            ("card", "workshop-sdk-card"),
            ("avatar", "workshop-sdk-avatar"),
            ("callout", "workshop-sdk-callout"),
            ("tooltip", "workshop-sdk-tooltip"),
        ] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            assert!(cx.update(|_, cx| side.read(cx).control_as::<DecorationSurface>().is_some()), "{id}");
            let drawn = cx.debug_bounds(selector).expect("drawn");
            assert!(drawn.size.width > px(0.) && drawn.size.height > px(0.), "{id} has size: {drawn:?}");

            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "{id}'s matrix draws");
        }
    }

    /// The Avatar and Callout pages draw the SDK's own elements, not stand-ins (#397).
    #[test]
    fn the_avatar_and_callout_pages_draw_the_sdk_elements() {
        let mut app = TestAppContext::single();
        let (_, cx) = open_surface(&mut app, "avatar");
        let row = cx.debug_bounds("workshop-sdk-avatar").expect("the avatars are drawn");
        let diameter = StyledAvatar::sdk().look().diameter;
        assert_eq!(row.size.height, px(diameter), "a row of SDK avatars, one diameter tall");

        let mut app = TestAppContext::single();
        let (_, cx) = open_surface(&mut app, "callout");
        assert!(cx.debug_bounds("callout-text").is_some(), "the SDK callout's text is drawn");
    }
}
