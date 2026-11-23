//! Shadcn `sm` / `lg` common geometry.
//!
//! The SDK ships a single scale (md for controls, sm for resize handles). Shadcn's size axis lives here: each entry is
//! selected with [`xgpui_luma::theme::stylesheet::CommonStylesheet::with_size_key`]
//! before SDK controls resolve plain pixel geometry.

use xgpui_luma::theme::stylesheet::*;

use crate::ShadcnSize;

/// Add Shadcn's `sm` and `lg` size entries to families whose dimensions vary by size.
pub(crate) fn apply_size_presets(common: &mut CommonStylesheet) {
    for (size, label_baseline) in [(ShadcnSize::Sm, 0.5), (ShadcnSize::Lg, 1.5)] {
        common.shared.sizes.get_or_default(size.key()).label_baseline = Some(label_baseline);
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
    // The SDK resize handle scale is Shadcn `sm`; `md` and `lg` handles are Shadcn's.
    for (size, lane, hit, cross, main) in
        [(ShadcnSize::Md, 12.0, 14.0, 6.0, 40.0), (ShadcnSize::Lg, 18.0, 20.0, 8.0, 44.0)]
    {
        let handle = common.resizable_panels.sizes.get_or_default(size.key());
        handle.lane_width = Some(lane);
        handle.hit_target = Some(hit);
        handle.grip_cross_axis = Some(cross);
        handle.grip_main_axis = Some(main);
    }
}
