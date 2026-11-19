//! The radio button on the shared choice template (issue #285): a dot mark in the `dot` part.

use gpui::{AnyElement, App, Div, ElementId, Hsla, SharedString, Stateful, Window, div, prelude::*, px};

use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::choice::styled::{ChoiceData, IndicatorChoice, StyledChoiceTemplate, render_indicator};
use crate::controls::radio_button::RadioButtonData;

/// A radio button template whose values come from `.lss` files.
pub type StyledRadioButtonTemplate = StyledChoiceTemplate<RadioButtonData>;

impl ChoiceData for RadioButtonData {
    const CONTROL: &'static str = "radio";
    const ON_STATE: &'static str = "selected";

    fn is_on(&self) -> bool {
        self.selected
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn toward(on: bool) -> Self {
        RadioButtonData::new(on)
    }

    fn root_id(id: &SharedString) -> ElementId {
        (id.clone(), 0usize).into()
    }

    fn render(
        template: &StyledChoiceTemplate<Self>,
        model: &ButtonRenderModel<Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        render_indicator(template, model, window, cx)
    }
}

impl IndicatorChoice for RadioButtonData {
    const MARK_PART: &'static str = "dot";
    const MARK_SIZE: &'static str = "dot_size";
    const ICON_ROLE_KEEPS_ROW: bool = true;

    /// The dot scales with progress so selection feels continuous.
    fn render_mark(&self, progress: f32, size: f32, color: Hsla) -> AnyElement {
        if progress <= f32::EPSILON || size <= 0.0 {
            return div().size(px(size)).into_any_element();
        }
        let dot = (size * progress).max(1.0);
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .child(div().size(px(dot)).bg(color).rounded(px(dot)).opacity(progress))
            .into_any_element()
    }
}

/// The SDK's default radio button template: the SDK's `.lss` files alone.
pub fn default_template() -> std::sync::Arc<dyn ButtonTemplate<RadioButtonData>> {
    static TEMPLATE: std::sync::OnceLock<std::sync::Arc<dyn ButtonTemplate<RadioButtonData>>> =
        std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| std::sync::Arc::new(StyledRadioButtonTemplate::sdk())).clone()
}
