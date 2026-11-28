//! Radix overlay-window theme for swatch info panels.

use std::sync::Arc;

use xgpui_luma::controls::overlay_window::{OverlayWindowTheme, StyledOverlayWindowTheme};

use crate::look::Look;

/// The Radix overlay window drawn from `.lss` (`style/overlay_window.lss`) at the look's size; follows the
/// look's later edits.
pub fn overlay_window_theme(look: &Look) -> Arc<dyn OverlayWindowTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("overlay_window", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(StyledOverlayWindowTheme::new(binding))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::overlay_window::{OverlayWindowMode, resolve_overlay_window};
    use xgpui_luma::style::PartRead;

    #[test]
    fn sizes_and_modes_match_the_radix_window() {
        let look = Look::built_in();
        let at = |size, mode| {
            let theme = overlay_window_theme(&look.with_size(size));
            resolve_overlay_window(&theme.style(), mode)
        };
        let modal = at(crate::RadixSize::Two, OverlayWindowMode::Modal);
        let m = modal.root();
        assert_eq!(
            (m.number("radius"), m.number("padding"), m.number("min_width"), m.number("max_width")),
            (8.0, 12.0, 360.0, 440.0)
        );
        assert!((m.color("backdrop").a - 0.42).abs() < 1e-3);
        assert_eq!(at(crate::RadixSize::Two, OverlayWindowMode::Modeless).root().color("backdrop").a, 0.0);
        let small = at(crate::RadixSize::One, OverlayWindowMode::Modeless);
        let s = small.root();
        assert_eq!((s.number("padding"), s.number("min_width"), s.number("estimated_height")), (8.0, 280.0, 180.0));
    }
}
