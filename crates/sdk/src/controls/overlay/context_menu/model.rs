crate::motion_profile!(ContextMenuMotion => context_menu {
    submenu: crate::motion::PresenceMotion,
    disclosure: crate::motion::Transition,
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, IntoElement, Pixels, Point, SharedString};

use super::{ContextMenu, ContextMenuTemplate, ControlFocusState, MenuPath, default_context_menu_template};
use crate::theme::InteractionState;
use super::template::modified_context_menu_template;
use crate::infra::menu_item::MenuItem;
use crate::motion::overlay_presence::OverlayPresence;

#[derive(Clone)]
pub struct ContextMenuModel {
    pub(crate) motion: crate::motion::MotionConfig<ContextMenuMotion>,
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) target_content: Option<ContextMenuTargetContent>,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    pub(crate) dismiss: crate::infra::popup_dismiss::PopupDismiss,
    pub(crate) template: Arc<dyn ContextMenuTemplate>,
}

pub type ContextMenuTargetContent = Arc<dyn Fn(&mut App) -> AnyElement + Send + Sync + 'static>;

pub struct ContextMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub target_content: Option<&'a ContextMenuTargetContent>,
    pub items: &'a [MenuItem],
    pub menu_position: Option<Point<Pixels>>,
    pub presence: OverlayPresence,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub submenu_presence: OverlayPresence,
    pub submenu_transition: Option<(usize, f32)>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: InteractionState,
}

pub struct ContextMenuBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::context_menu::ContextMenuEvent>,

    pub(crate) model: ContextMenuModel,
}

impl ContextMenuBuilder {
    crate::motion_methods!(ContextMenuMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::context_menu::ContextMenuEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ContextMenuEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::context_menu::ContextMenuEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            callbacks: Default::default(),

            model: ContextMenuModel {
                motion: Default::default(),
                label: id.clone(),
                id,
                target_content: None,
                items: Vec::new(),
                enabled: true,
                dismiss: crate::infra::popup_dismiss::PopupDismiss::LIGHT,
                template: default_context_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn target_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.target_content = Some(Arc::new(move |cx| content(cx).into_any_element()));
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    /// What closes the menu: an outside click, focus loss or Escape by default.
    pub fn dismiss(mut self, dismiss: crate::infra::popup_dismiss::PopupDismiss) -> Self {
        self.model.dismiss = dismiss;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ContextMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ContextMenuRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_context_menu_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<ContextMenu> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| ContextMenu::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_context_menu_template();
        let builder = ContextMenuBuilder::new("context-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
