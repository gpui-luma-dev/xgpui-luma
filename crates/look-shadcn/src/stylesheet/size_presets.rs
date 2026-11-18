//! Shadcn `sm` / `lg` common geometry.
//!
//! The SDK ships a single (md) scale. Shadcn's size axis lives here: each entry is
//! selected with [`xgpui_luma::theme::stylesheet::CommonStylesheet::with_size_key`]
//! before SDK controls resolve plain pixel geometry.

use xgpui_luma::theme::stylesheet::*;

use crate::ShadcnSize;

/// Add Shadcn's `sm` and `lg` size entries to families whose dimensions vary by size.
pub(crate) fn apply_size_presets(common: &mut CommonStylesheet) {
    for (size, label_baseline) in [(ShadcnSize::Sm, 0.5), (ShadcnSize::Lg, 1.5)] {
        common.shared.sizes.get_or_default(size.key()).label_baseline = Some(label_baseline);
    }
    for (size, scale, radius, inset) in [(ShadcnSize::Sm, 0.45, 2.0, 2.0), (ShadcnSize::Lg, 0.55, 6.0, 4.0)] {
        common.checkbox.sizes.get_or_default(size.key()).indicator(scale, radius, inset);
    }
    for (size, scale) in [(ShadcnSize::Sm, 0.45), (ShadcnSize::Lg, 0.55)] {
        let radio = common.radio.sizes.get_or_default(size.key());
        radio.indicator_scale = Some(scale);
        radio.dot_scale = Some(0.24);
    }
    for (size, thickness, track, thumb, min) in
        [(ShadcnSize::Sm, 10.0, 3.0, 6.0, 24.0), (ShadcnSize::Lg, 14.0, 5.0, 10.0, 32.0)]
    {
        let scrollbar = common.scrollbar.sizes.get_or_default(size.key());
        scrollbar.thickness = Some(thickness);
        scrollbar.track_thickness = Some(track);
        scrollbar.thumb_thickness = Some(thumb);
        scrollbar.min_thumb_length = Some(min);
    }
    for (size, track, diameter, stroke, thumb) in
        [(ShadcnSize::Sm, 4.0, 48.0, 5.0, 12.0), (ShadcnSize::Lg, 8.0, 80.0, 8.0, 20.0)]
    {
        let progress = common.progress.sizes.get_or_default(size.key());
        progress.track_height = Some(track);
        progress.size = Some(diameter);
        progress.stroke_width = Some(stroke);
        progress.thumb_size = Some(thumb);
    }
    for (size, badge, track) in [(ShadcnSize::Sm, 24.0, 2.0), (ShadcnSize::Lg, 40.0, 3.0)] {
        let stepper = common.stepper.sizes.get_or_default(size.key());
        stepper.step_badge_size = Some(badge);
        stepper.track_thickness = Some(track);
    }
    for (size, min, height) in [(ShadcnSize::Sm, 320.0, 196.0), (ShadcnSize::Lg, 480.0, 264.0)] {
        let overlay = common.overlay_window.sizes.get_or_default(size.key());
        overlay.min_width = Some(min);
        overlay.estimated_height = Some(height);
    }
}
