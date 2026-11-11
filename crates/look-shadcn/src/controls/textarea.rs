//! Text area — surface/filled/soft variants share the text field token resolver.

use xgpui_luma::controls::textarea::{TextAreaLook, TextAreaPalette, TextAreaState, compose_textarea_look};
use xgpui_luma::controls::textfield::TextFieldState;
use xgpui_luma::theme::{ControlSize, StandardBoxScale, ThemeMode};

use super::textfield::{ShadcnTextFieldStyle, textfield_palette_base_for_size};
use crate::mode::ShadcnModeTokens;

pub fn textarea_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
) -> TextAreaPalette {
    textarea_palette_for_size(mode, theme_mode, style, state, enabled, ControlSize::Md)
}

pub fn textarea_palette_for_size(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
    size: ControlSize,
) -> TextAreaPalette {
    let mut palette = textarea_from_textfield(textfield_palette_base_for_size(
        mode,
        theme_mode,
        style,
        textfield_state_from(state),
        enabled,
        size,
    ));
    let scale = StandardBoxScale::compute(size, &mode.metrics, 1.0);
    let geometry = textarea_geometry(mode, size, &scale, &palette.typography);
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut palette.typography,
        &geometry.font_size,
        &geometry.line_height,
    );
    palette
}

pub fn textarea_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
    size: ControlSize,
    scale: &StandardBoxScale,
) -> TextAreaLook {
    let palette = textarea_palette_for_size(mode, theme_mode, style, state, enabled, size);
    let geometry = textarea_geometry(mode, size, scale, &palette.typography);
    let mut scale = *scale;
    scale.height = geometry.min_height.value_px;
    scale.padding_x = geometry.padding_x.value_px;
    scale.padding_y = geometry.padding_y.value_px;
    let mut look = compose_textarea_look(&palette, &scale, mode.metrics.border_width.default);
    look.helper_geometry = mode.stylesheet().common.shared_geometry();
    look.caret_width = geometry.caret_width.value_px;
    look.caret_height_extra = geometry.caret_height_extra.value_px;
    look.disabled_opacity = geometry.disabled_opacity.value_px;
    look
}

pub(crate) fn textarea_geometry(
    mode: &ShadcnModeTokens,
    size: ControlSize,
    scale: &StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedTextAreaGeometry {
    mode.stylesheet().common.textarea.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::TextAreaGeometry {
            min_height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            font_size: typography.size,
            line_height: typography.line_height,
            ..Default::default()
        },
    )
}

fn textfield_state_from(state: TextAreaState) -> TextFieldState {
    TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focus_visible,
        invalid: state.invalid,
        cursor: state.cursor,
        selection_anchor: state.selection_anchor,
        preferred_column: state.preferred_column,
    }
}

fn textarea_from_textfield(textfield: xgpui_luma::controls::textfield::TextFieldPalette) -> TextAreaPalette {
    TextAreaPalette {
        focus_border: textfield.focus_border,
        background: textfield.background,
        foreground: textfield.foreground,
        border: textfield.border,
        placeholder: textfield.placeholder,
        selection_background: textfield.selection_background,
        selection_foreground: textfield.selection_foreground,
        caret: textfield.caret,
        shadow: textfield.shadow.clone(),
        typography: textfield.typography,
        font_family: textfield.font_family.to_string(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::controls::textfield::TextFieldState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::controls::textfield::{ShadcnTextFieldStyle, textfield_palette};
    use super::{textarea_from_textfield, textarea_palette};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("card".into(), "oklch(0.98 0.02 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn textarea_uses_same_surface_tokens_as_textfield() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let textfield =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);
        let textarea = textarea_from_textfield(textfield);

        assert_eq!(textarea.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(textarea.border, catalog.color("border").expect("border"));
        assert_eq!(textarea.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(textarea.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
        assert!(textarea.shadow.is_none());
    }

    #[test]
    fn primary_textarea_uses_background_fill_and_shadow() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let textarea = textarea_palette(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Primary,
            xgpui_luma::controls::textarea::TextAreaState::default(),
            true,
        );

        assert_eq!(textarea.background, catalog.color("background").expect("background"));
        assert_eq!(textarea.border, catalog.color("border").expect("border"));
        assert!(textarea.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn surface_textarea_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let textfield =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);
        let textarea = textarea_from_textfield(textfield);

        assert_eq!(textarea.background, catalog.color("muted").expect("muted"));
        assert_eq!(textarea.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert!(textarea.shadow.is_none());
    }
}
