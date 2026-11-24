//! The SDK checkbox on the design surface page (issues #291, #376).

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::checkbox::{self, CheckboxData, CheckboxEvent, StyledCheckboxTemplate};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::Theme;

use super::surface_page::ControlSide;
use crate::theme::Look;
use super::pinned_template::{Pin, PinnedTemplate};
use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};

/// The checkbox's text, on the stage and in each State matrix cell, with ascenders and descenders.
const LABEL: &str = "Apply";

pub(super) struct CheckboxSurface {
    checkbox: checkbox::Checkbox,
}

impl CheckboxSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let checkbox = checkbox::new("workshop-sdk-checkbox").content(|_, _| div().child(LABEL)).spawn(cx);
        Box::new(Self { checkbox })
    }

    /// The SDK checkbox, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn checkbox(&self) -> checkbox::Checkbox {
        self.checkbox.clone()
    }
}

impl SurfaceControl for CheckboxSurface {
    fn title(&self) -> &'static str {
        "Checkbox"
    }

    fn view(&self) -> AnyView {
        self.checkbox.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        // A pinned checkbox draws its checked value settled (no check animation in flight).
        let pin = pinned.map(|states| {
            let checked = states.has("checked");
            Pin::<CheckboxData> {
                state: states.interaction(),
                data: Arc::new(move |live: &CheckboxData| CheckboxData {
                    checked,
                    progress: if checked { 1.0 } else { 0.0 },
                    icons: live.icons.clone(),
                }),
            }
        });
        let template = PinnedTemplate::new(Arc::new(StyledCheckboxTemplate::new(theme)), pin);
        self.checkbox.update(cx, |checkbox, cx| checkbox.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.checkbox.update(cx, |checkbox, cx| checkbox.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        checkbox::default_checkbox_template().style_states()
    }

    /// `CheckboxBuilder` has no way to make a checkbox invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("checked", *self.checkbox.read(cx).data())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.checkbox, move |side: &mut ControlSide, _, event: &CheckboxEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, CheckboxEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                CheckboxEvent::Change { checked } => Some(("checked", *checked)),
                CheckboxEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                CheckboxEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
            data: CheckboxData::new(states.has("checked")),
            state: states.interaction(),
            content: Arc::new(|_, _| div().child(LABEL).into_any_element()),
            ..Default::default()
        };
        StyledCheckboxTemplate::new(theme.clone()).render(&model, window, cx).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
