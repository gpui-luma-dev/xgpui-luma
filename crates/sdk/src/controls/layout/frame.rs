//! Look-independent container decoration and layout.
//!
//! Frame has no focus, selection, scrolling, or pane visibility state. Use GPUI's
//! [`Styled`] methods for borders, directional padding, bounds, flex, and shadows.
//! Explicit styles override resolved look defaults. Rounded corners only affect
//! the frame's own painting; they do not clip arbitrary descendant backgrounds.
//!
//! ```no_run
//! use gpui::{AppContext, Entity, div, prelude::*, px};
//! use xgpui_luma::controls::frame::{FrameBuilder, FrameControl};
//!
//! fn inspector(cx: &mut impl AppContext) -> Entity<FrameControl> {
//!     FrameBuilder::new("inspector")
//!         .min_w(px(180.0)).max_w(px(360.0))
//!         .flex().flex_col().gap(px(8.0))
//!         .px(px(12.0)).py(px(8.0))
//!         .border_l(px(1.0)).border_color(gpui::black())
//!         .child_render(|| div().child("Inspector"))
//!         .spawn(cx)
//! }
//! ```
use std::{rc::Rc, sync::Arc};
use gpui::{
    AnyElement, App, AppContext, Context, Div, Entity, IntoElement, Refineable, Render, SharedString, Stateful,
    StyleRefinement, Styled, Subscription, Window, div, prelude::*, px,
};
use crate::style::{PartRead, StyleBinding};
use crate::theme::observe_theme_revision;

/// Called every render so look adapters can resolve current theme values: the frame's style, a `frame` control
/// (#433).
pub type FrameStyleSource = Arc<dyn Fn(&App) -> StyleBinding + Send + Sync>;

/// Content and explicit style overrides shared by inline and spawned frames.
#[derive(Clone)]
pub struct FrameModel {
    pub id: SharedString,
    pub style: StyleRefinement,
    children: Vec<Rc<dyn Fn() -> AnyElement>>,
}

/// A full-size frame by default. GPUI styles can override its dimensions/layout.
#[derive(Clone)]
pub struct FrameBuilder {
    model: FrameModel,
    style_source: FrameStyleSource,
}
impl FrameBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: FrameModel { id: id.into(), style: StyleRefinement::default(), children: Vec::new() },
            style_source: Arc::new(|_| StyleBinding::sdk("frame")),
        }
    }
    /// Inspect the reusable model for custom control templates.
    pub fn model(&self) -> &FrameModel {
        &self.model
    }

    /// The frame's style, resolved every render (#433): a `frame` control, the SDK's (unpainted) by default.
    pub fn style_source(mut self, source: impl Fn(&App) -> StyleBinding + Send + Sync + 'static) -> Self {
        self.style_source = Arc::new(source);
        self
    }
    /// Append a retained entity or other cloneable element.
    pub fn child<E: IntoElement + Clone + 'static>(mut self, child: E) -> Self {
        self.model.children.push(Rc::new(move || child.clone().into_any_element()));
        self
    }
    /// Append content rebuilt on each render.
    pub fn child_render<E: IntoElement>(mut self, render: impl Fn() -> E + 'static) -> Self {
        self.model.children.push(Rc::new(move || render().into_any_element()));
        self
    }
    pub fn render(&self, cx: &App) -> Stateful<Div> {
        render_frame(&self.model, &(self.style_source)(cx).resolve(&[], 1.0))
    }
    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<FrameControl> {
        cx.new(|cx| FrameControl { builder: self, _theme: observe_theme_revision(cx, |_, cx| cx.notify()) })
    }
}
impl Styled for FrameBuilder {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.model.style
    }
}

