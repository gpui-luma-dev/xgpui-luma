//! The SDK button on the design surface page (issue #376). A button has no lasting state: it is never
//! on or off, so its key only ever follows hover, press and focus, and Test mode logs its clicks.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, StyledButtonTemplate};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::Theme;

use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;

/// The button's text, on the stage and in each State matrix cell.
const LABEL: &str = "Button";

/// The button template sets `selected` only for a toggle; a plain button never reaches it.
const TOGGLE_ONLY: &str = "selected";

pub(super) struct ButtonSurface {
    button: Entity<Button>,
}

impl ButtonSurface {
    pub(super) fn make(cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let button = Button::new("workshop-sdk-button").content(|_, _| div().child(LABEL)).spawn(cx);
        Box::new(Self { button })
    }

    /// The SDK button, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn button(&self) -> Entity<Button> {
        self.button.clone()
    }
}

impl SurfaceControl for ButtonSurface {
    fn title(&self) -> &'static str {
        "Button"
    }

    fn view(&self) -> AnyView {
        self.button.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let pin = pinned.map(|states| Pin::<()> { state: states.interaction(), data: Arc::new(|_: &()| ()) });
        let template = PinnedTemplate::new(Arc::new(StyledButtonTemplate::<()>::new(theme)), pin);
        self.button.update(cx, |button, cx| button.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        StyledButtonTemplate::<()>::sdk().style_states().into_iter().filter(|s| *s != TOGGLE_ONLY).collect()
    }

    /// `ButtonBuilder` has no way to make a button invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.button, move |side: &mut ControlSide, _, event: &ButtonEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, ButtonEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                ButtonEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                ButtonEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
        let model = ButtonRenderModel {
            id,
            state: states.interaction(),
            content: Arc::new(|_, _| div().child(LABEL).into_any_element()),
            ..Default::default()
        };
        StyledButtonTemplate::<()>::new(theme.clone()).render(&model, window, cx).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{TestAppContext, VisualTestContext};

    use super::*;
    use crate::workshop::surface_page::{SurfacePage, WorkMode};
    use crate::workshop::{Sample, Workshop};

    fn open(app: &mut TestAppContext) -> (Entity<SurfacePage>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |v, cx| v.select("button", cx));
        cx.run_until_parked();
        let page = cx.update(|_, cx| match &workshop.read(cx).samples["button"] {
            Sample::Surface(v) => v.clone(),
            _ => panic!("the button page is a design surface page"),
        });
        (page, cx)
    }

    #[test]
    fn the_button_page_is_a_design_surface_page() {
        let mut app = TestAppContext::single();
        let (page, cx) = open(&mut app);
        for id in ["workshop-surface-pane", "workshop-surface-control", "workshop-surface-live", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no old inspector column");
        let side = cx.update(|_, cx| page.read(cx).side());
        let button = cx.update(|_, cx| side.read(cx).control_as::<ButtonSurface>().expect("a button").button());

        // Test: a click is logged; a button has no lasting state, so the key only follows hover and focus.
        let at = cx.debug_bounds("workshop-surface-control").expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Click"), "the click is logged: {text}");

        // No :selected (the template sets it only for a toggle); a button can't be invalid.
        let template_only = cx.update(|_, cx| side.read(cx).template_only_states());
        assert!(template_only.iter().all(|(name, _)| name != "selected"), "{template_only:?}");
        assert!(template_only.contains(&("invalid".to_string(), false)), "{template_only:?}");

        // Design: a picked :pressed draws on the button, which ignores input.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        // The click focused the button, and Design keeps the key as it was, so :focused stays picked.
        assert_eq!(pinned, StateSet::new(["focused", "pressed"]));
        cx.update(|window, cx| assert!(button.read(cx).render_model(window).state.disabled, "Design: no input"));

        // The State matrix: the interaction states across, focus and invalid down.
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        assert_eq!(rows.len(), 4, "{rows:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw buttons");
    }
}
