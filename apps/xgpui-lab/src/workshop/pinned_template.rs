//! Pinned states for a live control on a design surface (issue #301), like a designer's state picker
//! (Blend's States panel, Interface Builder's State Config): the control draws the chosen states
//! whatever the pointer and keyboard focus are doing. The wrapped template and the control are
//! unchanged; only the render model the template sees is replaced. Works for any SDK control drawn by a
//! `ButtonTemplate` (checkbox, switch, toggle, button…).

use std::cell::Cell;
use std::sync::Arc;

use gpui::{App, Div, Stateful, Window};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::theme::InteractionState;

/// The data a pinned control draws, from its live data (e.g. a checkbox's `checked`, keeping its icons).
pub(super) type PinData<D> = Arc<dyn Fn(&D) -> D + Send + Sync>;

/// What a pinned control draws: its interaction states, and its data from the live data.
pub(super) struct Pin<D> {
    pub state: InteractionState,
    pub data: PinData<D>,
}

/// Draws `base` with the pin's states and data in place of the live ones, or as the control is when
/// nothing is pinned.
pub(super) struct PinnedTemplate<D> {
    base: Arc<dyn ButtonTemplate<D>>,
    pin: Option<Pin<D>>,
}

impl<D> PinnedTemplate<D> {
    pub(super) fn new(base: Arc<dyn ButtonTemplate<D>>, pin: Option<Pin<D>>) -> Self {
        Self { base, pin }
    }
}

impl<D: Clone + 'static> ButtonTemplate<D> for PinnedTemplate<D> {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        match &self.pin {
            Some(pin) => self.base.render(&pinned_model(model, pin), window, cx),
            None => self.base.render(model, window, cx),
        }
    }

    fn transition(&self, data: &D) -> Option<xgpui_luma::motion::Transition> {
        self.base.transition(data)
    }

    fn style_states(&self) -> Vec<&'static str> {
        self.base.style_states()
    }
}

/// `model` with the pin's interaction states and data; everything else is the live model's.
fn pinned_model<D: Clone>(model: &ButtonRenderModel<D>, pin: &Pin<D>) -> ButtonRenderModel<D> {
    ButtonRenderModel {
        id: model.id.clone(),
        data: (pin.data)(&model.data),
        content: model.content.clone(),
        icon: model.icon.clone(),
        role: model.role,
        state: pin.state,
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
    use xgpui_luma::controls::checkbox::CheckboxData;

    #[test]
    fn a_pinned_model_replaces_only_the_state_and_data() {
        let model = ButtonRenderModel::<CheckboxData> {
            id: "box".into(),
            state: InteractionState { hovered: true, ..Default::default() },
            compact: true,
            ..Default::default()
        };
        model.radius_override.set(Some(3.));
        let pin = Pin::<CheckboxData> {
            state: InteractionState { focused: true, ..Default::default() },
            data: Arc::new(|live: &CheckboxData| CheckboxData {
                checked: true,
                progress: 1.0,
                icons: live.icons.clone(),
            }),
        };
        let drawn = pinned_model(&model, &pin);
        assert_eq!(drawn.state, pin.state);
        assert!(drawn.data.checked);
        assert_eq!(drawn.data.progress, 1.0, "settled, not animating");
        assert_eq!((drawn.id.as_ref(), drawn.compact, drawn.radius_override.get()), ("box", true, Some(3.)));
    }
}
