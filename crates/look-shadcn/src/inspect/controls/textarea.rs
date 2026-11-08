//! Inspect metadata for `textarea` (delegates to textfield).

use xgpui_luma::controls::textarea::TextAreaState;
use xgpui_luma::theme::{ControlSize, ThemeMode};
use crate::{ShadcnModeTokens, ShadcnTextFieldStyle};

use super::textfield::{TextFieldInspectMetrics, TextFieldInspectPalette, inspect_textfield_color_palette};

fn textfield_state_from(state: TextAreaState) -> xgpui_luma::controls::textfield::TextFieldState {
    xgpui_luma::controls::textfield::TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focus_visible,
        invalid: state.invalid,
        cursor: state.cursor,
        selection_anchor: state.selection_anchor,
        preferred_column: state.preferred_column,
    }
}

pub fn inspect_textarea_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
) -> TextFieldInspectPalette {
    inspect_textfield_color_palette(mode, theme_mode, style, textfield_state_from(state), enabled)
}

pub fn inspect_textarea_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TextFieldInspectMetrics {
    let mut table = crate::tables::metrics::resolve_textfield_metrics(mode, theme_mode, size);
    // Textarea owns geometry independently of configured textfield dimensions.
    let scale = xgpui_luma::theme::StandardBoxScale::compute(size, &mode.metrics, 1.0);
    let geometry = crate::controls::textarea::textarea_geometry(mode, size, &scale, &mode.typography.text.body);
    table.min_height = crate::tables::metrics::helpers::prefer_shared_metric(
        geometry.min_height,
        crate::tables::metrics::helpers::scaffold_control_metric(
            crate::tables::metrics::helpers::control_size_key(size),
            "control_height",
            scale.height,
        ),
    );
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(
        geometry.padding_x,
        crate::tables::metrics::helpers::spacing_control_metric(
            &mode.catalog,
            size,
            crate::catalog::SpacingField::PaddingX,
            scale.padding_x,
        ),
    );
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(
        geometry.padding_y,
        crate::tables::metrics::helpers::spacing_control_metric(
            &mode.catalog,
            size,
            crate::catalog::SpacingField::PaddingY,
            scale.padding_y,
        ),
    );
    table.icon_size = crate::tables::metrics::helpers::scaffold_control_metric(
        crate::tables::metrics::helpers::control_size_key(size),
        "icon_size",
        scale.icon_size,
    );
    table.into()
}
