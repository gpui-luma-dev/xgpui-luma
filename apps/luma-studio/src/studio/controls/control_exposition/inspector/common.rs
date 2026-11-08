use gpui::hsla;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use xgpui_luma_look_shadcn::ShadcnButtonStyle;
use xgpui_luma_look_shadcn::ShadcnTextFieldStyle;

use super::box_model::BoxModelLayerColors;

pub fn choice_variant_style(variant_id: &str) -> ShadcnButtonStyle {
    match variant_id {
        "secondary" => ShadcnButtonStyle::Secondary,
        "content-only" => ShadcnButtonStyle::ContentOnly,
        _ => ShadcnButtonStyle::Primary,
    }
}

pub fn button_variant_style(variant_id: &str) -> ShadcnButtonStyle {
    match variant_id {
        "secondary" => ShadcnButtonStyle::Secondary,
        "outline" => ShadcnButtonStyle::Outline,
        "ghost" => ShadcnButtonStyle::Ghost,
        "content-only" => ShadcnButtonStyle::ContentOnly,
        _ => ShadcnButtonStyle::Primary,
    }
}

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

pub fn control_size(size_id: &str) -> ControlSize {
    match size_id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
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

pub fn value_flag(value_id: &str, true_id: &str) -> bool {
    value_id == true_id
}

pub fn inspector_elevation_applies(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn choice_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    variant_id != "content-only" && inspector_elevation_applies(state_id)
}

pub fn button_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    variant_id != "content-only" && inspector_elevation_applies(state_id)
}

pub fn textfield_variant_style(variant_id: &str) -> ShadcnTextFieldStyle {
    match variant_id {
        "primary" => ShadcnTextFieldStyle::Primary,
        "surface" => ShadcnTextFieldStyle::Surface,
        _ => ShadcnTextFieldStyle::Outline,
    }
}

pub fn textfield_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn textfield_state(state_id: &str) -> xgpui_luma::controls::textfield::TextFieldState {
    match state_id {
        "hover" => xgpui_luma::controls::textfield::TextFieldState {
            hovered: true,
            ..xgpui_luma::controls::textfield::TextFieldState::default()
        },
        "focus" => xgpui_luma::controls::textfield::TextFieldState {
            focused: true,
            focus_visible: true,
            ..xgpui_luma::controls::textfield::TextFieldState::default()
        },
        "invalid" => xgpui_luma::controls::textfield::TextFieldState {
            invalid: true,
            ..xgpui_luma::controls::textfield::TextFieldState::default()
        },
        _ => xgpui_luma::controls::textfield::TextFieldState::default(),
    }
}

pub fn textarea_state(state_id: &str) -> xgpui_luma::controls::textarea::TextAreaState {
    match state_id {
        "hover" => xgpui_luma::controls::textarea::TextAreaState {
            hovered: true,
            ..xgpui_luma::controls::textarea::TextAreaState::default()
        },
        "focus" => xgpui_luma::controls::textarea::TextAreaState {
            focused: true,
            focus_visible: true,
            ..xgpui_luma::controls::textarea::TextAreaState::default()
        },
        "invalid" => xgpui_luma::controls::textarea::TextAreaState {
            invalid: true,
            ..xgpui_luma::controls::textarea::TextAreaState::default()
        },
        _ => xgpui_luma::controls::textarea::TextAreaState::default(),
    }
}

pub fn textfield_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    !variant_id.is_empty() && textfield_enabled(state_id)
}

pub fn badge_variant(variant_id: &str) -> xgpui_luma_look_shadcn::BadgeVariant {
    match variant_id {
        "secondary" => xgpui_luma_look_shadcn::BadgeVariant::Secondary,
        "outline" => xgpui_luma_look_shadcn::BadgeVariant::Outline,
        "ghost" => xgpui_luma_look_shadcn::BadgeVariant::Ghost,
        _ => xgpui_luma_look_shadcn::BadgeVariant::Default,
    }
}

pub fn progress_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn stepper_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn scrollbar_style(variant_id: &str) -> xgpui_luma::controls::scrollbar::ScrollbarStyle {
    match variant_id {
        "soft" => xgpui_luma::controls::scrollbar::ScrollbarStyle::Soft,
        _ => xgpui_luma::controls::scrollbar::ScrollbarStyle::Ghost,
    }
}

pub fn pager_style(variant_id: &str) -> xgpui_luma::controls::pager::PagerStyle {
    match variant_id {
        "minimal" => xgpui_luma::controls::pager::PagerStyle::Minimal,
        "minimal-edge" => xgpui_luma::controls::pager::PagerStyle::MinimalEdge,
        _ => xgpui_luma::controls::pager::PagerStyle::Numeric,
    }
}

pub fn pager_shell_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn pager_button_role(variant_id: &str, value_id: &str) -> xgpui_luma::controls::button_family::ButtonFamilyRole {
    match variant_id {
        "page" => {
            xgpui_luma::controls::button_family::ButtonFamilyRole::Toggle { selected: value_flag(value_id, "selected") }
        }
        _ => xgpui_luma::controls::button_family::ButtonFamilyRole::Icon,
    }
}

pub fn popup_menu_trigger_style(variant_id: &str) -> xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle {
    match variant_id {
        "primary" => xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Primary,
        "secondary" => xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Secondary,
        "ghost" => xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Ghost,
        _ => xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline,
    }
}

pub fn toolbar_variant(variant_id: &str) -> xgpui_luma::controls::toolbar::ToolbarVariant {
    match variant_id {
        "ghost" => xgpui_luma::controls::toolbar::ToolbarVariant::Ghost,
        _ => xgpui_luma::controls::toolbar::ToolbarVariant::Outline,
    }
}

pub fn listbox_row_state(state_id: &str) -> InteractionState {
    match state_id {
        "keyboard-active" => InteractionState { focused: true, ..InteractionState::default() },
        other => interaction_state(other),
    }
}

pub fn table_row_selected(value_id: &str) -> bool {
    value_id == "selected"
}

pub fn accordion_content_expanded(state_id: &str) -> bool {
    state_id == "expanded"
}

pub fn tabs_active(variant_id: &str) -> bool {
    variant_id == "active"
}

pub fn sidebar_item_selected(value_id: &str) -> bool {
    value_id == "selected"
}

pub fn resize_handle_size(size_id: &str) -> xgpui_luma::controls::resizable_panels::ResizeHandleSize {
    match size_id {
        "sm" => xgpui_luma::controls::resizable_panels::ResizeHandleSize::Sm,
        "lg" => xgpui_luma::controls::resizable_panels::ResizeHandleSize::Lg,
        _ => xgpui_luma::controls::resizable_panels::ResizeHandleSize::Md,
    }
}

#[cfg(test)]
mod tests {
    use super::{button_elevation_applies, textfield_elevation_applies};

    #[test]
    fn button_elevation_inspection_is_not_limited_to_outline() {
        assert!(button_elevation_applies("primary", "default"));
        assert!(button_elevation_applies("secondary", "default"));
        assert!(button_elevation_applies("outline", "default"));
        assert!(!button_elevation_applies("content-only", "default"));
    }

    #[test]
    fn textfield_elevation_inspection_is_not_limited_to_primary() {
        assert!(textfield_elevation_applies("outline", "default"));
        assert!(textfield_elevation_applies("primary", "default"));
        assert!(textfield_elevation_applies("surface", "default"));
        assert!(!textfield_elevation_applies("outline", "disabled"));
    }
}
