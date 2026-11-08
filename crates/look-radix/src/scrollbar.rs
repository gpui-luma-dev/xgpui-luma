//! Radix scrollbar chrome for standalone and composed SDK scrolling controls.
use std::sync::Arc;

use xgpui_luma::controls::scrollbar::{
    ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTemplate, ScrollbarTheme, ThemedScrollbarTemplate,
};
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};
use crate::{Look, ScaleFamily};

struct ScrollbarThemeAdapter {
    look: Look,
}

impl ScrollbarTheme for ScrollbarThemeAdapter {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        let geometry =
            self.look.common_stylesheet().scrollbar.resolve_geometry(size.stylesheet_key(), Default::default());
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        ScrollbarLook {
            disabled_opacity: geometry.disabled_opacity,
            track_background: if style == ScrollbarStyle::Soft {
                gray(3)
            } else {
                gpui::transparent_black()
            },
            thumb_background: gray(match state.layer() {
                InteractionLayer::Disabled => 5,
                InteractionLayer::Pressed => 10,
                InteractionLayer::Hovered => 9,
                InteractionLayer::Default => 8,
            }),
            length: match orientation {
                ScrollbarOrientation::Horizontal => geometry.length_h,
                ScrollbarOrientation::Vertical => geometry.length_v,
            },
            thickness: geometry.thickness,
            track_thickness: geometry.track_thickness,
            thumb_thickness: geometry.thumb_thickness,
            min_thumb_length: geometry.min_thumb_length,
            radius: self.look.metrics().radius.pill,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

/// Live Radix neutral scrollbar colors with SDK geometry (`sm`, `md`, `lg` keys).
/// Retains the explicit Look; notify affected owners after changing it.
pub fn scrollbar_theme(look: &Look) -> Arc<dyn ScrollbarTheme> {
    Arc::new(ScrollbarThemeAdapter { look: look.clone() })
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
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn states_sizes_orientations_and_modes_use_radix_colors() {
        let look = Look::built_in();
        let theme = scrollbar_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                    for (state, step) in [
                        (InteractionState::default(), 8),
                        (InteractionState { hovered: true, ..Default::default() }, 9),
                        (InteractionState { pressed: true, ..Default::default() }, 10),
                        (InteractionState { disabled: true, pressed: true, ..Default::default() }, 5),
                    ] {
                        let resolved = theme.resolve(state, orientation, size, ScrollbarStyle::Soft);
                        assert_eq!(resolved.thumb_background, look.resolve_step(ScaleFamily::Gray, step).hsla());
                        assert_eq!(resolved.track_background, look.resolve_step(ScaleFamily::Gray, 3).hsla());
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
        let resolve = || {
            theme.resolve(Default::default(), ScrollbarOrientation::Vertical, ControlSize::Md, ScrollbarStyle::Ghost)
        };
        let before = resolve();
        let mut sheet = fork.common_stylesheet();
        sheet.scrollbar.sizes.get_mut("md").unwrap().length_v = Some(321.0);
        sheet.scrollbar.geometry.disabled_opacity = Some(0.2);
        fork.set_common_stylesheet(sheet);
        fork.set_mode(ThemeMode::Dark);
        let after = resolve();
        assert_eq!(after.length, 321.0);
        assert_eq!(after.disabled_opacity, 0.2);
        assert_ne!(after.thumb_background, before.thumb_background);
        let original = scrollbar_theme(&look).resolve(
            Default::default(),
            ScrollbarOrientation::Vertical,
            ControlSize::Md,
            ScrollbarStyle::Ghost,
        );
        assert_eq!(original.length, before.length);
        assert_eq!(original.thumb_background, before.thumb_background);
        assert_eq!(original.track_background.a, 0.0);
    }
}
