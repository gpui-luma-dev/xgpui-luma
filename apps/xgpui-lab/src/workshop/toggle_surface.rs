//! The SDK toggle on the design surface page (issue #376). A toggle is a button that stays on: it is
//! drawn by the button template, which reads the `toggle` control (`toggle extends button`) for a model
//! in the toggle role. Its `.lss` calls on `selected`; the page says `on`.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, StyledButtonTemplate};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::toggle::{self, ToggleData, ToggleEvent, default_toggle_template};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::Theme;

use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The toggle's text, on the stage and in each State matrix cell.
const LABEL: &str = "Bold";

pub(super) struct ToggleSurface {
    toggle: toggle::Toggle,
}

impl ToggleSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let toggle = toggle::new("workshop-sdk-toggle").content(|_, _| div().child(LABEL)).spawn(cx);
        Box::new(Self { toggle })
    }

    /// The SDK toggle, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle(&self) -> toggle::Toggle {
        self.toggle.clone()
    }
}

impl SurfaceControl for ToggleSurface {
    fn title(&self) -> &'static str {
        "Toggle"
    }

    fn view(&self) -> AnyView {
        self.toggle.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        // A pinned toggle draws its face settled on or off (no blend in flight).
        let pin = pinned.map(|states| {
            let on = states.has("selected");
            Pin::<ToggleData> { state: states.interaction(), data: Arc::new(move |_: &ToggleData| ToggleData::new(on)) }
        });
        let template = PinnedTemplate::new(Arc::new(StyledButtonTemplate::<ToggleData>::new(theme)), pin);
        self.toggle.update(cx, |toggle, cx| toggle.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.toggle.update(cx, |toggle, cx| toggle.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        default_toggle_template().style_states()
    }

    /// A toggle is on or off; its `.lss` calls on `selected`.
    fn state_label(&self, state: &str) -> Option<&'static str> {
        (state == "selected").then_some("on")
    }

    /// `ToggleBuilder` has no way to make a toggle invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", *self.toggle.read(cx).data())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.toggle, move |side: &mut ControlSide, _, event: &ToggleEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, ToggleEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                ToggleEvent::Change { selected } => Some(("selected", *selected)),
                ToggleEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                ToggleEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
        let on = states.has("selected");
        // The toggle role makes the button template read the `toggle` control.
        let model = ButtonRenderModel {
            id,
            data: ToggleData::new(on),
            role: ButtonFamilyRole::Toggle { selected: on },
            state: states.interaction(),
            content: Arc::new(|_, _| div().child(LABEL).into_any_element()),
            ..Default::default()
        };
        StyledButtonTemplate::<ToggleData>::new(theme.clone()).render(&model, window, cx).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Entity, TestAppContext, VisualTestContext};

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
        workshop.update(cx, |v, cx| v.select("toggle", cx));
        cx.run_until_parked();
        let page = cx.update(|_, cx| match &workshop.read(cx).samples["toggle"] {
            Sample::Surface(v) => v.clone(),
            _ => panic!("the toggle page is a design surface page"),
        });
        (page, cx)
    }

    #[test]
    fn the_toggle_page_is_a_design_surface_page() {
        let mut app = TestAppContext::single();
        let (page, cx) = open(&mut app);
        for id in ["workshop-surface-pane", "workshop-surface-control", "workshop-surface-live", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no old inspector column");
        let side = cx.update(|_, cx| page.read(cx).side());
        let toggle = cx.update(|_, cx| side.read(cx).control_as::<ToggleSurface>().expect("a toggle").toggle());

        // Test: clicking the live toggle turns it on; the key follows :selected.
        let at = cx.debug_bounds("workshop-surface-control").expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *toggle.read(cx).data()), "clicked on");
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "the key follows :selected");

        // Design: picked states draw on the toggle, which ignores input.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover") && pinned.has("selected"), "{pinned:?}");
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *toggle.read(cx).data()), "Design: the click is ignored");

        // A toggle is on, not selected or checked: the design controls use its word.
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        assert!(rows.iter().any(|r| r == ":on"), "{rows:?}");
        assert!(rows.iter().all(|r| !r.contains("selected") && !r.contains("checked")), "{rows:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw toggles");
    }
}
