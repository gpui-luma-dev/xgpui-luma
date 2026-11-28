//! Stepper — accent track + accent-foreground fill (same color elements as progress).

use crate::size::ShadcnSize;

use crate::mode::ShadcnModeTokens;

/// The Shadcn stepper's style at `size` (`style/stepper.lss`) for `mode`'s catalog (#433).
pub fn stepper_style(mode: &ShadcnModeTokens, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
    let binding =
        xgpui_luma::style::StyleBinding::new("stepper", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    binding
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;

    use super::stepper_style;
    use xgpui_luma::controls::stepper::resolve_stepper;
    use xgpui_luma::style::PartRead;

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn stepper_uses_accent_track_and_accent_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let resolved = resolve_stepper(&stepper_style(&mode, crate::ShadcnSize::Md), true);
        let look = resolved.root();

        crate::assert_close(
            look.color("complete_background"),
            catalog.color("accent-foreground").expect("accent-foreground"),
        );
        crate::assert_close(look.color("complete_foreground"), catalog.color("accent").expect("accent"));
        crate::assert_close(look.color("track_active"), catalog.color("accent-foreground").expect("accent-foreground"));
        crate::assert_close(look.color("track_muted"), catalog.color("accent").expect("accent"));
    }

    #[test]
    fn disabled_stepper_complete_badge_has_checkmark_contrast() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let resolved = resolve_stepper(&stepper_style(&mode, crate::ShadcnSize::Md), false);
        let look = resolved.root();

        crate::assert_close(look.color("complete_background"), catalog.color("muted").expect("muted"));
        crate::assert_close(
            look.color("complete_foreground"),
            catalog.color("muted-foreground").expect("muted-foreground"),
        );
        assert_ne!(look.color("complete_background"), look.color("complete_foreground"));
    }

    #[test]
    fn stepper_size_changes_badge_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let badge = |size| resolve_stepper(&stepper_style(&mode, size), true).root().number("step_badge_size");

        assert!(badge(crate::ShadcnSize::Sm) < badge(crate::ShadcnSize::Lg));
    }
}
