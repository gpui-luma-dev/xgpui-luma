use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, SharedString, Window, point, px};

use super::{OverlayWindowControl, OverlayWindowTemplate, default_overlay_window_template};
use super::template::modified_overlay_window_template;

pub type OverlayWindowElementRenderer =
    Arc<dyn Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayWindowMode {
    #[default]
    Modal,
    Modeless,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayWindowDismissPolicy {
    #[default]
    KeepOpen,
    CloseOnClickAway,
    CloseOnFocusLoss,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OverlayWindowPosition {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Absolute(gpui::Point<gpui::Pixels>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OverlayWindowEvent {
    Opened,
    Dismissed,
}

#[derive(Clone)]
pub struct OverlayWindowModel {
    pub(crate) id: SharedString,
    pub(crate) content: OverlayWindowElementRenderer,
    pub(crate) open: bool,
    pub(crate) mode: OverlayWindowMode,
    pub(crate) position: OverlayWindowPosition,
    pub(crate) dismiss_policy: OverlayWindowDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) draggable: bool,
    pub(crate) width: Option<f32>,
    pub(crate) template: Arc<dyn OverlayWindowTemplate>,
}

pub struct OverlayWindowRenderModel<'a> {
    pub id: &'a SharedString,
    pub mode: OverlayWindowMode,
    pub position: OverlayWindowPosition,
    pub dismissible: bool,
    pub draggable: bool,
    pub width: Option<f32>,
    pub focused: bool,
    pub content: &'a OverlayWindowElementRenderer,
}

pub(crate) struct ThemeInvalidator(Box<dyn Fn(&mut App) + Send>);

impl ThemeInvalidator {
    pub fn from_entity<T: 'static>(entity: Entity<T>) -> Self {
        Self(Box::new(move |cx| {
            entity.update(cx, |_, cx| cx.notify());
        }))
    }

    pub fn invalidate(&self, cx: &mut App) {
        (self.0)(cx);
    }
}

pub struct OverlayWindowBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::overlay_window::OverlayWindowEvent>,

    pub(crate) model: OverlayWindowModel,
    pub(crate) theme_children: Vec<ThemeInvalidator>,
}

pub fn new(id: impl Into<SharedString>) -> OverlayWindowBuilder {
    OverlayWindowBuilder::new(id)
}

impl OverlayWindowBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::overlay_window::OverlayWindowEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `OverlayWindowEvent::Opened` through the existing event stream.
    pub fn on_opened<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::overlay_window::OverlayWindowEvent::Opened = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `OverlayWindowEvent::Dismissed` through the existing event stream.
    pub fn on_dismissed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::overlay_window::OverlayWindowEvent::Dismissed = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: OverlayWindowModel {
                id: id.into(),
                content: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                mode: OverlayWindowMode::Modal,
                position: OverlayWindowPosition::Center,
                dismiss_policy: OverlayWindowDismissPolicy::KeepOpen,
                dismissible: true,
                draggable: false,
                width: None,
                template: default_overlay_window_template(),
            },
            theme_children: Vec::new(),
        }
    }

    /// Register a persistent entity hosted in this overlay for theme invalidation fan-out.
    ///
    /// Spawn child controls before the overlay, then register them here so they re-render when
    /// [`crate::theme::revision::LumaThemeRevision`] changes without app-level notify plumbing.
    pub fn theme_child<T: 'static>(mut self, entity: Entity<T>) -> Self {
        self.theme_children.push(ThemeInvalidator::from_entity(entity));
        self
    }

    pub fn theme_children<I, T>(mut self, entities: I) -> Self
    where
        I: IntoIterator<Item = Entity<T>>,
        T: 'static,
    {
        for entity in entities {
            self.theme_children.push(ThemeInvalidator::from_entity(entity));
        }
        self
    }

    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.model.content = Arc::new(content);
        self
    }

    pub fn body<F>(self, body: F) -> Self
    where
        F: Fn(&OverlayWindowRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.content(body)
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn mode(mut self, mode: OverlayWindowMode) -> Self {
        self.model.mode = mode;
        self
    }

    pub fn position(mut self, position: OverlayWindowPosition) -> Self {
        self.model.position = position;
        self
    }

    pub fn absolute_position(mut self, x: f32, y: f32) -> Self {
        self.model.position = OverlayWindowPosition::Absolute(point(px(x), px(y)));
        self
    }

    pub fn dismiss_policy(mut self, dismiss_policy: OverlayWindowDismissPolicy) -> Self {
        self.model.dismiss_policy = dismiss_policy;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.model.dismissible = dismissible;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.model.draggable = draggable;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.model.width = Some(width);
        self
    }

    pub fn template(mut self, template: Arc<dyn OverlayWindowTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &OverlayWindowRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_overlay_window_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<OverlayWindowControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| OverlayWindowControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_overlay_window_template();
        let builder = OverlayWindowBuilder::new("dialog-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn theme_children_starts_empty() {
        let builder = OverlayWindowBuilder::new("dialog-test");
        assert!(builder.theme_children.is_empty());
    }
}
