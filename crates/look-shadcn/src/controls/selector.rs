//! Selector property mappings — outline/ghost command-button trigger + accent item panel.

#[cfg(feature = "inspect")]
use xgpui_luma::theme::{InteractionState, StandardBoxScale};

#[cfg(feature = "inspect")]
use crate::look_context::LookContext;
#[cfg(feature = "inspect")]
use crate::mode::ShadcnModeTokens;

#[cfg(feature = "inspect")]
use crate::size::ShadcnSize;

#[cfg(feature = "inspect")]
pub(crate) fn selector_geometry(
    mode: &ShadcnModeTokens,
    size: ShadcnSize,
    scale: &StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedSelectorGeometry {
    let icon = super::button::button_box_scale(
        &LookContext::new(mode, mode.theme_mode, InteractionState::default()),
        mode.stylesheet(),
        size,
        1.0,
    )
    .icon_size;
    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
        .selector
        .authored_geometry_for(size.key());
    let recipe = mode.stylesheet().common.selector.authored_geometry_for(size.key()).with_fallback(&baseline);
    mode.stylesheet().common.selector.inspect_geometry_for(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::SelectorGeometry {
            height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            gap: scale.gap,
            icon_size: icon,
            font_size: typography.size,
            line_height: typography.line_height,
            radius: scale.radius,
            menu_offset_y: recipe.gap.unwrap_or(scale.gap) * recipe.menu_offset_scale.unwrap_or_default(),
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use gpui::Hsla;
    use xgpui_luma::controls::selector::{SelectorLook, SelectorTriggerStyle, SelectorVisualState};
    use xgpui_luma::theme::{InteractionState, StandardBoxScale, ThemeMode};

    use crate::ShadcnLook;

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
    }

    fn look() -> ShadcnLook {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
        look.set_mode(ThemeMode::Light);
        look
    }

    fn css(look: &ShadcnLook, name: &str) -> Hsla {
        look.mode_tokens().catalog.color(name).unwrap()
    }

    fn trigger(look: &ShadcnLook, style: SelectorTriggerStyle, state: SelectorVisualState, flat: bool) -> SelectorLook {
        let theme = look.selector_theme();
        theme.resolve_visual_look(style, state, &StandardBoxScale::compute(&theme.metrics(), 1.0), flat)
    }

    fn with(interaction: InteractionState) -> SelectorVisualState {
        SelectorVisualState { interaction, ..Default::default() }
    }

    #[test]
    fn outline_is_the_outline_button_with_its_shadow_and_a_muted_chevron() {
        let look = look();
        let outline = trigger(&look, SelectorTriggerStyle::Outline, Default::default(), false);
        assert!(close(outline.trigger_background, css(&look, "background")));
        assert!(close(outline.trigger_border.unwrap(), css(&look, "border")));
        assert!(close(outline.trigger_icon, css(&look, "muted-foreground")));
        assert!(outline.trigger_shadow.as_ref().is_some_and(|layers| !layers.is_empty()));
        assert!(trigger(&look, SelectorTriggerStyle::Outline, Default::default(), true).trigger_shadow.is_none());
        assert_eq!(outline.trigger_padding_x, 14.0);
    }

    #[test]
    fn ghost_has_no_face_edge_or_shadow() {
        let look = look();
        let ghost = trigger(&look, SelectorTriggerStyle::Ghost, Default::default(), false);
        assert_eq!(ghost.trigger_background.a, 0.0);
        assert!(ghost.trigger_border.is_none_or(|border| border.a == 0.0));
        assert!(ghost.trigger_shadow.is_none());
    }

    #[test]
    fn open_looks_hovered_but_disabled_still_wins() {
        let look = look();
        let style = SelectorTriggerStyle::Outline;
        let open = trigger(&look, style, SelectorVisualState { open: true, ..Default::default() }, false);
        let hover = trigger(&look, style, with(InteractionState { hovered: true, ..Default::default() }), false);
        assert_eq!(open.trigger_background, hover.trigger_background);
        let disabled_state = InteractionState { disabled: true, ..Default::default() };
        let disabled_open = trigger(
            &look,
            style,
            SelectorVisualState { open: true, interaction: disabled_state, ..Default::default() },
            false,
        );
        let disabled = trigger(&look, style, with(disabled_state), false);
        assert_eq!(disabled_open.trigger_background, disabled.trigger_background);
        assert_ne!(disabled.trigger_background, hover.trigger_background);
    }

    #[test]
    fn invalid_marks_the_edge_and_ring_destructive() {
        let look = look();
        let invalid = trigger(
            &look,
            SelectorTriggerStyle::Outline,
            SelectorVisualState { invalid: true, ..Default::default() },
            false,
        );
        assert!(close(invalid.trigger_border.unwrap(), css(&look, "destructive")));
        assert!(close(invalid.trigger_focus_border, css(&look, "destructive")));
    }

    #[test]
    fn sizes_scale_the_label_font() {
        let look = look();
        let size = |s| {
            trigger(&look.with_size(s), SelectorTriggerStyle::Outline, Default::default(), false)
                .trigger_typography
                .size
        };
        assert_eq!(
            (size(crate::ShadcnSize::Sm), size(crate::ShadcnSize::Md), size(crate::ShadcnSize::Lg)),
            (12.0, 14.0, 16.0)
        );
    }
}
