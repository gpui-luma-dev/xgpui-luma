//! The SDK text field on the design surface page (issue #383). Its `.lss` (`textfield`) reads hover,
//! focus, invalid and disabled; the field is invalid while its text holds a digit, so `:invalid` can be
//! reached by typing one. Test mode's key follows hover and focus from the field's events.

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, prelude::*};
use xgpui_luma::controls::text::StyledTextFieldTheme;
use xgpui_luma::controls::textfield::{
    self, TextField, TextFieldEvent, TextFieldLook, TextFieldRenderModel, TextFieldState, TextFieldTemplate,
    TextFieldTemplateHandlers, ThemedTextFieldTemplate,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::ThemeSource;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The live field's placeholder, with ascenders and descenders.
const PLACEHOLDER: &str = "Type a quick reply…";
/// The text each State matrix cell shows, with ascenders and descenders.
const SAMPLE: &str = "Quickly typed";

/// The `.lss` states the text-field themes read.
pub(super) const STATES: [&str; 4] = ["hover", "focused", "invalid", "disabled"];

pub(super) struct TextFieldSurface {
    field: TextField,
}

impl TextFieldSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let field = textfield::new("workshop-sdk-textfield")
            .placeholder(PLACEHOLDER)
            .validator(Arc::new(|text: &str| !text.chars().any(|c| c.is_ascii_digit())))
            .spawn(cx);
        Box::new(Self { field })
    }

    /// The SDK text field, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn field(&self) -> TextField {
        self.field.clone()
    }
}

/// The text-field template for the `textfield` control in `theme`.
pub(super) fn template(theme: Theme) -> ThemedTextFieldTemplate {
    let theme = StyledTextFieldTheme::new(ThemeSource::Fixed(theme));
    ThemedTextFieldTemplate::new(Arc::new(theme))
}

/// `live` with the picked states in place of its hover, focus and invalid.
fn pinned_state(live: TextFieldState, states: &StateSet) -> TextFieldState {
    let focused = states.has("focused");
    TextFieldState {
        hovered: states.has("hover"),
        focused,
        focus_visible: focused,
        invalid: states.has("invalid"),
        ..live
    }
}

/// Draws the base template with the picked states (Design mode), or as the field is.
pub(super) struct PinnedTextFieldTemplate {
    pub(super) base: ThemedTextFieldTemplate,
    pub(super) pinned: Option<StateSet>,
}

impl TextFieldTemplate for PinnedTextFieldTemplate {
    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        scale_factor: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        match &self.pinned {
            Some(states) => self.base.resolve_look_with_scale(
                pinned_state(state, states),
                !states.has("disabled"),
                scale_factor,
                cx,
            ),
            None => self.base.resolve_look_with_scale(state, enabled, scale_factor, cx),
        }
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let Some(states) = &self.pinned else {
            return self.base.render(model, handlers, window, cx);
        };
        let pinned = TextFieldRenderModel {
            id: model.id,
            placeholder: model.placeholder,
            value: model.value,
            prefix_icon: model.prefix_icon,
            enabled: !states.has("disabled"),
            full_width: model.full_width,
            state: pinned_state(model.state, states),
            marked_range: model.marked_range.clone(),
            caret_visible: false,
            horizontal_scroll: model.horizontal_scroll,
            character_offsets: model.character_offsets.clone(),
            look: model.look.clone(),
        };
        self.base.render(&pinned, handlers, window, cx)
    }
}

/// Handlers that do nothing, for a State matrix cell.
fn inert_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
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

impl SurfaceControl for TextFieldSurface {
    fn title(&self) -> &'static str {
        "Text Field"
    }

    fn view(&self) -> AnyView {
        self.field.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedTextFieldTemplate { base: template(theme), pinned: pinned.cloned() };
        self.field.update(cx, |field, cx| field.set_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.field.update(cx, |field, cx| field.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("invalid", self.field.read(cx).state().invalid)]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.field, move |side: &mut ControlSide, field, event: &TextFieldEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, TextFieldEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                TextFieldEvent::Change { .. } => Some(("invalid", field.read(cx).state().invalid)),
                TextFieldEvent::FocusChanged { focused } => Some(("focused", *focused)),
                TextFieldEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
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
        let template = template(theme.clone());
        let enabled = !states.has("disabled");
        let state = pinned_state(TextFieldState::default(), states);
        let look = template.resolve_look_with_scale(state, enabled, 1.0, cx);
        // The placeholder sizes the field; the sample text is drawn over it.
        let (placeholder, value) = (SharedString::from(SAMPLE), SharedString::from(SAMPLE));
        let model = TextFieldRenderModel {
            id: &id,
            placeholder: &placeholder,
            value: &value,
            prefix_icon: None,
            enabled,
            full_width: false,
            state,
            marked_range: None,
            caret_visible: false,
            horizontal_scroll: 0.,
            character_offsets: Vec::new(),
            look,
        };
        template.render(&model, inert_handlers(), window, cx).into_any_element()
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
    fn the_text_field_page_edits_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "textfield");
        assert!(cx.debug_bounds("workshop-surface-control").is_some());
        let side = cx.update(|_, cx| page.read(cx).side());
        let field = cx.update(|_, cx| side.read(cx).control_as::<TextFieldSurface>().expect("a text field").field());

        // Test: typing a digit makes the field invalid, and the key follows.
        cx.update(|window, cx| field.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_input("Jumpy 4");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(field.read(cx).value(), "Jumpy 4"));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Change"), "edits are logged: {text}");
        assert!(cx.update(|_, cx| side.read(cx).state_on("invalid")), "the key follows :invalid");

        // Design: the picked states draw on the field.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("hover") && pinned.has("invalid"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw fields");
    }
}
