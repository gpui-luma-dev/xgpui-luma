use std::sync::Arc;

use xgpui_luma::controls::split_view::{SplitViewTheme, StyledSplitViewTheme};

use crate::look::ShadcnLook;

/// The Shadcn split view from `.lss` (`style/split_view.lss`).
pub fn split_view_theme(theme: ShadcnLook) -> Arc<dyn SplitViewTheme> {
    Arc::new(StyledSplitViewTheme::new(theme.style_binding("split_view")))
}

#[cfg(test)]
mod lss_tests {
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    use xgpui_luma::theme::{InteractionState, ThemeMode};

    use crate::ShadcnSize;

    fn states() -> [InteractionState; 4] {
        [
            InteractionState::default(),
            InteractionState { hovered: true, ..Default::default() },
            InteractionState { pressed: true, ..Default::default() },
            InteractionState { disabled: true, ..Default::default() },
        ]
    }

    #[test]
    fn lss_layout_matches_the_native_layout() {
        let base = crate::test_support::fallback_look();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            base.set_mode(mode);
            for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
                let look = base.with_size(size);
                let tokens = look.mode_tokens();
                let scrollbar = super::super::scrollbar_theme(look.clone());
                for style in [ScrollbarStyle::Ghost, ScrollbarStyle::Soft] {
                    for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                        for state in states() {
                            let got = scrollbar.resolve(state, orientation, style);
                            let want =
                                crate::controls::scrollbar::scrollbar_look(&tokens, state, orientation, size, style);
                            crate::assert_close(got.track_background, want.track_background);
                            crate::assert_close(got.thumb_background, want.thumb_background);
                            assert_eq!(
                                (got.length, got.thickness, got.track_thickness, got.thumb_thickness),
                                (want.length, want.thickness, want.track_thickness, want.thumb_thickness)
                            );
                            assert_eq!(
                                (got.min_thumb_length, got.radius, got.disabled_opacity),
                                (want.min_thumb_length, want.radius, want.disabled_opacity)
                            );
                        }
                    }
                }

                let panels = super::super::resizable_panels_theme(look.clone());
                for state in states() {
                    let got = panels.resolve(state);
                    let want = crate::controls::resizable_panels::resizable_panels_look(&tokens, mode, state);
                    crate::assert_close(got.border, want.border);
                    crate::assert_close(got.divider, want.divider);
                    crate::assert_close(got.grip, want.grip);
                    crate::assert_close(got.grip_emphasis, want.grip_emphasis);
                    assert_eq!(
                        (got.border_width, got.divider_width, got.disabled_opacity),
                        (want.border_width, want.divider_width, want.disabled_opacity)
                    );
                }

                let split = super::split_view_theme(look.clone());
                let dock = super::super::dock_splitter_theme(look.clone());
                for enabled in [true, false] {
                    let got = split.resolve(false, enabled);
                    let want = crate::controls::split_view::split_view_look(&tokens, mode, false, enabled);
                    crate::assert_close(got.separator, want.separator);
                    crate::assert_close(got.separator_hover, want.separator_hover);
                    assert_eq!(got.geometry, want.geometry);

                    let got = dock.resolve(enabled);
                    let (border, muted) = (tokens.palette.border_default, tokens.palette.disabled_foreground);
                    let line = if enabled { border } else { muted };
                    crate::assert_close(got.line_color, line);
                    crate::assert_close(got.hover_color, line);
                    crate::assert_close(
                        got.thumb_color,
                        if enabled {
                            tokens.palette.primary.background
                        } else {
                            muted
                        },
                    );
                    assert_eq!(
                        got.geometry,
                        tokens.stylesheet().common.dock_splitter.resolve_geometry_for("md", Default::default())
                    );
                }
            }
        }
    }
}
