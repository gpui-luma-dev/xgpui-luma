//! Stepper — accent track + accent-foreground fill (same color elements as progress).

use xgpui_luma::controls::stepper::StepperLook;
use crate::size::ShadcnSize;

use crate::mode::ShadcnModeTokens;
use crate::provenance::ColorSource;
use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct StepperColorTable {
    pub complete_bg: ResolvedColor,
    pub complete_fg: ResolvedColor,
    pub in_progress_bg: ResolvedColor,
    pub in_progress_fg: ResolvedColor,
    pub incomplete_bg: ResolvedColor,
    pub incomplete_border: ResolvedColor,
    pub incomplete_fg: ResolvedColor,
    pub track_active: ResolvedColor,
    pub track_muted: ResolvedColor,
}

impl StepperColorTable {
    pub fn fallback() -> Self {
        Self {
            complete_bg: ResolvedColor::fallback_foreground(),
            complete_fg: ResolvedColor::transparent(),
            in_progress_bg: ResolvedColor::fallback_foreground(),
            in_progress_fg: ResolvedColor::transparent(),
            incomplete_bg: ResolvedColor::transparent(),
            incomplete_border: ResolvedColor {
                value: gpui::hsla(0.0, 0.0, 0.0, 0.0),
                source: ColorSource::Transparent,
            },
            incomplete_fg: ResolvedColor::fallback_foreground(),
            track_active: ResolvedColor::fallback_foreground(),
            track_muted: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
        }
    }
}

/// The Shadcn stepper at `size`, drawn from `.lss` (`style/stepper.lss`) for `mode`'s catalog.
pub fn stepper_look(mode: &ShadcnModeTokens, enabled: bool, size: ShadcnSize) -> StepperLook {
    let binding =
        xgpui_luma::style::StyleBinding::new("stepper", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    xgpui_luma::controls::stepper::styled_stepper_look(&binding, enabled)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;

    use super::stepper_look;

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
        let look = stepper_look(&mode, true, crate::ShadcnSize::Md);

        crate::assert_close(look.complete_bg, catalog.color("accent-foreground").expect("accent-foreground"));
        crate::assert_close(look.complete_fg, catalog.color("accent").expect("accent"));
        crate::assert_close(look.track_active_color, catalog.color("accent-foreground").expect("accent-foreground"));
        crate::assert_close(look.track_muted_color, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn disabled_stepper_complete_badge_has_checkmark_contrast() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = stepper_look(&mode, false, crate::ShadcnSize::Md);

        crate::assert_close(look.complete_bg, catalog.color("muted").expect("muted"));
        crate::assert_close(look.complete_fg, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_ne!(look.complete_bg, look.complete_fg);
    }

    #[test]
    fn stepper_size_changes_badge_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = stepper_look(&mode, true, crate::ShadcnSize::Sm);
        let large = stepper_look(&mode, true, crate::ShadcnSize::Lg);

        assert!(small.step_badge_size < large.step_badge_size);
    }
}

#[derive(Clone, Debug)]
pub struct StepperMetricTable {
    pub step_badge_size: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
}

pub fn resolve_stepper_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
) -> StepperMetricTable {
    resolve_stepper_metrics_for_size(mode, theme_mode, crate::ShadcnSize::Md)
}

/// Resolve the selected look's Stepper dimensions and their sources for a size.
pub fn resolve_stepper_metrics_for_size(
    mode: &crate::ShadcnModeTokens,
    _theme_mode: xgpui_luma::theme::ThemeMode,
    size: crate::ShadcnSize,
) -> StepperMetricTable {
    use crate::tables::metrics::helpers::derived_metric;

    let look = crate::controls::stepper::stepper_look(mode, true, size);

    // What the `.lss` resolves is what's drawn; `[common.stepper]` is no longer a stylesheet section.
    StepperMetricTable {
        step_badge_size: derived_metric("stepper badge size", look.step_badge_size),
        track_thickness: derived_metric("stepper track thickness", look.track_thickness),
    }
}
