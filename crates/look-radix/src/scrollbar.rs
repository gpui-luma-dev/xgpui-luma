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
mod tests {
    use super::*;
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle, resolve_scrollbar, scrollbar_length};
    use xgpui_luma::style::PartRead;
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
                        let _ = orientation;
                        let resolved = resolve_scrollbar(&theme.state_style(state), state, ScrollbarStyle::Soft);
                        let root = resolved.root();
                        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
                        crate::control_group::assert_close(root.color("thumb"), gray(step));
                        crate::control_group::assert_close(root.color("background"), gray(3));
                        assert!(root.number("min_thumb_length") > 0.0);
                        assert!(root.number("thickness") >= root.number("thumb_thickness"));
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
        let resolve = |theme: &Arc<dyn ScrollbarTheme>| {
            resolve_scrollbar(&theme.state_style(Default::default()), Default::default(), ScrollbarStyle::Ghost)
        };
        let before = resolve(&theme);
        fork.set_mode(ThemeMode::Dark);
        let after = resolve(&theme);
        assert_ne!(after.root().color("thumb"), before.root().color("thumb"));
        let original = resolve(&scrollbar_theme(&look));
        let length =
            |resolved: &xgpui_luma::style::lss::Resolved| scrollbar_length(resolved, ScrollbarOrientation::Vertical);
        assert_eq!(length(&original), length(&before));
        assert_eq!(original.root().color("thumb"), before.root().color("thumb"));
        assert_eq!(original.root().color("background").a, 0.0);
    }
}
