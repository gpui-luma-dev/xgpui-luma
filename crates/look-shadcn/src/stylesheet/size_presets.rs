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
}
