//! The SDK radio button on the design surface page (issue #376). Its `.lss` (`radio`) names its
//! on-state `selected`, the radio's own word, so it needs no state labels.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::radio_button::{
    self, RadioButtonData, RadioButtonEvent, StyledRadioButtonTemplate, default_radio_button_template,
};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::Theme;

use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The radio button's text, on the stage and in each State matrix cell, with ascenders and descenders.
const LABEL: &str = "Quality";

pub(super) struct RadioButtonSurface {
    radio: radio_button::RadioButton,
}

impl RadioButtonSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let radio = radio_button::new("workshop-sdk-radio-button").content(|_, _| div().child(LABEL)).spawn(cx);
        Box::new(Self { radio })
    }

    /// The SDK radio button, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn radio(&self) -> radio_button::RadioButton {
        self.radio.clone()
    }
}

impl SurfaceControl for RadioButtonSurface {
    fn title(&self) -> &'static str {
        "Radio Button"
    }

    fn view(&self) -> AnyView {
        self.radio.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        // A pinned radio button draws its dot settled (no fill animation in flight).
        let pin = pinned.map(|states| {
            let selected = states.has("selected");
            Pin::<RadioButtonData> {
                state: states.interaction(),
                data: Arc::new(move |_: &RadioButtonData| RadioButtonData::new(selected)),
            }
        });
        let template = PinnedTemplate::new(Arc::new(StyledRadioButtonTemplate::new(theme)), pin);
        self.radio.update(cx, |radio, cx| radio.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.radio.update(cx, |radio, cx| radio.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        default_radio_button_template().style_states()
    }

    /// `RadioButtonBuilder` has no way to make a radio button invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", *self.radio.read(cx).data())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.radio, move |side: &mut ControlSide, _, event: &RadioButtonEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, RadioButtonEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                RadioButtonEvent::Change { selected } => Some(("selected", *selected)),
                RadioButtonEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                RadioButtonEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
            data: RadioButtonData::new(states.has("selected")),
            state: states.interaction(),
            content: Arc::new(|_, _| div().child(LABEL).into_any_element()),
            ..Default::default()
        };
        StyledRadioButtonTemplate::new(theme.clone()).render(&model, window, cx).into_any_element()
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
        workshop.update(cx, |v, cx| v.select("radio-button", cx));
        cx.run_until_parked();
        let page = cx.update(|_, cx| match &workshop.read(cx).samples["radio-button"] {
            Sample::Surface(v) => v.clone(),
            _ => panic!("the radio button page is a design surface page"),
        });
        (page, cx)
    }

    #[test]
    fn the_radio_button_page_is_a_design_surface_page() {
        let mut app = TestAppContext::single();
        let (page, cx) = open(&mut app);
        for id in ["workshop-surface-pane", "workshop-surface-control", "workshop-surface-live", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no old inspector column");
        let side = cx.update(|_, cx| page.read(cx).side());
        let radio = cx.update(|_, cx| side.read(cx).control_as::<RadioButtonSurface>().expect("a radio").radio());

        // Test: clicking the live radio button selects it; the key follows :selected.
        let at = cx.debug_bounds("workshop-surface-control").expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *radio.read(cx).data()), "clicked: selected");
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "the key follows :selected");

        // A radio button can't be invalid.
        let template_only = cx.update(|_, cx| side.read(cx).template_only_states());
        assert!(template_only.contains(&("invalid".to_string(), false)), "{template_only:?}");

        // Design: picked states draw on the radio button.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("pressed") && pinned.has("selected"), "{pinned:?}");

        // The State matrix speaks the radio's words: :selected, never checked.
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        assert!(rows.iter().any(|r| r == ":selected"), "{rows:?}");
        assert!(rows.iter().all(|r| !r.contains("checked")), "{rows:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw radio buttons");
    }
}
