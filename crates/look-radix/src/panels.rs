//! Radix panel and drawer chrome over the SDK layout and overlay engines.
use std::sync::Arc;
use gpui::{AnyElement, Pixels, SharedString, Size};
use xgpui_luma::controls::{resizable_panels::*, slide_panel::*};
use xgpui_luma::theme::InteractionState;
use crate::{Look, ScaleFamily, SemanticRole};

struct PanelsTheme {
    look: Look,
}
impl ResizablePanelsTheme for PanelsTheme {
    fn resize_handle_metrics(&self, size: ResizeHandleSize) -> ResizeHandleMetrics {
        size.metrics_with_stylesheet(&self.look.common_stylesheet())
    }
    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let geometry = self.look.common_stylesheet().resizable_panels.resolve_geometry("md", Default::default());
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        ResizablePanelsLook {
            border: self.look.resolve_role(SemanticRole::Border).hsla(),
            border_width: geometry.border_width,
            divider_width: geometry.divider_width,
            divider: if state.disabled {
                self.look.resolve_step(ScaleFamily::Gray, 4).hsla()
            } else {
                self.look.resolve_role(SemanticRole::Border).hsla()
            },
            grip: muted,
            grip_emphasis: if state.disabled {
                muted
            } else {
                self.look.resolve_role(SemanticRole::Focus).hsla()
            },
            disabled_opacity: if state.disabled { geometry.disabled_opacity } else { 1.0 },
        }
    }
}
/// Live Radix frame, divider, grip, disabled styling and SDK handle geometry.
/// Notify affected owners after editing the supplied Look.
pub fn resizable_panels_theme(look: &Look) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(PanelsTheme { look: look.clone() })
}
/// Native SDK panel template; pair with [`resizable_panels_theme`] when composing manually.
pub fn resizable_panels_template() -> Arc<dyn ResizablePanelsTemplate> {
    default_resizable_panels_template()
}
/// SDK panel builder retaining orientation, constraints, keyboard/drag resizing,
/// logical panel identities, collapse behavior, callbacks, and custom pane APIs.
///
/// ```no_run
/// use gpui::{Context, div, px};
/// use xgpui_luma::controls::resizable_panels::{ResizablePanelSpec, ResizeHandleSize};
/// use xgpui_luma_look_radix::{Look, resizable_panels_builder};
/// fn panels<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     resizable_panels_builder(look, "panels")
///         .panel(ResizablePanelSpec::new_render(|| div()).weight(1.0))
///         .panel(ResizablePanelSpec::new_render(|| div()).weight(2.0))
///         .size(px(600.0), px(300.0)).resize_handle(ResizeHandleSize::Md).spawn(cx);
/// }
/// ```
pub fn resizable_panels_builder(look: &Look, id: impl Into<SharedString>) -> ResizablePanelsBuilder {
    ResizablePanelsBuilder::new(id)
        .motion_defaults(look.motion_defaults().resizable_panels)
        .theme(resizable_panels_theme(look))
        .template(resizable_panels_template())
}
/// Create app-owned SDK drawer state with captured Look geometry and motion defaults.
/// Supply content/focus/resize handlers through the SDK state APIs and render functions.
pub fn slide_panel_state(look: &Look, anchor: SlidePanelTopAnchor, size: SlidePanelSizeConfig) -> SlidePanelState {
    SlidePanelState::new(anchor, size)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().slide_panel)
}
/// Resolve live Radix drawer resize chrome shared with Resizable Panels.
pub fn slide_panel_look(look: &Look) -> ResizablePanelsLook {
    resizable_panels_theme(look).resolve(Default::default())
}
/// Render a Radix drawer through the SDK window overlay; state and handlers stay app-owned.
pub fn render_slide_panel_overlay(
    look: &Look,
    state: &SlidePanelState,
    viewport: Size<Pixels>,
    content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize: SlidePanelResizeHandlers,
) -> AnyElement {
    xgpui_luma::controls::slide_panel::render_slide_panel_overlay(
        look.resolve_role(SemanticRole::Surface).hsla(),
        &slide_panel_look(look),
        state,
        viewport,
        content,
        handlers,
        resize,
    )
}
/// Render the same SDK drawer chrome inside a bounded local viewport, suitable for templates.
pub fn render_slide_panel_surface(
    look: &Look,
    state: &SlidePanelState,
    viewport: Size<Pixels>,
    content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize: SlidePanelResizeHandlers,
) -> AnyElement {
    xgpui_luma::controls::slide_panel::render_slide_panel_surface(
        look.resolve_role(SemanticRole::Surface).hsla(),
        &slide_panel_look(look),
        state,
        viewport,
        content,
        handlers,
        resize,
    )
}
/// Render an inset right-edge SDK drawer with live Radix colors.
pub fn render_slide_panel_inset(
    look: &Look,
    state: &SlidePanelState,
    content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize: SlidePanelResizeHandlers,
) -> AnyElement {
    xgpui_luma::controls::slide_panel::render_slide_panel_inset(
        &slide_panel_look(look),
        state,
        content,
        look.resolve_role(SemanticRole::Surface).hsla(),
        handlers,
        resize,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Accent, Gray};
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn panel_colors_and_metrics_follow_live_look() {
        let look = Look::built_in();
        let theme = resizable_panels_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for accent in [Accent::Blue, Accent::Lime] {
                look.set_palettes(accent, Gray::Auto);
                let normal = theme.resolve(Default::default());
                let disabled = theme.resolve(InteractionState {
                    disabled: true,
                    hovered: true,
                    pressed: true,
                    focused: true,
                    ..Default::default()
                });
                assert_eq!(normal.grip_emphasis, look.resolve_role(SemanticRole::Focus).hsla());
                assert_eq!(disabled.grip_emphasis, look.resolve_role(SemanticRole::MutedForeground).hsla());
                assert!(disabled.disabled_opacity < normal.disabled_opacity);
                assert_eq!(slide_panel_look(&look).divider, normal.divider);
                let sizes = [ResizeHandleSize::Sm, ResizeHandleSize::Md, ResizeHandleSize::Lg]
                    .map(|size| theme.resize_handle_metrics(size));
                assert!(sizes[0].hit_target_px < sizes[1].hit_target_px);
                assert!(sizes[1].hit_target_px < sizes[2].hit_target_px);
            }
        }
    }
    #[test]
    fn panel_forks_and_captured_drawer_defaults_are_isolated() {
        let base = Look::built_in();
        let fork = base.fork();
        let theme = resizable_panels_theme(&fork);
        let mut sheet = fork.common_stylesheet();
        sheet.resizable_panels.sizes.sm.hit_target = Some(25.0);
        sheet.slide_panel.geometry.side_width = Some(280.0);
        fork.set_common_stylesheet(sheet);
        let state =
            slide_panel_state(&fork, SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 100.0, 500.0));
        assert_eq!(theme.resize_handle_metrics(ResizeHandleSize::Sm).hit_target_px, 25.0);
        assert_eq!(state.geometry().side_width, Some(280.0));
        assert_ne!(resizable_panels_theme(&base).resize_handle_metrics(ResizeHandleSize::Sm).hit_target_px, 25.0);
        let mut sheet = fork.common_stylesheet();
        sheet.slide_panel.geometry.side_width = Some(320.0);
        fork.set_common_stylesheet(sheet);
        assert_eq!(state.geometry().side_width, Some(280.0));
    }
    #[cfg(feature = "test-support")]
    #[test]
    fn panel_factory_preserves_regions_and_hide_restore() {
        use gpui::{div, px, TestAppContext};
        let app = TestAppContext::single();
        let region = PanelId::new("sidebar");
        let control = app.update(|cx| {
            resizable_panels_builder(&Look::built_in(), "regions")
                .region(region.clone(), ResizablePanelSpec::new_render(|| div()).min(px(40.0)).weight(1.0))
                .panel(ResizablePanelSpec::new_render(|| div()).min(px(40.0)).weight(2.0))
                .size(px(300.0), px(100.0))
                .motion_enabled(false)
                .spawn(cx)
        });
        app.update(|cx| {
            control.update(cx, |panels, cx| {
                panels.set_frame_size(px(300.0), px(101.0), cx);
                let before = panels.panel_sizes_px();
                assert!(before[0] >= 40.0 && before[1] >= 40.0);
                panels.hide_panel(0, PanelHideMode::Completely, cx);
                assert_eq!(panels.is_region_hidden(&region), Some(true));
                panels.show_panel(0, cx);
                assert_eq!(panels.is_region_hidden(&region), Some(false));
                assert_eq!(panels.panel_sizes_px(), before);
            })
        });
    }
    #[cfg(feature = "test-support")]
    #[test]
    fn drawer_state_preserves_all_edge_resize_directions_and_close() {
        use gpui::{point, px, TestAppContext};
        let app = TestAppContext::single();
        let focus = app.update(|cx| cx.focus_handle());
        for edge in [SlidePanelEdge::Left, SlidePanelEdge::Right, SlidePanelEdge::Top, SlidePanelEdge::Bottom] {
            let mut state = slide_panel_state(
                &Look::built_in(),
                SlidePanelTopAnchor::WindowEdge,
                SlidePanelSizeConfig::new(80.0, 40.0, 120.0),
            )
            .motion_enabled(false);
            state.open(edge, focus.clone());
            assert_eq!(state.open_progress(), 1.0);
            state.begin_resize(point(px(100.0), px(100.0)), edge);
            let delta = if matches!(edge, SlidePanelEdge::Left | SlidePanelEdge::Top) {
                200.0
            } else {
                0.0
            };
            state.update_resize(point(px(delta), px(delta)), edge);
            assert_eq!(state.main_axis_size(0.0), 120.0);
            assert!(state.finish_resize());
            assert!(state.request_close());
            assert!(state.active_edge().is_none());
        }
    }
}
