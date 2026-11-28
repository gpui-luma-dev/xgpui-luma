//! Shared selector row and panel styling for SDK selection controls.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use xgpui_luma::controls::selector_list::SelectorItemsPanelLook;
use xgpui_luma::style::StyleBinding;

use crate::{Look, PopupMenuVariant, Tone, floating_menu_binding_for};

/// The Radix selector popup panel's style at the look's default size (#433): the `floating_menu` binding the
/// SDK's panel templates resolve and draw from, placed by the look's `popover.lss`. Follows the look's later edits.
///
/// ```
/// use xgpui_luma_look_radix::{Look, PopupMenuVariant, Tone, selector_items_panel_style};
/// let look = Look::built_in();
/// let panel = selector_items_panel_style(&look, PopupMenuVariant::Soft, Tone::Accent);
/// # let _ = panel;
/// ```
pub fn selector_items_panel_style(look: &Look, variant: PopupMenuVariant, tone: Tone) -> StyleBinding {
    selector_items_panel_style_for(look, variant, tone, crate::RadixSize::Two)
}

/// [`selector_items_panel_style`] at Radix size `1`–`3`.
pub fn selector_items_panel_style_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: crate::RadixSize,
) -> StyleBinding {
    floating_menu_binding_for(look, variant, tone, size)
}

/// Resolve Radix row and panel styling for the SDK selector-items template.
///
/// Reuses menu colors and geometry, placed by the look's `popover.lss`, then applies
/// `selector_list` stylesheet overrides using the existing Radix size key (`"2"`). Pair with the SDK's
/// `default_selector_items_template`; state, input, and callbacks stay in the SDK.
/// Resolve again during render after changing the supplied Look, and notify the
/// owner. Explicit and forked Looks are used directly, without ambient lookup.
///
#[deprecated(note = "use `selector_items_panel_style`; the SDK's panel templates draw from the style (#433)")]
pub fn selector_items_panel_look(look: &Look, variant: PopupMenuVariant, tone: Tone) -> SelectorItemsPanelLook {
    selector_items_panel_look_for(look, variant, tone, crate::RadixSize::Two)
}

/// Size-aware selector row/panel chrome using Radix stylesheet keys `1`–`3`.
/// Resolve again after Look changes, as with [`selector_items_panel_look`].
#[deprecated(note = "use `selector_items_panel_style_for`; the SDK's panel templates draw from the style (#433)")]
pub fn selector_items_panel_look_for(
    look: &Look,
    variant: PopupMenuVariant,
    tone: Tone,
    size: crate::RadixSize,
) -> SelectorItemsPanelLook {
    SelectorItemsPanelLook::from_style(&selector_items_panel_style_for(look, variant, tone, size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::style::ThemeSource;
    use xgpui_luma::theme::stylesheet::PopoverGeometry;
    use crate::{Accent, Gray, floating_menu_look};
    use xgpui_luma::theme::ThemeMode;

    /// Radix selector popups are placed by the look's `popover.lss`, not the SDK's alone (#397).
    #[test]
    fn popups_are_placed_by_the_looks_popover_lss() {
        let look = Look::built_in();
        let want = PopoverGeometry::from_binding(&StyleBinding::new("popover", ThemeSource::Fixed(look.style_theme())));
        let panel = selector_items_panel_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        assert_eq!((panel.popup_offset_y, panel.window_margin), (want.panel_offset_y, want.window_margin));
    }

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
                        assert_eq!(panel.item_disabled_opacity, menu.item_disabled_opacity);
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
        let panel = selector_items_panel_look(&fork, PopupMenuVariant::Solid, Tone::Accent);
        assert_ne!(panel.item_hover_background, before.item_hover_background);
        let unchanged = selector_items_panel_look(&original, PopupMenuVariant::Solid, Tone::Accent);
        assert_eq!(unchanged.item_height, before.item_height);
        assert_eq!(unchanged.item_hover_background, before.item_hover_background);
    }
}

/// The panel style draws the border width the old look took from the size's metrics, at every size (#433).
#[cfg(test)]
mod parity_tests {
    use super::*;
    use xgpui_luma::style::PartRead;

    #[test]
    fn the_style_keeps_each_sizes_border_width() {
        let look = Look::built_in();
        for size in [crate::RadixSize::One, crate::RadixSize::Two, crate::RadixSize::Three] {
            let style = selector_items_panel_style_for(&look, PopupMenuVariant::Solid, Tone::Accent, size);
            let border_width = style.resolve(&[], 1.0).root().number("border_width");
            assert_eq!(border_width, look.metrics_for(size).border_width, "{size:?}");
        }
    }
}
