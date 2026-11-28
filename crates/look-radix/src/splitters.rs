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
    let geometry = xgpui_luma::theme::stylesheet::SharedGeometry::from_theme(&look.style_theme());
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
    use xgpui_luma::controls::dock_splitter::resolve_dock_splitter;
    use xgpui_luma::controls::split_view::resolve_split_view;
    use xgpui_luma::style::PartRead;
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
                    let dock = resolve_dock_splitter(&dock.style(), enabled);
                    let split = resolve_split_view(&split.style(), enabled);
                    let (dock, split) = (dock.root(), split.root());
                    crate::control_group::assert_close(dock.color("hover_line"), split.color("separator_hover"));
                    crate::control_group::assert_close(
                        dock.color("hover_line"),
                        look.resolve_role(if enabled {
                            SemanticRole::Focus
                        } else {
                            SemanticRole::MutedForeground
                        })
                        .hsla(),
                    );
                    assert!(dock.number("hit_target") >= dock.number("visible_line"));
                    assert!(split.number("hitbox_width") >= split.number("cue_hovered_width"));
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
    use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle, resolve_scrollbar, scrollbar_length};
    use xgpui_luma::controls::dock_splitter::resolve_dock_splitter;
    use xgpui_luma::controls::resizable_panels::resolve_resizable_panels;
    use xgpui_luma::controls::split_view::resolve_split_view;
    use xgpui_luma::theme::stylesheet::{DockSplitterGeometry, SplitViewGeometry};
    use xgpui_luma::style::PartRead;
    use xgpui_luma::theme::{InteractionState, ThemeMode};

    use crate::control_group::assert_close;
    use crate::{Look, RadixSize, ScaleFamily, SemanticRole};

    /// The splitters' colors read the Radix roles and gray steps for each state. Lengths are checked
    /// as relationships (larger sizes are thicker, thumbs fit their tracks), not shipped values.
    #[test]
    fn lss_layout_reads_the_radix_roles() {
        let base = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            base.set_mode(mode);
            let gray = |n| base.resolve_step(ScaleFamily::Gray, n).hsla();
            let role = |r| base.resolve_role(r).hsla();
            let mut previous_thickness = 0.0;
            for size in [RadixSize::One, RadixSize::Two, RadixSize::Three] {
                let look = base.with_size(size);
                let scrollbar = crate::scrollbar_theme(&look);
                for (state, step) in [
                    (InteractionState::default(), 8),
                    (InteractionState { hovered: true, ..Default::default() }, 9),
                    (InteractionState { pressed: true, ..Default::default() }, 10),
                    (InteractionState { disabled: true, ..Default::default() }, 5),
                ] {
                    let ghost_style = resolve_scrollbar(&scrollbar.state_style(state), state, ScrollbarStyle::Ghost);
                    let soft_style = resolve_scrollbar(&scrollbar.state_style(state), state, ScrollbarStyle::Soft);
                    let (ghost, soft) = (ghost_style.root(), soft_style.root());
                    let thickness = ghost.number("thickness");
                    assert_close(ghost.color("thumb"), gray(step));
                    assert_eq!(ghost.color("background").a, 0.0);
                    assert_close(soft.color("background"), gray(3));
                    assert!(
                        scrollbar_length(&ghost_style, ScrollbarOrientation::Horizontal) > 0.0
                            && scrollbar_length(&soft_style, ScrollbarOrientation::Vertical) > 0.0
                    );
                    assert!(thickness > previous_thickness, "{size:?} is thicker than the size below");
                    assert!(0.0 < ghost.number("track_thickness") && ghost.number("track_thickness") <= thickness);
                    assert!(0.0 < ghost.number("thumb_thickness") && ghost.number("thumb_thickness") <= thickness);
                    assert!(ghost.number("min_thumb_length") > ghost.number("thumb_thickness"));
                    assert!(ghost.number("radius") >= thickness / 2.0, "a pill");
                    assert_eq!(ghost.opacity(), 1.0, "Radix draws disabled colors without fading");
                }
                previous_thickness = resolve_scrollbar(
                    &scrollbar.state_style(InteractionState::default()),
                    InteractionState::default(),
                    ScrollbarStyle::Ghost,
                )
                .root()
                .number("thickness");

                let panels = crate::resizable_panels_theme(&look);
                for disabled in [false, true] {
                    let got = resolve_resizable_panels(
                        &panels.style(),
                        InteractionState { disabled, hovered: true, ..Default::default() },
                    );
                    let got = got.root();
                    assert_close(got.color("border"), role(SemanticRole::Border));
                    assert_close(got.color("divider"), if disabled { gray(4) } else { role(SemanticRole::Border) });
                    assert_close(got.color("grip"), role(SemanticRole::MutedForeground));
                    let emphasis = if disabled {
                        SemanticRole::MutedForeground
                    } else {
                        SemanticRole::Focus
                    };
                    assert_close(got.color("grip_emphasis"), role(emphasis));
                    assert!(got.number("border_width") > 0.0 && got.number("divider_width") > 0.0);
                    assert_eq!(got.opacity(), 1.0, "Radix draws disabled colors without fading");
                }

                let (dock, split) = (crate::dock_splitter_theme(&look), crate::split_view_theme(&look));
                for enabled in [true, false] {
                    let active = role(if enabled {
                        SemanticRole::Focus
                    } else {
                        SemanticRole::MutedForeground
                    });
                    let got = resolve_dock_splitter(&dock.style(), enabled);
                    let geometry = DockSplitterGeometry::from_resolved(&got);
                    let got = got.root();
                    assert_close(
                        got.color("line"),
                        role(if enabled {
                            SemanticRole::Border
                        } else {
                            SemanticRole::MutedForeground
                        }),
                    );
                    assert_close(got.color("hover_line"), active);
                    assert_close(got.color("thumb"), active);
                    assert!(got.number("hit_target") >= got.number("visible_line") && got.number("visible_line") > 0.0);
                    assert!(geometry.thumb_height > 0.0);
                    let got = resolve_split_view(&split.style(), enabled);
                    let geometry = SplitViewGeometry::from_resolved(&got);
                    let got = got.root();
                    assert_close(got.color("separator"), if enabled { role(SemanticRole::Border) } else { gray(4) });
                    assert_close(got.color("separator_hover"), active);
                    assert!(geometry.hitbox_width >= geometry.cue_hovered_width);
                    assert!(geometry.cue_hovered_width > 0.0);
                }
            }
        }
    }
}
