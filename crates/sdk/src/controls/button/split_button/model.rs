use std::sync::Arc;
use gpui::{AppContext, Entity, SharedString};
use crate::controls::popup_menu::{PopupMenuBuilder, PopupMenuPlacement, PopupMenuTriggerStyle};
use crate::infra::{
    callbacks::EventCallbacks,
    presenter::{ControlPresenter, HasPresenter},
    menu_item::MenuItem,
};
use crate::theme::{ControlSize, InteractionState};
use super::{SplitButton, SplitButtonEvent, SplitButtonTriggerModel, SplitButtonTemplate, default_split_button_template};

/// Retained feedback for both faces; focus belongs to the composite root.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SplitButtonState {
    pub primary: InteractionState,
    pub disclosure: InteractionState,
}
impl SplitButtonState {
    pub(crate) fn resolved(self, enabled: bool, primary_enabled: bool, focused: bool) -> Self {
        fn face(mut state: InteractionState, enabled: bool, focused: bool) -> InteractionState {
            state.disabled = !enabled;
            state.focused = focused && enabled;
            if !enabled {
                state.hovered = false;
                state.pressed = false;
            }
            state
        }
        Self {
            primary: face(self.primary, enabled && primary_enabled, focused),
            disclosure: face(self.disclosure, enabled, focused),
        }
    }
}

/// Split-button configuration, composed with the shared popup model.
/// The shared engine owns menu items, placement, motion and the sole focus handle.
pub struct SplitButtonModel {
    pub(super) popup: crate::controls::popup_menu::PopupMenuModel,
}
pub struct SplitButtonBuilder {
    pub(super) model: SplitButtonModel,
    callbacks: EventCallbacks<SplitButtonEvent>,
    engine_callbacks: EventCallbacks<crate::controls::popup_menu::PopupMenuEvent>,
}
impl SplitButtonBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_button::SplitButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SplitButtonEvent::ActionClick` through the existing event stream.
    pub fn on_action_click<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::ActionClick = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::from_popup_builder(PopupMenuBuilder::new(id)).template(default_split_button_template())
    }

    /// Migrate an existing popup preset configuration. Popup callbacks remain
    /// attached to the inner engine; new callbacks use `SplitButtonEvent`.
    pub fn from_popup_builder(mut popup: PopupMenuBuilder) -> Self {
        popup.model.icon_only = false;
        popup.model.split = true;
        Self {
            model: SplitButtonModel { popup: popup.model },
            engine_callbacks: popup.callbacks,
            callbacks: Default::default(),
        }
    }

    pub(super) fn into_popup_builder(self) -> PopupMenuBuilder {
        PopupMenuBuilder { model: self.model.popup, callbacks: self.engine_callbacks }
    }

    fn map_popup(mut self, map: impl FnOnce(PopupMenuBuilder) -> PopupMenuBuilder) -> Self {
        let popup = map(PopupMenuBuilder { model: self.model.popup, callbacks: self.engine_callbacks });
        self.model.popup = popup.model;
        self.engine_callbacks = popup.callbacks;
        self
    }
    pub fn label(self, label: impl Into<SharedString>) -> Self {
        self.map_popup(|popup| popup.label(label))
    }
    pub fn items(self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.map_popup(|popup| popup.items(items))
    }
    pub fn item(self, item: MenuItem) -> Self {
        self.map_popup(|popup| popup.item(item))
    }
    pub fn enabled(self, enabled: bool) -> Self {
        self.map_popup(|popup| popup.enabled(enabled))
    }
    pub fn tab_stop(self, tab_stop: bool) -> Self {
        self.map_popup(|popup| popup.tab_stop(tab_stop))
    }
    pub fn placement(self, placement: PopupMenuPlacement) -> Self {
        self.map_popup(|popup| popup.placement(placement))
    }
    pub fn trigger_style(self, style: PopupMenuTriggerStyle) -> Self {
        self.map_popup(|popup| popup.trigger_style(style))
    }
    pub fn size(self, size: ControlSize) -> Self {
        self.map_popup(|popup| popup.size(size))
    }
    pub fn menu_size(self, size: ControlSize) -> Self {
        self.map_popup(|popup| popup.menu_size(size))
    }
    pub fn full_width(self, full_width: bool) -> Self {
        self.map_popup(|popup| popup.full_width(full_width))
    }
    pub fn disclosure_icons(self, icons: crate::infra::icon::DisclosureIcons) -> Self {
        self.map_popup(|popup| popup.disclosure_icons(icons))
    }
    pub fn end_icon(self, icon: impl Into<crate::controls::button::ControlIcon>) -> Self {
        self.map_popup(|popup| popup.end_icon(icon))
    }
    /// Disable the primary action without disabling the menu or its tab stop.
    pub fn primary_enabled(mut self, enabled: bool) -> Self {
        self.model.popup.primary_enabled = enabled;
        self
    }
    pub fn without_elevation(self) -> Self {
        self.map_popup(|popup| popup.without_elevation())
    }
    pub fn ghost(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Ghost)
    }
    /// Compatibility no-op: this builder always creates a split control.
    pub fn split(self) -> Self {
        self
    }
    /// Icon content in the primary face, preserving split-button geometry.
    pub fn icon(mut self, icon: impl Into<crate::controls::button::ControlIcon>) -> Self {
        self.set_presenter(crate::controls::popup_menu::icon_content(icon));
        self
    }
    pub fn template(self, template: Arc<dyn SplitButtonTemplate>) -> Self {
        self.map_popup(|popup| popup.template(Arc::new(super::template::EngineTemplate(template))))
    }
    pub fn with_template_modifier<F>(self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &super::SplitButtonRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.map_popup(|popup| popup.with_template_modifier(modifier))
    }
    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SplitButton> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = cx.new(|cx| SplitButton::from_builder(self, cx));
        callbacks.bind(&control, cx);
        control
    }
}
impl HasPresenter<SplitButtonTriggerModel> for SplitButtonBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<SplitButtonTriggerModel>) {
        self.model.popup.content = content;
        self.model.popup.icon_only = false;
        self.model.popup.icon = None;
    }
}
