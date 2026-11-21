//! The checkbox on the shared choice template (issue #285): a checkmark mark in the `glyph` part.

use gpui::{AnyElement, App, Div, Hsla, Stateful, Window, div, prelude::*, px};

use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::checkbox::CheckboxData;
use crate::controls::choice::styled::{ChoiceData, IndicatorChoice, StyledChoiceTemplate, render_indicator};

/// A checkbox template whose values come from `.lss` files.
pub type StyledCheckboxTemplate = StyledChoiceTemplate<CheckboxData>;

impl ChoiceData for CheckboxData {
    const CONTROL: &'static str = "checkbox";
    const ON_STATE: &'static str = "checked";

    fn is_on(&self) -> bool {
        self.checked
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn toward(on: bool) -> Self {
        CheckboxData::new(on)
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

impl IndicatorChoice for CheckboxData {
    const MARK_PART: &'static str = "glyph";
    const MARK_SIZE: &'static str = "glyph_size";

    fn render_mark(&self, progress: f32, size: f32, color: Hsla) -> AnyElement {
        render_checkmark(progress, size, color, &self.icons.selected)
    }
}

/// The SDK's default checkbox template: the SDK's `.lss` files alone.
pub fn default_template() -> std::sync::Arc<dyn ButtonTemplate<CheckboxData>> {
    static TEMPLATE: std::sync::OnceLock<std::sync::Arc<dyn ButtonTemplate<CheckboxData>>> = std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| std::sync::Arc::new(StyledCheckboxTemplate::sdk())).clone()
}

fn render_checkmark(progress: f32, size: f32, color: Hsla, icon: &crate::infra::icon::IconSource) -> AnyElement {
    if progress <= f32::EPSILON {
        return div().size(px(size)).into_any_element();
    }
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .opacity(progress)
        .child(crate::infra::icon::render_icon_source(icon, color, size))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::IntoElement;

    use crate::controls::button_family::ButtonFamilyRole;
    use crate::theme::InteractionState;

    use super::*;
    use crate::controls::button::{ButtonRenderModel, ButtonTemplate};

    fn model(state: InteractionState, checked: bool, role: ButtonFamilyRole) -> ButtonRenderModel<CheckboxData> {
        ButtonRenderModel { data: CheckboxData::new(checked), state, role, ..Default::default() }
    }

    #[test]
    fn styled_checkbox_layout_is_stable_across_states_and_scales() {
        for role in [ButtonFamilyRole::Text, ButtonFamilyRole::Icon] {
            let template = StyledCheckboxTemplate::sdk();
            crate::infra::focus_adorner::testing::assert_stable_layout(move |state, checked, window, cx| {
                template.render(&model(state, checked, role), window, cx).into_any_element()
            });
        }
    }
}
