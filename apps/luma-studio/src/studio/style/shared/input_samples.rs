use xgpui_luma::theme::InteractionState;

#[derive(Clone, Copy)]
pub(crate) struct InputInteractionSample {
    pub id: &'static str,
    pub label: &'static str,
    pub state: InteractionState,
}

pub(crate) fn input_interaction_samples() -> [InputInteractionSample; 5] {
    [
        InputInteractionSample { id: "default", label: "Standard", state: InteractionState::default() },
        InputInteractionSample {
            id: "hover",
            label: "Hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "focus",
            label: "Focus",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "active",
            label: "Active",
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "disabled",
            label: "Disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ]
}
