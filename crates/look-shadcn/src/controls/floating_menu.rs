//! Floating menu property mappings (shadcn / tweakcn):
//!
//! | Part          | Token                              |
//! |---------------|------------------------------------|
//! | Surface       | `popover` (fallback `card`)        |
//! | Foreground    | `popover-foreground`               |
//! | Border        | `border`                           |
//! | Item hover bg | `accent`                           |
//! | Item hover fg | `accent-foreground`                |
//! | Item disabled | `muted-foreground`               |

use crate::size::ShadcnSize;
use xgpui_luma::theme::ThemeMode;

use crate::mode::ShadcnModeTokens;

/// The Shadcn floating menu style at `size` (`style/floating_menu.lss`) for `mode`'s catalog: what menu templates
/// resolve and draw from (#433).
pub fn floating_menu_binding(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ShadcnSize,
) -> xgpui_luma::style::StyleBinding {
    debug_assert_eq!(theme_mode, mode.theme_mode, "the tokens' own mode draws the menu");
    let binding = xgpui_luma::style::StyleBinding::new(
        "floating_menu",
        xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()),
    );
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    binding
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use crate::size::ShadcnSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::floating_menu_binding;

    fn floating_menu_look(
        mode: &ShadcnModeTokens,
        theme_mode: xgpui_luma::theme::ThemeMode,
        size: ShadcnSize,
    ) -> xgpui_luma::controls::floating_menu::FloatingMenuLook {
        xgpui_luma::controls::floating_menu::styled_floating_menu_look(&floating_menu_binding(mode, theme_mode, size))
    }

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("shadow-md".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 2px 4px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn floating_menu_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Md);

        assert!(!look.shadow.is_empty());
    }

    #[test]
    fn floating_menu_uses_popover_surface_and_accent_item_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Md);

        crate::assert_close(look.background, catalog.color("popover").expect("popover"));
        crate::assert_close(look.foreground, catalog.color("popover-foreground").expect("popover-foreground"));
        crate::assert_close(look.item_hover_background, catalog.color("accent").expect("accent"));
        crate::assert_close(look.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn floating_menu_item_typography_scales_with_size() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let sm = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Sm);
        let md = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Md);
        let lg = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Lg);

        assert!((sm.item_typography.size - 12.0).abs() < f32::EPSILON);
        assert!((md.item_typography.size - 14.0).abs() < f32::EPSILON);
        assert!((lg.item_typography.size - 16.0).abs() < f32::EPSILON);
        assert!(sm.item_typography.size < md.item_typography.size);
        assert!(md.item_typography.size < lg.item_typography.size);
    }
}
