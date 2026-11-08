//! Shared selector row and panel styling for SDK selection controls.

use xgpui_luma::controls::selector_list::SelectorItemsPanelLook;

use crate::{Look, PopupMenuVariant, Tone, floating_menu_look_for};

/// Resolve Radix row and panel styling for the SDK selector-items template.
///
/// Reuses menu colors and geometry, then applies `selector_list` stylesheet
/// overrides using the existing Radix size key (`"2"`). Pair with the SDK's
/// `default_selector_items_template`; state, input, and callbacks stay in the SDK.
/// Resolve again during render after changing the supplied Look, and notify the
/// owner. Explicit and forked Looks are used directly, without ambient lookup.
///
/// ```
/// use xgpui_luma_look_radix::{Look, PopupMenuVariant, Tone, selector_items_panel_look};
/// let look = Look::built_in();
/// let panel = selector_items_panel_look(&look, PopupMenuVariant::Soft, Tone::Accent);
/// assert!(panel.item_height > 0.0);
/// ```
pub fn selector_items_panel_look(look: &Look, variant: PopupMenuVariant, tone: Tone) -> SelectorItemsPanelLook {
    selector_items_panel_look_for(look, variant, tone, xgpui_luma::theme::ControlSize::Md)
}

/// Size-aware selector row/panel chrome using Radix stylesheet keys `1`–`3`.
/// Resolve again after Look changes, as with [`selector_items_panel_look`].
pub fn selector_items_panel_look_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: xgpui_luma::theme::ControlSize,
) -> SelectorItemsPanelLook {
    SelectorItemsPanelLook::from_menu(
        floating_menu_look_for(look, variant, tone, size),
        &look.metrics(),
        &look.common_stylesheet(),
        crate::look::sdk_size_key(size),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Accent, Gray, floating_menu_look};
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn shared_chrome_tracks_mode_palette_variant_and_tone() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for accent in [Accent::Blue, Accent::Red] {
                look.set_palettes(accent, Gray::Slate);
                for variant in [PopupMenuVariant::Solid, PopupMenuVariant::Soft] {
                    for tone in [Tone::Accent, Tone::Gray] {
                        let panel = selector_items_panel_look(&look, variant, tone);
                        let menu = floating_menu_look(&look, variant, tone);
                        assert_eq!(panel.background, menu.background);
                        assert_eq!(panel.item_hover_background, menu.item_hover_background);
                        assert_eq!(panel.item_hover_foreground, menu.item_hover_foreground);
                        assert_eq!(panel.item_disabled_foreground, menu.item_disabled_foreground);
                        assert_eq!(panel.disabled_opacity, menu.disabled_opacity);
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_fork_and_selector_overrides_remain_local() {
        let original = Look::built_in();
        let fork = original.fork();
        let before = selector_items_panel_look(&original, PopupMenuVariant::Solid, Tone::Accent);
        fork.set_palettes(Accent::Red, Gray::Slate);
        let mut stylesheet = fork.common_stylesheet();
        stylesheet.selector_list.geometry.item_height = Some(53.0);
        stylesheet.selector_list.geometry.disabled_opacity = Some(0.3);
        fork.set_common_stylesheet(stylesheet);
        let panel = selector_items_panel_look(&fork, PopupMenuVariant::Solid, Tone::Accent);
        assert_eq!(panel.item_height, 53.0);
        assert_eq!(panel.disabled_opacity, 0.3);
        assert_ne!(panel.item_hover_background, before.item_hover_background);
        let unchanged = selector_items_panel_look(&original, PopupMenuVariant::Solid, Tone::Accent);
        assert_eq!(unchanged.item_height, before.item_height);
        assert_eq!(unchanged.item_hover_background, before.item_hover_background);
    }
}