/// Render the same frame model inside a larger control template, drawn from `style`: a `frame` control's
/// resolved style (#433).
pub fn render_frame(model: &FrameModel, style: &xgpui_luma_style::Resolved) -> Stateful<Div> {
    let look = style.root();
    let (border_width, padding_x, padding_y) =
        (px(look.number("border_width")), px(look.number("padding_x")), px(look.number("padding_y")));
    let mut root = div()
        .id(model.id.clone())
        .size_full()
        .min_w_0()
        .min_h_0()
        .bg(look.color("background"))
        .border_color(look.color("border"))
        .rounded(px(look.number("radius")))
        .px(padding_x)
        .py(padding_y)
        .border(border_width);
    root.style().refine(&model.style);
    root.children(model.children.iter().map(|render| render()))
}

/// Retained SDK frame. Look changes repaint without recreating its child entities.
pub struct FrameControl {
    builder: FrameBuilder,
    _theme: Subscription,
}
impl Render for FrameControl {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.builder.render(cx)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    struct Host {
        frame: FrameBuilder,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.frame.render(cx).debug_selector(|| "frame".into())
        }
    }
    #[test]
    fn border_insets_padding_and_flex_obey_explicit_bounds() {
        for (requested, expected) in [(80.0, 120.0), (200.0, 180.0)] {
            let mut app = TestAppContext::single();
            let (_, cx) = app.add_window_view(|_, _| Host {
                frame: FrameBuilder::new("frame")
                    .style_source(|_| {
                        use xgpui_luma_style::{Overrides, PartId, Prop};
                        let root = PartId::ROOT;
                        StyleBinding::sdk("frame").overrides(
                            Overrides::new()
                                .set(root, Prop::<f32>::new("padding_x"), 30.0)
                                .set(root, Prop::<f32>::new("padding_y"), 30.0)
                                .set(root, Prop::<f32>::new("border_width"), 8.0),
                        )
                    })
                    .w(px(requested))
                    .min_w(px(120.0))
                    .max_w(px(180.0))
                    .h(px(100.0))
                    .min_h(px(60.0))
                    .max_h(px(90.0))
                    .border_t(px(1.0))
                    .border_r(px(2.0))
                    .border_b(px(3.0))
                    .border_l(px(4.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .pl(px(12.0))
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(4.0))
                    .child_render(|| div().debug_selector(|| "first".into()).w(px(20.0)).h(px(10.0)).flex_none())
                    .child_render(|| div().debug_selector(|| "second".into()).w(px(20.0)).h(px(10.0)).flex_none()),
            });
            cx.run_until_parked();
            let frame = cx.debug_bounds("frame").unwrap();
            let first = cx.debug_bounds("first").unwrap();
            let second = cx.debug_bounds("second").unwrap();
            assert_eq!(frame.size.width, px(expected));
            assert_eq!(frame.size.height, px(90.0));
            assert_eq!(first.left() - frame.left(), px(16.0));
            assert_eq!(first.top() - frame.top(), px(7.0));
            assert_eq!(second.left() - first.right(), px(4.0));
        }
    }

    #[test]
    fn explicit_paint_overrides_look_without_removing_other_defaults() {
        let frame = FrameBuilder::new("frame").bg(gpui::black()).border_color(gpui::white()).rounded_tl(px(0.0));
        use xgpui_luma_style::{Overrides, PartId, Prop};
        let root = PartId::ROOT;
        let style = StyleBinding::sdk("frame")
            .overrides(
                Overrides::new()
                    .set(root, Prop::new("background"), crate::style::lss_color(gpui::white()))
                    .set(root, Prop::new("border"), crate::style::lss_color(gpui::black()))
                    .set(root, Prop::<f32>::new("radius"), 12.0),
            )
            .resolve(&[], 1.0);
        let mut root = render_frame(frame.model(), &style);
        assert_eq!(root.style().background, Some(gpui::black().into()));
        assert_eq!(root.style().border_color, Some(gpui::white()));
        assert_eq!(root.style().corner_radii.top_left, Some(px(0.0).into()));
        assert_eq!(root.style().corner_radii.top_right, Some(px(12.0).into()));
    }
}
