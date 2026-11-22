//! Pinned states for a live control on a design surface (issue #301), like a designer's state picker
//! (Blend's States panel, Interface Builder's State Config): the control draws the chosen states
//! whatever the pointer and keyboard focus are doing. The wrapped template and the control are
//! unchanged; only the render model the template sees is replaced.

use std::cell::Cell;
use std::sync::Arc;

use gpui::{App, Div, Stateful, Window};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::checkbox::CheckboxData;
use xgpui_luma::theme::InteractionState;

/// The states a pinned checkbox draws: interaction states and whether it is checked.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PinnedCheckbox {
    pub state: InteractionState,
    pub checked: bool,
}

/// Draws `base` with `pinned` in place of the live interaction state and checked value, or as the
/// control is when nothing is pinned.
pub(super) struct PinnedCheckboxTemplate {
    base: Arc<dyn ButtonTemplate<CheckboxData>>,
    pinned: Option<PinnedCheckbox>,
}

impl PinnedCheckboxTemplate {
    pub(super) fn new(base: Arc<dyn ButtonTemplate<CheckboxData>>, pinned: Option<PinnedCheckbox>) -> Self {
        Self { base, pinned }
    }
}

impl ButtonTemplate<CheckboxData> for PinnedCheckboxTemplate {
    fn render(&self, model: &ButtonRenderModel<CheckboxData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        match self.pinned {
            Some(pinned) => self.base.render(&pinned_model(model, pinned), window, cx),
            None => self.base.render(model, window, cx),
        }
    }

    fn transition(&self, data: &CheckboxData) -> Option<xgpui_luma::motion::Transition> {
        self.base.transition(data)
    }

    fn style_states(&self) -> Vec<&'static str> {
        self.base.style_states()
    }
}

/// `model` with the pinned states; the checked value is settled (no check animation in flight).
fn pinned_model(model: &ButtonRenderModel<CheckboxData>, pinned: PinnedCheckbox) -> ButtonRenderModel<CheckboxData> {
    let data = CheckboxData {
        checked: pinned.checked,
        progress: if pinned.checked { 1.0 } else { 0.0 },
        icons: model.data.icons.clone(),
    };
    ButtonRenderModel {
        id: model.id.clone(),
        data,
        content: model.content.clone(),
        icon: model.icon.clone(),
        role: model.role,
        state: pinned.state,
        round: model.round,
        radius_override: Cell::new(model.radius_override.get()),
        elevation: model.elevation,
        compact: model.compact,
        switch_track_width_extra: model.switch_track_width_extra,
        switch_track_width: model.switch_track_width,
        switch_track_height: model.switch_track_height,
        switch_thumb_size: model.switch_thumb_size,
        switch_orientation: model.switch_orientation,
        switch_track_content: model.switch_track_content.clone(),
        switch_thumb_content: model.switch_thumb_content.clone(),
        look: model.look.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pinned_model_replaces_only_the_state_and_checked_value() {
        let model = ButtonRenderModel::<CheckboxData> {
            id: "box".into(),
            state: InteractionState { hovered: true, ..Default::default() },
            compact: true,
            ..Default::default()
        };
        model.radius_override.set(Some(3.));
        let pinned = PinnedCheckbox { state: InteractionState { focused: true, ..Default::default() }, checked: true };
        let drawn = pinned_model(&model, pinned);
        assert_eq!(drawn.state, pinned.state);
        assert!(drawn.data.checked);
        assert_eq!(drawn.data.progress, 1.0, "settled, not animating");
        assert_eq!((drawn.id.as_ref(), drawn.compact, drawn.radius_override.get()), ("box", true, Some(3.)));
    }
}
