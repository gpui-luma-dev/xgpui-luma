//! The SDK popover's panel on the design surface page (issues #393, #397), drawn on the stage with no trigger,
//! like the Tooltip page's bubble. `popover.lss` holds where a popover opens (offsets and edge margin,
//! behavior the stage can't show) and its `panel` part, drawn while `show_panel` is on. A look turns the panel
//! on; the page always draws it, so the part is what the stage shows. The **Panel** option opens and closes it
//! with the popover's own motion (`PopoverMotion`'s presence), and the stage keeps the open panel's size. The
//! `.lss` has no states, so the matrix is a single cell.

use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, Context, Entity, Pixels, SharedString, Size, Subscription, Window, div, point,
    prelude::*, px,
};
use xgpui_luma::controls::button::HasPresenter;
use xgpui_luma::controls::checkbox::Checkbox;
use xgpui_luma::controls::popover_button::{PopoverMotion, resolve_popover_panel, show_popover_panel, wrap_popover_panel};
use xgpui_luma::motion::overlay_presence::OverlayPresence;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The content's heading and body, with ascenders and descenders.
const HEADING: &str = "Delivery options";
const BODY: &str = "Pick a day, and we'll keep you posted by email.";

/// The popover panel `theme` draws, turned on, around plain text.
fn panel(theme: &Theme) -> AnyElement {
    let binding = show_popover_panel(StyleBinding::new("popover", ThemeSource::Fixed(theme.clone())), true);
    let content = div().w(px(236.)).flex().flex_col().gap(px(4.)).child(HEADING).child(BODY);
    wrap_popover_panel(&resolve_popover_panel(&binding), content)
}

/// Draws the panel from the theme last bound, opening and closing with the popover's motion.
struct PanelHost {
    theme: Option<Theme>,
    presence: OverlayPresence,
    /// The open panel's size, kept while it is closed so the stage doesn't move.
    size: Option<Size<Pixels>>,
}

impl Render for PanelHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_animating = self.presence.is_animating();
        let is_animating = self.presence.sync();
        self.presence.schedule_frame(window, cx);
        if was_animating || is_animating {
            cx.notify();
        }
        let entity = cx.entity();
        let mut stage = div().debug_selector(|| "workshop-sdk-popover".into());
        if let Some(size) = self.size {
            stage = stage.w(size.width).h(size.height);
        }
        let painted = self.presence.should_paint().then(|| self.theme.as_ref().map(panel)).flatten();
        let Some(panel) = painted else {
            return stage;
        };
        let offset = self.presence.adjust_offset(point(px(0.), px(0.)), self.size.unwrap_or_default());
        stage
            .on_children_prepainted(move |bounds, _, cx| {
                if let Some(bounds) = bounds.first() {
                    entity.update(cx, |host, _| {
                        if host.size.is_none() && host.presence.is_logical_open() && !host.presence.is_animating() {
                            host.size = Some(bounds.size);
                        }
                    });
                }
            })
            .child(div().relative().left(offset.x).top(offset.y).opacity(self.presence.opacity()).child(panel))
    }
}

pub(super) struct PopoverSurface {
    host: Entity<PanelHost>,
    /// Radix checkbox: open or close the panel.
    panel_option: Checkbox,
}

impl PopoverSurface {
    pub(super) fn make(look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let presence = OverlayPresence::with_motion(true, true, PopoverMotion::default().presence);
        let host = cx.new(|_| PanelHost { theme: None, presence, size: None });
        let target = host.clone();
        let panel_option = xgpui_luma_look_radix::Checkbox::new("workshop-surface-popover-panel")
            .look(look.as_ref())
            .checked(true)
            .content(|_, _| div().child("Panel").into_any_element())
            .on_value_changed(cx, move |_: &mut ControlSide, checked, cx| {
                target.update(cx, |host, cx| {
                    host.presence.set_open(checked);
                    cx.notify();
                });
            })
            .spawn(cx);
        Box::new(Self { host, panel_option })
    }

    /// The Panel option, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn panel_option(&self) -> Checkbox {
        self.panel_option.clone()
    }

    /// The stage, for tests.
    #[cfg(all(test, feature = "test-support"))]
    fn host(&self) -> Entity<PanelHost> {
        self.host.clone()
    }
}

impl SurfaceControl for PopoverSurface {
    fn title(&self) -> &'static str {
        "Popover"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, _pinned: Option<&StateSet>, cx: &mut App) {
        self.host.update(cx, |host, cx| {
            host.theme = Some(theme);
            // A new theme may size the panel differently.
            host.size = None;
            cx.notify();
        });
    }

    /// The panel is drawn, not triggered, so there is no input to turn off.
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A drawn panel emits no events.
    fn subscribe(&self, cx: &mut Context<ControlSide>, _on_event: OnSurfaceEvent) -> Subscription {
        cx.observe(&self.host, |_, _, _| {})
    }

    /// The open panel.
    fn cell(&self, theme: &Theme, _id: SharedString, _states: &StateSet, _: &mut Window, _: &mut App) -> AnyElement {
        panel(theme)
    }

    fn options(&self) -> Option<AnyElement> {
        Some(self.panel_option.clone().into_any_element())
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::TestAppContext;
    use xgpui_luma::controls::checkbox::CheckboxEvent;

    use super::*;
    use crate::workshop::tests_support::open_surface;

    /// The page draws the SDK popover panel open on the stage, with no trigger, and a matrix cell (#397).
    #[test]
    fn the_popover_page_draws_the_open_panel_and_a_matrix_cell() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "popover");
        let panel = cx.debug_bounds("popover-panel").expect("the panel is open on the stage");
        let resolved = resolve_popover_panel(&StyleBinding::sdk("popover"));
        let look = {
            use xgpui_luma::style::PartRead;
            let panel = resolved.part(xgpui_luma::style::lss::PartId::new("panel"));
            (panel.number("padding"), panel.number("border_width"))
        };
        assert_eq!(panel.size.width, px(236. + 2. * look.0 + 2. * look.1), "text, padding, border");
        assert!(cx.debug_bounds("workshop-sdk-popover-trigger").is_none(), "no trigger");
        let side = cx.update(|_, cx| page.read(cx).side());
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "the matrix draws the panel");
    }

    /// The Panel option closes and reopens the panel, and the stage keeps the open panel's size (#397).
    #[test]
    fn the_panel_option_closes_and_reopens_the_panel_in_place() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "popover");
        let side = cx.update(|_, cx| page.read(cx).side());
        let (option, host) = cx.update(|_, cx| {
            let surface = side.read(cx).control_as::<PopoverSurface>().expect("a popover");
            (surface.panel_option(), surface.host())
        });
        host.update(cx, |host, _| host.presence.set_motion_enabled(false));
        cx.run_until_parked();
        let open_stage = cx.debug_bounds("workshop-sdk-popover").expect("the stage");

        option.update(cx, |_, cx| cx.emit(CheckboxEvent::Change { checked: false }));
        cx.run_until_parked();
        assert!(cx.debug_bounds("popover-panel").is_none(), "closed");
        let closed_stage = cx.debug_bounds("workshop-sdk-popover").expect("the stage");
        assert_eq!(closed_stage.size, open_stage.size, "the stage keeps its size while the panel is closed");

        option.update(cx, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }));
        cx.run_until_parked();
        assert!(cx.debug_bounds("popover-panel").is_some(), "open again");
    }
}
