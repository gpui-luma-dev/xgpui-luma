//! The SDK textarea on the design surface page (issue #383). Its `.lss` (`textarea`, which extends
//! `textfield`) reads hover, focus, invalid and disabled; the textarea is invalid while its text holds a
//! digit. It emits no hover event, so Test mode's key doesn't follow hover. State matrix cells show the
//! placeholder only: the template draws text from line measurements only the live control makes.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::text::StyledTextAreaTheme;
use xgpui_luma::controls::textarea::{
    TextArea, TextAreaBuilder, TextAreaEvent, TextAreaRenderModel, TextAreaState, TextAreaTemplate,
    TextAreaTemplateHandlers, TextAreaTheme, ThemedTextAreaTemplate,
};
use xgpui_luma::style::ThemeSource;
use xgpui_luma::style::lss::Theme;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The placeholder, with ascenders and descenders.
const PLACEHOLDER: &str = "Jot down a few quick thoughts…";

/// The `.lss` states the textarea theme reads.
const STATES: [&str; 4] = ["hover", "focused", "invalid", "disabled"];

/// Rows the textarea shows.
const ROWS: usize = 3;
/// The textarea fills its width: the stage's (scaled with the zoom) and each matrix cell's.
const STAGE_WIDTH: f32 = 360.;
const CELL_WIDTH: f32 = 300.;

pub(super) struct TextAreaSurface {
    area: Entity<TextArea>,
}

impl TextAreaSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let area = TextAreaBuilder::new("workshop-sdk-textarea")
            .placeholder(PLACEHOLDER)
            .rows(ROWS)
            .full_width(true)
            .validator(Arc::new(|text: &str| !text.chars().any(|c| c.is_ascii_digit())))
            .spawn(cx);
        Box::new(Self { area })
    }

    /// The SDK textarea, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn area(&self) -> Entity<TextArea> {
        self.area.clone()
    }
}

/// The textarea theme for the `textarea` control in `theme`.
fn theme_for(theme: Theme) -> Arc<dyn TextAreaTheme> {
    Arc::new(StyledTextAreaTheme::new(ThemeSource::Fixed(theme)))
}

/// `live` with the picked states in place of its hover, focus and invalid.
fn pinned_state(live: TextAreaState, states: &StateSet) -> TextAreaState {
    let focused = states.has("focused");
    TextAreaState {
        hovered: states.has("hover"),
        focused,
        focus_visible: focused,
        invalid: states.has("invalid"),
        ..live
    }
}

/// Draws the base template with the picked states (Design mode), or as the textarea is.
struct PinnedTextAreaTemplate {
    base: ThemedTextAreaTemplate,
    theme: Arc<dyn TextAreaTheme>,
    pinned: Option<StateSet>,
}

impl TextAreaTemplate for PinnedTextAreaTemplate {
    fn theme(&self) -> Option<Arc<dyn TextAreaTheme>> {
        Some(self.theme.clone())
    }

    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let Some(states) = &self.pinned else {
            return self.base.render(model, handlers, window, cx);
        };
        let pinned = TextAreaRenderModel {
            id: model.id,
            placeholder: model.placeholder,
            value: model.value,
            enabled: !states.has("disabled"),
            full_width: model.full_width,
            rows: model.rows,
            state: pinned_state(model.state, states),
            caret_visible: false,
            vertical_scroll: model.vertical_scroll,
            line_metrics: model.line_metrics.clone(),
        };
        self.base.render(&pinned, handlers, window, cx)
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(|_, _, _| {}),
        mouse_down: Box::new(|_, _, _| {}),
        mouse_move: Box::new(|_, _, _| {}),
        mouse_up: Box::new(|_, _, _| {}),
        mouse_up_out: Box::new(|_, _, _| {}),
        click: Box::new(|_, _, _| {}),
        key_down: Box::new(|_, _, _| {}),
        drag_move: Box::new(|_, _, _| {}),
    }
}

impl SurfaceControl for TextAreaSurface {
    fn title(&self) -> &'static str {
        "Text Area"
    }

    fn view(&self) -> AnyView {
        self.area.clone().into()
    }

    /// The textarea fills the width it's given.
    fn stage_width(&self) -> Option<f32> {
        Some(STAGE_WIDTH)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let theme = theme_for(theme);
        let template =
            PinnedTextAreaTemplate { base: ThemedTextAreaTemplate::new(theme.clone()), theme, pinned: pinned.cloned() };
        self.area.update(cx, |area, cx| area.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.area.update(cx, |area, cx| area.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("invalid", self.area.read(cx).state().invalid)]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.area, move |side: &mut ControlSide, area, event: &TextAreaEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, TextAreaEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                TextAreaEvent::Change { .. } => Some(("invalid", area.read(cx).state().invalid)),
                TextAreaEvent::FocusChanged { focused } => Some(("focused", *focused)),
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
        let template = ThemedTextAreaTemplate::new(theme_for(theme.clone()));
        let (placeholder, value) = (SharedString::from(PLACEHOLDER), SharedString::default());
        let model = TextAreaRenderModel {
            id: &id,
            placeholder: &placeholder,
            value: &value,
            enabled: !states.has("disabled"),
            full_width: true,
            rows: ROWS,
            state: pinned_state(TextAreaState::default(), states),
            caret_visible: false,
            vertical_scroll: 0.,
            line_metrics: Vec::new(),
        };
        div()
            .w(px(CELL_WIDTH))
            .child(template.render(&model, inert_handlers(), window, cx))
            .into_any_element()
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
    fn the_text_area_page_edits_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "textarea");
        let side = cx.update(|_, cx| page.read(cx).side());
        let area = cx.update(|_, cx| side.read(cx).control_as::<TextAreaSurface>().expect("a textarea").area());

        cx.update(|window, cx| area.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_input("Quietly");
        cx.simulate_keystrokes("enter");
        cx.simulate_input("typed 2");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(area.read(cx).value(), "Quietly\ntyped 2"));
        assert!(cx.update(|_, cx| side.read(cx).state_on("invalid")), "the key follows :invalid");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("disabled"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw textareas");
    }
}
