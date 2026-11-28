//! Progress — accent track + accent-foreground fill.

use crate::size::ShadcnSize;

use crate::mode::ShadcnModeTokens;

/// Shadcn progress's style at `size` (`style/progress.lss`) for `mode`'s catalog (#433).
pub fn progress_style(mode: &ShadcnModeTokens, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
    let binding =
        xgpui_luma::style::StyleBinding::new("progress", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
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
    use super::progress_style;

    /// What a test reads of the progress style (#433).
    struct Probe {
        track_color: gpui::Hsla,
        progress_color: gpui::Hsla,
        size: f32,
        stroke_width: f32,
    }

    fn progress_look(mode: &crate::mode::ShadcnModeTokens, enabled: bool, size: crate::ShadcnSize) -> Probe {
        use xgpui_luma::style::PartRead;
        use xgpui_luma::style::lss::PartId;
        let resolved = xgpui_luma::controls::progress::resolve_progress(&progress_style(mode, size), enabled);
        Probe {
            track_color: resolved.part(PartId::new("track")).color("background"),
            progress_color: resolved.part(PartId::new("fill")).color("background"),
            size: resolved.root().number("ring_size"),
            stroke_width: resolved.root().number("stroke_width"),
        }
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
