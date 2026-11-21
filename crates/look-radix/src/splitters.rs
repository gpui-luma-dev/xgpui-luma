//! Radix resize chrome over the SDK splitter and split-view engines.
use std::sync::Arc;
use gpui::{SharedString, px};
use xgpui_luma::controls::{dock_splitter::*, split_view::*};
use crate::Look;

/// Live Radix splitter from `.lss` (`style/dock_splitter.lss`). Notify controls after Look edits.
pub fn dock_splitter_theme(look: &Look) -> Arc<dyn DockSplitterTheme> {
    Arc::new(StyledDockSplitterTheme::new(crate::textfield::sized_binding(look, "dock_splitter")))
}
/// SDK splitter template; enable the optional vertical grip with `show_thumb`.
pub fn dock_splitter_template(show_thumb: bool) -> Arc<dyn DockSplitterTemplate> {
    Arc::new(ThemedDockSplitterTemplate::new(show_thumb))
}
/// Native SDK builder retaining pointer/keyboard resizing and callback APIs.
/// Pair a custom thumb template with this builder when needed.
pub fn dock_splitter_builder(
    look: &Look,
    id: impl Into<SharedString>,
    orientation: SplitterOrientation,
) -> DockSplitterBuilder {
    DockSplitterBuilder::new(id, orientation)
        .theme(dock_splitter_theme(look))
        .template(dock_splitter_template(false))
}
/// Live Radix separator from `.lss` (`style/split_view.lss`), including disabled chrome.
pub fn split_view_theme(look: &Look) -> Arc<dyn SplitViewTheme> {
    Arc::new(StyledSplitViewTheme::new(crate::textfield::sized_binding(look, "split_view")))
}
/// SDK pane/separator template. The render model supplies [`split_view_theme`].
pub fn split_view_template() -> Arc<dyn SplitViewTemplate> {
    default_split_view_template()
}
/// Native SDK split view with Radix chrome and Look motion/pane geometry defaults.
/// Configure panes, constraints, collapse behavior, and callbacks before spawning.
///
/// ```no_run
/// use gpui::{Context, div, px};
/// use xgpui_luma::controls::dock_splitter::SplitterOrientation;
/// use xgpui_luma_look_radix::{Look, dock_splitter_builder, split_view_builder};
/// fn panes<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     dock_splitter_builder(look, "divider", SplitterOrientation::Vertical).spawn(cx);
///     split_view_builder(look, "panes").sidebar(|| div()).content(|| div())
///         .sidebar_width(px(240.0)).spawn(cx);
/// }
/// ```
pub fn split_view_builder(look: &Look, id: impl Into<SharedString>) -> SplitViewBuilder {
    let geometry = look.common_stylesheet().shared_geometry();
    SplitViewBuilder::new(id)
        .motion_defaults(look.motion_defaults().split_view)
        .sidebar_min_width(px(geometry.split_sidebar_min_width))
        .sidebar_max_width(px(geometry.split_sidebar_max_width))
        .sidebar_width(px(geometry.split_sidebar_width))
        .theme(split_view_theme(look))
        .template(split_view_template())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Accent, Gray, SemanticRole};
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn splitter_colors_follow_live_palettes_and_disabled_state() {
        let look = Look::built_in();
        let dock = dock_splitter_theme(&look);
        let split = split_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for accent in [Accent::Blue, Accent::Lime] {
                look.set_palettes(accent, Gray::Auto);
                for enabled in [false, true] {
                    let dock = dock.resolve(enabled);
                    let split = split.resolve(true, enabled);
                    crate::control_group::assert_close(dock.hover_color, split.separator_hover);
                    crate::control_group::assert_close(
                        dock.hover_color,
                        look.resolve_role(if enabled {
                            SemanticRole::Focus
                        } else {
                            SemanticRole::MutedForeground
                        })
                        .hsla(),
                    );
                    assert!(dock.hit_target_px >= dock.visible_line_px);
                    assert!(split.geometry.hitbox_width >= split.geometry.cue_hovered_width);
                }
            }
        }
    }
    #[cfg(feature = "test-support")]
    #[test]
    fn split_view_factory_retains_constraints_and_collapse_state() {
        use gpui::TestAppContext;
        let app = TestAppContext::single();
        let control = app.update(|cx| {
            split_view_builder(&Look::built_in(), "constraints")
                .sidebar_min_width(px(60.0))
                .sidebar_max_width(px(100.0))
                .sidebar_width(px(80.0))
                .motion_enabled(false)
                .spawn(cx)
        });
        app.update(|cx| {
            control.update(cx, |view, cx| {
                view.set_sidebar_width(px(999.0), cx);
                assert_eq!(view.sidebar_width(), px(100.0));
                view.set_collapsed(true, cx);
                assert_eq!(view.effective_sidebar_width(), px(0.0));
                view.set_collapsed(false, cx);
                assert_eq!(view.effective_sidebar_width(), px(100.0));
            })
        });
    }
}

