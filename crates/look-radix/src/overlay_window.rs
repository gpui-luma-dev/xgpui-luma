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
    use xgpui_luma::controls::overlay_window::OverlayWindowMode;

    #[test]
    fn sizes_and_modes_match_the_radix_window() {
        let look = Look::built_in();
        let at = |size, mode| overlay_window_theme(&look.with_size(size)).resolve(mode);
        let modal = at(crate::RadixSize::Two, OverlayWindowMode::Modal);
        assert_eq!((modal.radius, modal.padding, modal.min_width, modal.max_width), (8.0, 12.0, 360.0, 440.0));
        assert!((modal.backdrop_background.a - 0.42).abs() < 1e-3);
        assert_eq!(at(crate::RadixSize::Two, OverlayWindowMode::Modeless).backdrop_background.a, 0.0);
        let small = at(crate::RadixSize::One, OverlayWindowMode::Modeless);
        assert_eq!((small.padding, small.min_width, small.estimated_height), (8.0, 280.0, 180.0));
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_overlay_window_stylesheet() -> xgpui_luma::theme::stylesheet::OverlayWindowStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    OverlayWindowStylesheet {
        geometry: OverlayWindowGeometryStylesheet { backdrop_opacity: Some(0.42), ..Default::default() },
        sizes: [("sm", 280.0, 360.0, 180.0), ("md", 360.0, 440.0, 220.0), ("lg", 440.0, 520.0, 260.0)]
            .into_iter()
            .map(|(key, min, max, height)| {
                (
                    key.into(),
                    OverlayWindowGeometryStylesheet {
                        min_width: Some(min),
                        max_width: Some(max),
                        estimated_height: Some(height),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}
