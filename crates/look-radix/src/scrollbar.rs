//! Radix scrollbar chrome for standalone and composed SDK scrolling controls.
use std::sync::Arc;

use xgpui_luma::controls::scrollbar::{ScrollbarTemplate, ScrollbarTheme, StyledScrollbarTheme, ThemedScrollbarTemplate};
use xgpui_luma::theme::MetricTokens;
use crate::Look;

struct ScrollbarThemeAdapter {
    look: Look,
    styled: StyledScrollbarTheme,
}
impl ScrollbarTheme for ScrollbarThemeAdapter {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.styled.style()
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics_for(self.look.size())
    }
}
/// Live Radix neutral scrollbar from `.lss` (`style/scrollbar.lss`), sizes 1–3.
/// Retains the explicit Look; notify affected owners after changing it.
pub fn scrollbar_theme(look: &Look) -> Arc<dyn ScrollbarTheme> {
    let styled = StyledScrollbarTheme::new(crate::textfield::sized_binding(look, "scrollbar"), MetricTokens::default());
    Arc::new(ScrollbarThemeAdapter { look: look.clone(), styled })
}

/// Shared template for SDK Scrollbar, ScrollContainer, and PopupScrollSurface.
/// Dragging, wheel routing, visibility, and auto-hide behavior remain in the SDK.
///
/// ```
/// use xgpui_luma::controls::scrollbar::ScrollbarBuilder;
/// use xgpui_luma_look_radix::{Look, scrollbar_template};
/// let look = Look::built_in();
/// let builder = ScrollbarBuilder::new("scrollbar").template(scrollbar_template(&look));
/// ```
pub fn scrollbar_template(look: &Look) -> Arc<dyn ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(scrollbar_theme(look)))
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    use xgpui_luma::theme::InteractionState;
    use crate::ScaleFamily;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn states_sizes_orientations_and_modes_use_radix_colors() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for size in [crate::RadixSize::One, crate::RadixSize::Two, crate::RadixSize::Three] {
                let theme = scrollbar_theme(&look.with_size(size));
                for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                    for (state, step) in [
                        (InteractionState::default(), 8),
                        (InteractionState { hovered: true, ..Default::default() }, 9),
                        (InteractionState { pressed: true, ..Default::default() }, 10),
                        (InteractionState { disabled: true, pressed: true, ..Default::default() }, 5),
                    ] {
                        let resolved = theme.resolve(state, orientation, ScrollbarStyle::Soft);
                        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
                        crate::control_group::assert_close(resolved.thumb_background, gray(step));
                        crate::control_group::assert_close(resolved.track_background, gray(3));
                        assert!(resolved.min_thumb_length > 0.0);
                        assert!(resolved.thickness >= resolved.thumb_thickness);
                    }
                }
            }
        }
    }

    #[test]
    fn live_geometry_and_fork_changes_are_local() {
        let look = Look::built_in();
        let fork = look.fork();
        let theme = scrollbar_theme(&fork);
        let resolve = || theme.resolve(Default::default(), ScrollbarOrientation::Vertical, ScrollbarStyle::Ghost);
        let before = resolve();
        fork.set_mode(ThemeMode::Dark);
        let after = resolve();
        assert_ne!(after.thumb_background, before.thumb_background);
        let original =
            scrollbar_theme(&look).resolve(Default::default(), ScrollbarOrientation::Vertical, ScrollbarStyle::Ghost);
        assert_eq!(original.length, before.length);
        assert_eq!(original.thumb_background, before.thumb_background);
        assert_eq!(original.track_background.a, 0.0);
    }
}