#[cfg(test)]
mod lss_tests {
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    use xgpui_luma::theme::{InteractionState, ThemeMode};

    use crate::control_group::assert_close;
    use crate::{Look, RadixSize, ScaleFamily, SemanticRole};

    #[test]
    fn lss_layout_matches_the_native_layout() {
        let base = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            base.set_mode(mode);
            let gray = |n| base.resolve_step(ScaleFamily::Gray, n).hsla();
            let role = |r| base.resolve_role(r).hsla();
            for (size, (thickness, track, thumb, min)) in [
                (RadixSize::One, (10.0, 3.0, 6.0, 24.0)),
                (RadixSize::Two, (12.0, 4.0, 8.0, 28.0)),
                (RadixSize::Three, (14.0, 5.0, 10.0, 32.0)),
            ] {
                let look = base.with_size(size);
                let scrollbar = crate::scrollbar_theme(&look);
                for (state, step) in [
                    (InteractionState::default(), 8),
                    (InteractionState { hovered: true, ..Default::default() }, 9),
                    (InteractionState { pressed: true, ..Default::default() }, 10),
                    (InteractionState { disabled: true, ..Default::default() }, 5),
                ] {
                    let ghost = scrollbar.resolve(state, ScrollbarOrientation::Horizontal, ScrollbarStyle::Ghost);
                    let soft = scrollbar.resolve(state, ScrollbarOrientation::Vertical, ScrollbarStyle::Soft);
                    assert_close(ghost.thumb_background, gray(step));
                    assert_eq!(ghost.track_background.a, 0.0);
                    assert_close(soft.track_background, gray(3));
                    assert_eq!((ghost.length, soft.length), (260.0, 180.0));
                    assert_eq!(
                        (ghost.thickness, ghost.track_thickness, ghost.thumb_thickness, ghost.min_thumb_length),
                        (thickness, track, thumb, min)
                    );
                    assert_eq!((ghost.radius, ghost.disabled_opacity), (999.0, 0.56));
                }

                let panels = crate::resizable_panels_theme(&look);
                for disabled in [false, true] {
                    let got = panels.resolve(InteractionState { disabled, hovered: true, ..Default::default() });
                    assert_close(got.border, role(SemanticRole::Border));
                    assert_close(got.divider, if disabled { gray(4) } else { role(SemanticRole::Border) });
                    assert_close(got.grip, role(SemanticRole::MutedForeground));
                    let emphasis = if disabled {
                        SemanticRole::MutedForeground
                    } else {
                        SemanticRole::Focus
                    };
                    assert_close(got.grip_emphasis, role(emphasis));
                    assert_eq!((got.border_width, got.divider_width), (1.0, 1.0));
                    assert_eq!(got.disabled_opacity, if disabled { 0.45 } else { 1.0 });
                }

                let (dock, split) = (crate::dock_splitter_theme(&look), crate::split_view_theme(&look));
                for enabled in [true, false] {
                    let active = role(if enabled {
                        SemanticRole::Focus
                    } else {
                        SemanticRole::MutedForeground
                    });
                    let got = dock.resolve(enabled);
                    assert_close(
                        got.line_color,
                        role(if enabled {
                            SemanticRole::Border
                        } else {
                            SemanticRole::MutedForeground
                        }),
                    );
                    assert_close(got.hover_color, active);
                    assert_close(got.thumb_color, active);
                    assert_eq!((got.hit_target_px, got.visible_line_px, got.geometry.thumb_height), (8.0, 1.0, 36.0));
                    let got = split.resolve(false, enabled);
                    assert_close(got.separator, if enabled { role(SemanticRole::Border) } else { gray(4) });
                    assert_close(got.separator_hover, active);
                    assert_eq!((got.geometry.hitbox_width, got.geometry.cue_hovered_width), (20.0, 8.0));
                }
            }
        }
    }
}
