//! Progress — accent track + accent-foreground fill.

use xgpui_luma::controls::progress::ProgressLook;
use crate::size::ShadcnSize;

use crate::mode::ShadcnModeTokens;
use crate::provenance::ColorSource;
use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct ProgressColorTable {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

impl ProgressColorTable {
    pub fn fallback() -> Self {
        Self {
            track_color: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
            progress_color: ResolvedColor::fallback_foreground(),
        }
    }
}

/// Shadcn progress at `size`, drawn from `.lss` (`style/progress.lss`) for `mode`'s catalog.
pub fn progress_look(mode: &ShadcnModeTokens, enabled: bool, size: ShadcnSize) -> ProgressLook {
    let binding =
        xgpui_luma::style::StyleBinding::new("progress", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
    let binding = if size != ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    xgpui_luma::controls::progress::styled_progress_look(&binding, enabled)
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::progress_look;

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
    fn progress_uses_accent_track_and_accent_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = progress_look(&mode, true, crate::ShadcnSize::Md);

        crate::assert_close(look.track_color, catalog.color("accent").expect("accent"));
        crate::assert_close(look.progress_color, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn progress_size_changes_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = progress_look(&mode, true, crate::ShadcnSize::Sm);
        let medium = progress_look(&mode, true, crate::ShadcnSize::Md);
        let large = progress_look(&mode, true, crate::ShadcnSize::Lg);

        assert!(small.size < medium.size);
        assert!(medium.size < large.size);
        assert!(small.stroke_width < medium.stroke_width);
        assert!(medium.stroke_width < large.stroke_width);
    }
}

#[derive(Clone, Debug)]
pub struct ProgressMetricTable {
    pub size: crate::ResolvedMetric,
    pub stroke_width: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
}

pub fn resolve_progress_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
) -> ProgressMetricTable {
    resolve_progress_metrics_for_size(mode, theme_mode, crate::ShadcnSize::Md)
}

/// Resolve the selected look's Progress dimensions and their sources for a size.
pub fn resolve_progress_metrics_for_size(
    mode: &crate::ShadcnModeTokens,
    _theme_mode: xgpui_luma::theme::ThemeMode,
    size: crate::ShadcnSize,
) -> ProgressMetricTable {
    use crate::tables::metrics::helpers::derived_metric;

    let look = crate::controls::progress::progress_look(mode, true, size);

    // What the `.lss` resolves is what's drawn; `[common.progress]` is no longer a stylesheet section.
    ProgressMetricTable {
        size: derived_metric("progress ring diameter", look.size),
        stroke_width: derived_metric("progress stroke width", look.stroke_width),
        track_height: derived_metric("progress track height", look.track_height),
        thumb_size: derived_metric("progress thumb size", look.thumb_size),
    }
}
