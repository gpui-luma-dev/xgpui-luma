//! Radix resize chrome over the SDK splitter and split-view engines.
use std::sync::Arc;
use gpui::{SharedString, px};
use xgpui_luma::controls::{dock_splitter::*, split_view::*};
use crate::{Look, ScaleFamily, SemanticRole};

struct DockTheme {
    look: Look,
}
impl DockSplitterTheme for DockTheme {
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        let geometry = self.look.common_stylesheet().dock_splitter.resolve_geometry("md", Default::default());
        let active = self
            .look
            .resolve_role(if enabled {
                SemanticRole::Focus
            } else {
                SemanticRole::MutedForeground
            })
            .hsla();
        DockSplitterLook {
            line_color: self
                .look
                .resolve_role(if enabled {
                    SemanticRole::Border
                } else {
                    SemanticRole::MutedForeground
                })
                .hsla(),
            hover_color: active,
            thumb_color: active,
            hit_target_px: geometry.hit_target,
            visible_line_px: geometry.visible_line,
            geometry,
        }
    }
}
struct SplitTheme {
    look: Look,
}
impl SplitViewTheme for SplitTheme {
    fn resolve(&self, _: bool, enabled: bool) -> SplitViewLook {
        SplitViewLook {
            geometry: self.look.common_stylesheet().split_view.resolve_geometry("", Default::default()),
            separator: if enabled {
                self.look.resolve_role(SemanticRole::Border).hsla()
            } else {
                self.look.resolve_step(ScaleFamily::Gray, 4).hsla()
            },
            separator_hover: self
                .look
                .resolve_role(if enabled {
                    SemanticRole::Focus
                } else {
                    SemanticRole::MutedForeground
                })
                .hsla(),
        }
    }
}
/// Live Radix splitter colors and shared geometry. Notify controls after Look edits.
pub fn dock_splitter_theme(look: &Look) -> Arc<dyn DockSplitterTheme> {
    Arc::new(DockTheme { look: look.clone() })
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
/// Live Radix separator colors and geometry, including disabled chrome.
pub fn split_view_theme(look: &Look) -> Arc<dyn SplitViewTheme> {
    Arc::new(SplitTheme { look: look.clone() })
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
    use crate::{Accent, Gray};
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
                    assert_eq!(dock.hover_color, split.separator_hover);
                    assert_eq!(
                        dock.hover_color,
                        look.resolve_role(if enabled {
                            SemanticRole::Focus
                        } else {
                            SemanticRole::MutedForeground
                        })
                        .hsla()
                    );
                    assert!(dock.hit_target_px >= dock.visible_line_px);
                    assert!(split.geometry.hitbox_width >= split.geometry.cue_hovered_width);
                }
            }
        }
    }
    #[test]
    fn splitter_geometry_updates_are_isolated_to_forks() {
        let base = Look::built_in();
        let fork = base.fork();
        let dock = dock_splitter_theme(&fork);
        let split = split_view_theme(&fork);
        let before = dock_splitter_theme(&base).resolve(true);
        let mut sheet = fork.common_stylesheet();
        sheet.dock_splitter.geometry.hit_target = Some(19.0);
        sheet.split_view.geometry.hitbox_width = Some(27.0);
        fork.set_common_stylesheet(sheet);
        assert_eq!(dock.resolve(true).hit_target_px, 19.0);
        assert_eq!(split.resolve(false, true).geometry.hitbox_width, 27.0);
        assert_eq!(dock_splitter_theme(&base).resolve(true).hit_target_px, before.hit_target_px);
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
