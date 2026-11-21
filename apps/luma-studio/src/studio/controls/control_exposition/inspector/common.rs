use gpui::hsla;
use xgpui_luma::theme::{InteractionState, ThemeMode};

use super::box_model::BoxModelLayerColors;

pub fn choice_indicator_only(part_id: &str) -> bool {
    part_id == "indicator-only"
}

pub fn interaction_state(state_id: &str) -> InteractionState {
    match state_id {
        "hover" => InteractionState { hovered: true, ..InteractionState::default() },
        "focused" => InteractionState { focused: true, ..InteractionState::default() },
        "pressed" => InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        "disabled" => InteractionState { disabled: true, ..InteractionState::default() },
        _ => InteractionState::default(),
    }
}

pub fn neutral_box_model_colors(mode: ThemeMode) -> BoxModelLayerColors {
    match mode {
        ThemeMode::Dark => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.16, 1.0),
            border: hsla(0.0, 0.0, 0.24, 1.0),
            padding: hsla(0.0, 0.0, 0.32, 1.0),
            content: hsla(0.0, 0.0, 0.40, 1.0),
            stroke: hsla(0.0, 0.0, 0.88, 0.92),
            highlight: hsla(0.0, 0.0, 0.96, 1.0),
        },
        ThemeMode::Light => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.94, 1.0),
            border: hsla(0.0, 0.0, 0.86, 1.0),
            padding: hsla(0.0, 0.0, 0.78, 1.0),
            content: hsla(0.0, 0.0, 0.70, 1.0),
            stroke: hsla(0.0, 0.0, 0.18, 0.88),
            highlight: hsla(0.0, 0.0, 0.08, 1.0),
        },
    }
}

pub fn neutral_box_model_label_color(mode: ThemeMode) -> gpui::Hsla {
    match mode {
        ThemeMode::Dark => hsla(0.0, 0.0, 0.94, 0.96),
        ThemeMode::Light => hsla(0.0, 0.0, 0.14, 0.96),
    }
}
