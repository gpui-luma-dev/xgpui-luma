//! Radix popup menus (Copy / Actions menus), drawn from `.lss`.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use std::sync::Arc;

use xgpui_luma::controls::floating_menu::FloatingMenuLook;
use xgpui_luma::controls::popup_menu::{PopupMenuTemplate, PopupMenuTheme, StyledPopupMenuTheme, ThemedPopupMenuTemplate};

use crate::look::Look;
use crate::tone::Tone;

/// Radix Themes menu content variants, as carried by the SDK popup menu control.
///
/// The variant only governs the highlighted item: the panel itself is identical.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupMenuVariant {
    /// Highlighted item takes the solid accent (`accent-9` on `accent-contrast`).
    #[default]
    Solid,
    /// Highlighted item takes a soft accent tint (`accent-4`), text unchanged.
    Soft,
}

/// Resolve shared Radix menu panel, row, and separator styling for SDK composition.
///
/// Pass the result to the SDK floating-menu renderer. This is a resolved snapshot:
/// resolve again during render after changing the supplied Look, and notify the owner.
/// Explicit and forked Looks are used directly; no ambient Look is consulted.
/// Uses the existing Radix menu size (`"2"`) and its shared stylesheet overrides.
#[deprecated(note = "menus draw from `floating_menu_binding_for`; read it through `MenuStyle` (#433)")]
pub fn floating_menu_look(look: &Look, variant: PopupMenuVariant, tone: Tone) -> FloatingMenuLook {
    floating_menu_look_for(look, variant, tone, crate::RadixSize::Two)
}

/// Size-aware shared menu chrome. Size keys are Radix `1`, `2`, and `3`.
/// Like [`floating_menu_look`], this resolves a snapshot from the supplied Look.
/// The size-aware Radix menu style (the `floating_menu` control): what menu templates resolve and draw from (#433).
pub fn floating_menu_binding_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: crate::RadixSize,
) -> xgpui_luma::style::StyleBinding {
    let binding = xgpui_luma::style::StyleBinding::new("floating_menu", crate::textfield::live_source(look));
    let binding = match variant {
        PopupMenuVariant::Solid => binding,
        PopupMenuVariant::Soft => binding.variant("soft"),
    };
    let binding = if tone == Tone::Gray {
        binding.attr("tone", "gray")
    } else {
        binding
    };
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    binding
}

/// Size-aware shared menu chrome as a look snapshot, from [`floating_menu_binding_for`].
#[deprecated(note = "menus draw from `floating_menu_binding_for`; read it through `MenuStyle` (#433)")]
pub fn floating_menu_look_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: crate::RadixSize,
) -> FloatingMenuLook {
    xgpui_luma::controls::floating_menu::styled_floating_menu_look(&floating_menu_binding_for(
        look, variant, tone, size,
    ))
}

pub fn popup_menu_theme(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTheme> {
    styled_popup_menu_theme(look, variant, tone, look.size())
}

/// The Radix popup trigger drawn from `.lss` (`style/popup_menu.lss`) at the look's size, with its floating
/// menu at `menu_size`. Follows the look's later edits.
fn styled_popup_menu_theme(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    menu_size: crate::RadixSize,
) -> Arc<dyn PopupMenuTheme> {
    let size = look.size();
    let menu_look = look.clone();
    Arc::new(
        StyledPopupMenuTheme::new(
            crate::textfield::live_source(look),
            Arc::new(move || floating_menu_binding_for(&menu_look, variant, tone, menu_size)),
        )
        .with(|binding| {
            let binding = if tone == Tone::Gray {
                binding.attr("tone", "gray")
            } else {
                binding
            };
            if size != crate::RadixSize::Two {
                binding.attr("size", size.key())
            } else {
                binding
            }
        }),
    )
}

pub fn popup_menu_template(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(look, variant, tone)))
}

/// Popup template whose trigger uses the look handle's size and whose menu uses `menu_size`.
pub fn popup_menu_template_with_menu_size(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    menu_size: crate::RadixSize,
) -> Arc<dyn PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(styled_popup_menu_theme(look, variant, tone, menu_size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_group::assert_close;
    use crate::semantic::SemanticRole;
    use xgpui_luma::controls::popup_menu::{PopupMenuTriggerStyle, resolve_popup_menu_trigger};
    use xgpui_luma::style::PartRead as _;
    use xgpui_luma::theme::InteractionState;

    fn trigger_background(look: &Look, tone: Tone) -> gpui::Hsla {
        let theme = popup_menu_theme(look, PopupMenuVariant::default(), tone);
        resolve_popup_menu_trigger(
            &theme.style(),
            PopupMenuTriggerStyle::Primary,
            InteractionState::default(),
            false,
            1.0,
        )
        .root()
        .color("background")
    }

    #[test]
    fn primary_trigger_uses_accent() {
        let look = Look::built_in();
        assert_close(trigger_background(&look, Tone::Accent), look.resolve_role(SemanticRole::Primary).hsla());
    }

    #[test]
    fn gray_tone_locks_the_trigger_to_the_neutral_scale() {
        let look = Look::built_in();
        let gray = trigger_background(&look, Tone::Gray);

        assert_close(gray, Tone::Gray.step(&look, 9));
        assert_ne!(gray, trigger_background(&look, Tone::Accent));
    }

    #[test]
    fn solid_variant_highlights_items_with_the_solid_accent() {
        let look = Look::built_in();
        let menu = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);

        crate::control_group::assert_close(menu.item_hover_background, look.resolve_role(SemanticRole::Primary).hsla());
        crate::control_group::assert_close(
            menu.item_hover_foreground,
            look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
        );
    }

    #[test]
    fn soft_variant_tints_items_and_keeps_the_panel_text() {
        let look = Look::built_in();
        let solid = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let soft = floating_menu_look(&look, PopupMenuVariant::Soft, Tone::Accent);

        crate::control_group::assert_close(
            soft.item_hover_foreground,
            look.resolve_role(SemanticRole::Foreground).hsla(),
        );
        assert!(soft.item_hover_background.l > solid.item_hover_background.l);
        assert_eq!(soft.background, solid.background);
    }

    #[test]
    fn gray_tone_keeps_the_panel_and_only_swaps_the_highlight() {
        let look = Look::built_in();
        let accent = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let gray = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Gray);

        crate::control_group::assert_close(gray.item_hover_background, Tone::Gray.step(&look, 9));
        assert_ne!(gray.item_hover_background, accent.item_hover_background);
        assert_eq!(gray.background, accent.background);
    }
}
