//! Shadcn look binding for the SDK Frame.
//!
//! `Frame::new` uses general panel tokens. `Frame::sidebar` explicitly selects
//! sidebar tokens. GPUI's `Styled` methods override either preset, including
//! border widths per edge, asymmetric padding, min/max bounds, and flex layout.
use gpui::{App, Context, Edges, Entity, IntoElement, SharedString, Stateful, Div, StyleRefinement, Styled, px};
use xgpui_luma::controls::frame::{FrameBuilder, FrameControl, FrameLook};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::ShadcnRadius;
use super::sidebar::sidebar_container_look;

/// Look-owned builder; rendering and retained state live in the SDK.
pub struct Frame {
    look: Option<ShadcnLook>,
    sidebar: bool,
    builder: FrameBuilder,
}
impl Frame {
    /// General panel appearance with zero padding, independent of sidebar tokens.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { look: None, sidebar: false, builder: FrameBuilder::new(id) }
    }
    /// Explicit sidebar appearance with an 8px inset and the same style options as any frame.
    pub fn sidebar(id: impl Into<SharedString>) -> Self {
        Self { sidebar: true, ..Self::new(id) }
    }
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn child<E: IntoElement + Clone + 'static>(mut self, child: E) -> Self {
        self.builder = self.builder.child(child);
        self
    }
    pub fn child_render<E: IntoElement>(mut self, render: impl Fn() -> E + 'static) -> Self {
        self.builder = self.builder.child_render(render);
        self
    }
    /// Bind the look handle now, while resolving its current colors every render.
    pub fn into_sdk_builder(self, cx: &App) -> FrameBuilder {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.builder.look_provider(move |_| frame_look(&look, self.sidebar))
    }
    pub fn render(&self, cx: &App) -> Stateful<Div> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        let sidebar = self.sidebar;
        self.builder.clone().look_provider(move |_| frame_look(&look, sidebar)).render(cx)
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<FrameControl> {
        self.into_sdk_builder(cx).spawn(cx)
    }
}
impl Styled for Frame {
    fn style(&mut self) -> &mut StyleRefinement {
        self.builder.style()
    }
}

fn frame_look(look: &ShadcnLook, sidebar: bool) -> FrameLook {
    let (background, border_color, radius) = if sidebar {
        let container = sidebar_container_look(look.mode_tokens().as_ref());
        (container.background, container.border, px(container.radius))
    } else {
        (look.chrome().panel_background, look.chrome().border, px(look.radius(ShadcnRadius::Lg)))
    };
    FrameLook {
        background,
        border_color,
        radius,
        padding: Edges::all(px(if sidebar { 8.0 } else { 0.0 })),
        ..FrameLook::default()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn generic_and_sidebar_presets_resolve_independently_after_theme_changes() {
        let app = TestAppContext::single();
        let look = crate::test_support::fallback_look();
        let panel = app.update(|cx| Frame::new("panel").look(&look).into_sdk_builder(cx));
        let sidebar = app.update(|cx| Frame::sidebar("sidebar").look(&look).into_sdk_builder(cx));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            app.update(|cx| {
                let mut panel_root = panel.render(cx);
                let mut sidebar_root = sidebar.render(cx);
                let sidebar_defaults = sidebar_container_look(look.mode_tokens().as_ref());
                assert_eq!(panel_root.style().background, Some(look.chrome().panel_background.into()));
                assert_eq!(sidebar_root.style().background, Some(sidebar_defaults.background.into()));
                assert_ne!(panel_root.style().background, sidebar_root.style().background);
                assert_eq!(panel_root.style().border_color, Some(look.chrome().border));
                assert_eq!(sidebar_root.style().border_color, Some(sidebar_defaults.border));
                assert_eq!(panel_root.style().corner_radii.top_left, Some(px(look.radius(ShadcnRadius::Lg)).into()));
            });
        }
    }

    #[test]
    fn inline_and_spawned_frames_share_style_and_padding() {
        use gpui::{Render, Window, div, prelude::*};
        struct Host {
            spawned: Entity<FrameControl>,
            look: ShadcnLook,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().flex().child(self.spawned.clone()).child(
                    Frame::new("inline")
                        .look(&self.look)
                        .w(px(200.0))
                        .h(px(100.0))
                        .px(px(12.0))
                        .py(px(4.0))
                        .child_render(|| div().debug_selector(|| "inline-content".into()).size_full())
                        .render(cx),
                )
            }
        }
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            let look = crate::test_support::fallback_look();
            let spawned = Frame::new("spawned")
                .look(&look)
                .w(px(200.0))
                .h(px(100.0))
                .px(px(12.0))
                .py(px(4.0))
                .child_render(|| div().debug_selector(|| "spawned-content".into()).size_full())
                .spawn(cx);
            Host { spawned, look }
        });
        cx.run_until_parked();
        let spawned = cx.debug_bounds("spawned-content").unwrap();
        let inline = cx.debug_bounds("inline-content").unwrap();
        assert_eq!(spawned.size, inline.size);
        assert_eq!(spawned.size.width, px(176.0));
        assert_eq!(spawned.size.height, px(92.0));
        assert_eq!(spawned.top(), px(4.0));
        assert_eq!(spawned.left(), px(12.0));
    }
}
