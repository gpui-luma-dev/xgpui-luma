//! Shared selector row and panel styling for SDK selection controls.

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

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::style::PartRead;
    use xgpui_luma::style::ThemeSource;
    use xgpui_luma::theme::stylesheet::PopoverGeometry;
    use crate::{Accent, Gray};
    use xgpui_luma::controls::floating_menu::MenuStyle;
    use xgpui_luma::theme::ThemeMode;

    /// Radix selector popups are placed by the look's `popover.lss`, not the SDK's alone (#397).
    #[test]
    fn popups_are_placed_by_the_looks_popover_lss() {
        let look = Look::built_in();
        let want = PopoverGeometry::from_binding(&StyleBinding::new("popover", ThemeSource::Fixed(look.style_theme())));
        let panel = MenuStyle::resolve(&selector_items_panel_style(&look, PopupMenuVariant::Solid, Tone::Accent));
        assert_eq!(panel.window_margin(), want.window_margin);
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
                        let panel = MenuStyle::resolve(&selector_items_panel_style(&look, variant, tone));
                        let menu =
                            MenuStyle::resolve(&floating_menu_binding_for(&look, variant, tone, crate::RadixSize::Two));
                        assert_eq!(panel.panel().color("background"), menu.panel().color("background"));
                        assert_eq!(panel.row_colors().hover_background, menu.row_colors().hover_background);
                        assert_eq!(panel.row_colors().hover_foreground, menu.row_colors().hover_foreground);
                        assert_eq!(panel.row_colors().disabled_foreground, menu.row_colors().disabled_foreground);
                        assert_eq!(panel.disabled_opacity(), menu.disabled_opacity());
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_fork_and_selector_overrides_remain_local() {
        let original = Look::built_in();
        let fork = original.fork();
        let panel_of =
            |look: &Look| MenuStyle::resolve(&selector_items_panel_style(look, PopupMenuVariant::Solid, Tone::Accent));
        let before = panel_of(&original);
        fork.set_palettes(Accent::Red, Gray::Slate);
        let panel = panel_of(&fork);
        assert_ne!(panel.row_colors().hover_background, before.row_colors().hover_background);
        let unchanged = panel_of(&original);
        assert_eq!(unchanged.item().number("height"), before.item().number("height"));
        assert_eq!(unchanged.row_colors().hover_background, before.row_colors().hover_background);
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
