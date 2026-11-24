crate::motion_profile!(SplitViewMotion => split_view {
    extent: crate::motion::Transition,
});

use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, Hsla, IntoElement, Pixels, SharedString, div, px};

use super::{SplitView, SplitViewTemplate, SplitViewTheme, default_split_view_template, default_split_view_theme};
use super::template::modified_split_view_template;

pub type PaneRender = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SplitViewSeparatorVisibility {
    #[default]
    Always,
    Hover,
}

#[derive(Clone)]
pub struct SplitViewModel {
    pub(crate) motion: crate::motion::MotionConfig<SplitViewMotion>,
    pub(crate) id: SharedString,
    pub(crate) sidebar_width: Pixels,
    pub(crate) sidebar_min_width: Pixels,
    pub(crate) sidebar_max_width: Pixels,
    pub(crate) sidebar_collapsed_width: Pixels,
    pub(crate) collapsed: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) resizable: bool,
    pub(crate) enabled: bool,
    pub(crate) separator_visibility: SplitViewSeparatorVisibility,
    pub(crate) theme: Arc<dyn SplitViewTheme>,
    pub(crate) separator_color_override: Option<Hsla>,
    pub(crate) separator_hover_color_override: Option<Hsla>,
    pub(crate) sidebar: PaneRender,
    pub(crate) content: PaneRender,
    pub(crate) template: Arc<dyn SplitViewTemplate>,
}

pub struct SplitViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub sidebar_width: Pixels,
    pub sidebar_collapsed_width: Pixels,
    pub effective_sidebar_width: Pixels,
    pub collapsed: bool,
    pub resizable: bool,
    pub enabled: bool,
    pub separator_hovered: bool,
    pub separator_visibility: SplitViewSeparatorVisibility,
    pub theme: &'a Arc<dyn SplitViewTheme>,
    pub separator_color_override: Option<Hsla>,
    pub separator_hover_color_override: Option<Hsla>,
}

pub struct SplitViewBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::split_view::SplitViewEvent>,

    pub(crate) model: SplitViewModel,
}

impl SplitViewBuilder {
    crate::motion_methods!(SplitViewMotion, model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_view::SplitViewEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SplitViewEvent::ResizeStart` through the existing event stream.
    pub fn on_resize_start<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::ResizeStart = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::SidebarWidthChanged` through the existing event stream.
    pub fn on_sidebar_width_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::Pixels, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::SidebarWidthChanged { width } = event {
                callback(owner, *width, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::ResizeEnd` through the existing event stream.
    pub fn on_resize_end<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::Pixels, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::ResizeEnd { width } = event {
                callback(owner, *width, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::CollapsedChanged` through the existing event stream.
    pub fn on_collapsed_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::CollapsedChanged { collapsed } = event {
                callback(owner, *collapsed, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::SeparatorHoverChanged` through the existing event stream.
    pub fn on_separator_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::SeparatorHoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: SplitViewModel {
                motion: Default::default(),
                id: id.into(),
                sidebar_width: px(crate::theme::stylesheet::SharedGeometry::sdk().split_sidebar_width),
                sidebar_min_width: px(crate::theme::stylesheet::SharedGeometry::sdk().split_sidebar_min_width),
                sidebar_max_width: px(crate::theme::stylesheet::SharedGeometry::sdk().split_sidebar_max_width),
                sidebar_collapsed_width: px(0.0),
                collapsed: false,
                motion_enabled: true,
                resizable: true,
                enabled: true,
                separator_visibility: SplitViewSeparatorVisibility::Always,
                theme: default_split_view_theme(),
                separator_color_override: None,
                separator_hover_color_override: None,
                sidebar: render_pane(|| div()),
                content: render_pane(|| div()),
                template: default_split_view_template(),
            },
        }
    }

    pub fn sidebar_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_width =
            clamp_sidebar_width(width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_min_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_min_width = width.max(px(0.0));
        self.model.sidebar_max_width = self.model.sidebar_max_width.max(self.model.sidebar_min_width);
        self.model.sidebar_width =
            clamp_sidebar_width(self.model.sidebar_width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_max_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_max_width = width.max(self.model.sidebar_min_width);
        self.model.sidebar_width =
            clamp_sidebar_width(self.model.sidebar_width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_collapsed_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_collapsed_width = width.max(px(0.0));
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.model.collapsed = collapsed;
        self
    }

    /// Enables or disables the sidebar expand/collapse transition.
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.model.resizable = resizable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn separator_visibility(mut self, visibility: SplitViewSeparatorVisibility) -> Self {
        self.model.separator_visibility = visibility;
        self
    }

    pub fn theme(mut self, theme: Arc<dyn SplitViewTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    /// Overrides the themed separator color with a fixed value.
    pub fn separator_color(mut self, color: impl Into<Hsla>) -> Self {
        self.model.separator_color_override = Some(color.into());
        self
    }

    /// Overrides the themed hover separator color with a fixed value.
    pub fn separator_hover_color(mut self, color: impl Into<Hsla>) -> Self {
        self.model.separator_hover_color_override = Some(color.into());
        self
    }

    pub fn sidebar<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.sidebar = render_pane(render);
        self
    }

    pub fn content<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.content = render_pane(render);
        self
    }

    pub fn template(mut self, template: Arc<dyn SplitViewTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &SplitViewRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_split_view_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SplitView> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| SplitView::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn render_pane<E, F>(render: F) -> PaneRender
where
    E: IntoElement,
    F: Fn() -> E + 'static,
{
    Rc::new(move || render().into_any_element())
}

pub(crate) fn clamp_sidebar_width(width: Pixels, min_width: Pixels, max_width: Pixels) -> Pixels {
    width.max(min_width.max(px(0.0))).min(max_width.max(min_width))
}

pub(crate) fn effective_sidebar_width(collapsed: bool, sidebar_width: Pixels, collapsed_width: Pixels) -> Pixels {
    if collapsed {
        collapsed_width.max(px(0.0))
    } else {
        sidebar_width.max(px(0.0))
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;
    use std::sync::Arc;

    use super::{SplitViewBuilder, clamp_sidebar_width, default_split_view_template, effective_sidebar_width};

    #[test]
    fn sidebar_width_clamps_to_configured_range() {
        assert_eq!(clamp_sidebar_width(px(120.0), px(220.0), px(420.0)), px(220.0));
        assert_eq!(clamp_sidebar_width(px(320.0), px(220.0), px(420.0)), px(320.0));
        assert_eq!(clamp_sidebar_width(px(520.0), px(220.0), px(420.0)), px(420.0));
    }

    #[test]
    fn effective_sidebar_width_uses_collapsed_width_without_destroying_expanded_width() {
        assert_eq!(effective_sidebar_width(false, px(320.0), px(0.0)), px(320.0));
        assert_eq!(effective_sidebar_width(true, px(320.0), px(0.0)), px(0.0));
        assert_eq!(effective_sidebar_width(true, px(320.0), px(64.0)), px(64.0));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_split_view_template();
        let builder = SplitViewBuilder::new("split-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn animated_defaults_true() {
        assert!(SplitViewBuilder::new("split-animated").model.motion_enabled);
        assert!(!SplitViewBuilder::new("split-animated-off").motion_enabled(false).model.motion_enabled);
    }
}
