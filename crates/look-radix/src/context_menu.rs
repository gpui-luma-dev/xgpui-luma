//! Radix context menus use the same panel and item treatments as popup menus.
use std::sync::Arc;
use xgpui_luma::controls::context_menu::{
    ContextMenuTemplate, ContextMenuTheme, StyledContextMenuTheme, ThemedContextMenuTemplate,
};
use crate::{Look, Tone};

/// Solid or Soft highlighted items, shared with Radix popup menus.
pub type ContextMenuVariant = crate::PopupMenuVariant;
/// The Radix context-menu target drawn from `.lss` (`style/context_menu.lss`); its menu is the floating
/// menu for `variant` and `tone`.
pub fn context_menu_theme(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTheme> {
    let menu_look = look.clone();
    Arc::new(
        StyledContextMenuTheme::new(
            crate::textfield::live_source(look),
            Arc::new(move || crate::popup_menu::floating_menu_look(&menu_look, variant, tone)),
        )
        .with(|binding| {
            if tone == Tone::Gray {
                binding.attr("tone", "gray")
            } else {
                binding
            }
        }),
    )
}
pub fn context_menu_template(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(look, variant, tone)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SemanticRole;
    use crate::control_group::assert_close;
    use xgpui_luma::theme::InteractionState;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn menu_matches_popup_styling_in_both_modes_and_tones() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [ContextMenuVariant::Solid, ContextMenuVariant::Soft] {
                for tone in [Tone::Accent, Tone::Gray] {
                    let actual =
                        context_menu_theme(&look, variant, tone).resolve(InteractionState::default()).floating_menu;
                    let expected = crate::popup_menu::floating_menu_look(&look, variant, tone);
                    assert_eq!(actual.background, expected.background);
                    assert_eq!(actual.border, expected.border);
                    assert_eq!(actual.item_hover_background, expected.item_hover_background);
                    assert_eq!(actual.item_hover_foreground, expected.item_hover_foreground);
                }
            }
        }
    }
    #[test]
    fn focus_and_disabled_target_follow_radix_roles() {
        let look = Look::built_in();
        let theme = context_menu_theme(&look, ContextMenuVariant::Solid, Tone::Accent);
        let focused = theme.resolve(InteractionState { focused: true, ..Default::default() });
        assert_close(focused.target_border, look.resolve_role(SemanticRole::Focus).hsla());
        let disabled = theme.resolve(InteractionState { focused: true, disabled: true, ..Default::default() });
        assert_close(disabled.target_border, look.resolve_role(SemanticRole::Border).hsla());
        assert_close(disabled.target_foreground, look.resolve_role(SemanticRole::MutedForeground).hsla());
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_floating_menu_stylesheet() -> xgpui_luma::theme::stylesheet::FloatingMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    FloatingMenuStylesheet {
        geometry: FloatingMenuGeometryStylesheet {
            separator_inset: Some(8.0),
            font_size: Some(14.0),
            line_height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_context_menu_stylesheet() -> xgpui_luma::theme::stylesheet::ContextMenuStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ContextMenuStylesheet {
        geometry: ContextMenuGeometryStylesheet {
            min_width: Some(200.0),
            font_size: Some(14.0),
            line_height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    }
}
