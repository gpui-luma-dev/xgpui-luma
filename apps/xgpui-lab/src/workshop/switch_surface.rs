//! The SDK switch on the design surface page (issue #376).

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::switch::{self, StyledSwitchTemplate, SwitchData, SwitchEvent};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::Theme;

use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The switch's text, on the stage and in each State matrix cell.
const LABEL: &str = "Label";

pub(super) struct SwitchSurface {
    switch: switch::Switch,
}

impl SwitchSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let switch = switch::new("workshop-sdk-switch").content(|_, _| div().child(LABEL)).spawn(cx);
        Box::new(Self { switch })
    }

    /// The SDK switch, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn switch(&self) -> switch::Switch {
        self.switch.clone()
    }
}

impl SurfaceControl for SwitchSurface {
    fn title(&self) -> &'static str {
        "Switch"
    }

    fn view(&self) -> AnyView {
        self.switch.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        // A pinned switch draws its thumb settled at its end (no slide in flight).
        let pin = pinned.map(|states| {
            let checked = states.has("checked");
            Pin::<SwitchData> {
                state: states.interaction(),
                data: Arc::new(move |_: &SwitchData| SwitchData::new(checked)),
            }
        });
        let template = PinnedTemplate::new(Arc::new(StyledSwitchTemplate::new(theme)), pin);
        self.switch.update(cx, |switch, cx| switch.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.switch.update(cx, |switch, cx| switch.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        switch::default_switch_template().style_states()
    }

    /// A switch is on or off; its `.lss` calls on `checked`.
    fn state_label(&self, state: &str) -> Option<&'static str> {
        (state == "checked").then_some("on")
    }

    /// `SwitchBuilder` has no way to make a switch invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("checked", *self.switch.read(cx).data())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.switch, move |side: &mut ControlSide, _, event: &SwitchEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, SwitchEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                SwitchEvent::Change { on } => Some(("checked", *on)),
                SwitchEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                SwitchEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
            data: SwitchData::new(states.has("checked")),
            state: states.interaction(),
            content: Arc::new(|_, _| div().child(LABEL).into_any_element()),
            ..Default::default()
        };
        StyledSwitchTemplate::new(theme.clone()).render(&model, window, cx).into_any_element()
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
        workshop.update(cx, |v, cx| v.select("switch", cx));
        cx.run_until_parked();
        let page = cx.update(|_, cx| match &workshop.read(cx).samples["switch"] {
            Sample::Surface(v) => v.clone(),
            _ => panic!("the switch page is a design surface page"),
        });
        (page, cx)
    }

    #[test]
    fn the_switch_page_is_a_design_surface_page() {
        let mut app = TestAppContext::single();
        let (page, cx) = open(&mut app);
        for id in ["workshop-surface-pane", "workshop-surface-control", "workshop-surface-live", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no old inspector column");
        let side = cx.update(|_, cx| page.read(cx).side());
        let switch = cx.update(|_, cx| side.read(cx).control_as::<SwitchSurface>().expect("a switch").switch());

        // Test: clicking the live switch turns it on; the key follows :checked and the event is logged.
        let at = cx.debug_bounds("workshop-surface-control").expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *switch.read(cx).data()), "clicked on");
        assert!(cx.update(|_, cx| side.read(cx).state_on("checked")), "the key follows :checked");

        // The template sets states no switch rule uses; a switch can't be invalid.
        let template_only = cx.update(|_, cx| side.read(cx).template_only_states());
        assert!(template_only.contains(&("focused".to_string(), true)), "{template_only:?}");
        assert!(template_only.contains(&("invalid".to_string(), false)), "{template_only:?}");

        // Design: picked states draw on the switch, which ignores input.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover") && pinned.has("checked"), "{pinned:?}");
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *switch.read(cx).data()), "Design: the click is ignored");

        // The State matrix: the exclusive interaction states across, every other combination down.
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        assert_eq!(rows.len(), 8, "{rows:?}");
        // A switch is on, not checked: the design controls use its word; the key keeps the `.lss` name.
        assert!(rows.iter().any(|r| r == ":on"), "{rows:?}");
        assert!(rows.iter().all(|r| !r.contains("checked")), "{rows:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw switches");
        assert!(cx.debug_bounds("workshop-surface-matrix-label-:on").is_some(), "the :on row label");
    }
}
