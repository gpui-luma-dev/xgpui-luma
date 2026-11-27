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
            Arc::new(move || {
                crate::popup_menu::floating_menu_binding_for(&menu_look, variant, tone, crate::RadixSize::Two)
            }),
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
    use xgpui_luma::style::StyleStates;
    use xgpui_luma::theme::InteractionState;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn menu_matches_popup_styling_in_both_modes_and_tones() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [ContextMenuVariant::Solid, ContextMenuVariant::Soft] {
                for tone in [Tone::Accent, Tone::Gray] {
                    let actual = xgpui_luma::controls::floating_menu::styled_floating_menu_look(
                        &context_menu_theme(&look, variant, tone).menu(),
                    );
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
        let target = |state: InteractionState| {
            xgpui_luma::style::PartRead::color(
                &theme.style().resolve_states(StyleStates::new(state), 1.0).root(),
                "border",
            )
        };
        let foreground = |state: InteractionState| {
            xgpui_luma::style::PartRead::color(
                &theme.style().resolve_states(StyleStates::new(state), 1.0).root(),
                "foreground",
            )
        };
        let focused = InteractionState { focused: true, ..Default::default() };
        assert_close(target(focused), look.resolve_role(SemanticRole::Focus).hsla());
        let disabled = InteractionState { focused: true, disabled: true, ..Default::default() };
        assert_close(target(disabled), look.resolve_role(SemanticRole::Border).hsla());
        assert_close(foreground(disabled), look.resolve_role(SemanticRole::MutedForeground).hsla());
    }
}
