use gpui::{App, Context, Entity, EventEmitter, Focusable, Render, SharedString, Subscription, Window, div, prelude::*};
use crate::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement, PopupMenuTriggerStyle};
use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use super::{SplitButtonBuilder, SplitButtonTemplate, SplitButtonTriggerModel};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SplitButtonEvent {
    ActionClick,
    Select { item_id: SharedString, label: SharedString },
    OpenChanged { open: bool },
    Dismiss,
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
}

/// A distinct retained control; one shared popup engine owns all input and focus.
pub struct SplitButton {
    pub(super) popup: Entity<PopupMenu>,
    attachments: AttachmentHost,
    _events: Subscription,
}
impl EventEmitter<SplitButtonEvent> for SplitButton {}
impl SplitButton {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SplitButtonBuilder {
        SplitButtonBuilder::new(id)
    }

    /// Hosted popup. Disclosure drives the arrow; presence drives the menu.
    pub fn inner_popup(&self) -> Entity<PopupMenu> {
        self.popup.clone()
    }

    pub(super) fn from_builder(builder: SplitButtonBuilder, cx: &mut Context<Self>) -> Self {
        let popup = builder.into_popup_builder().spawn(cx);
        let events = cx.subscribe(&popup, |_, _, event, cx| {
            let event = match event {
                PopupMenuEvent::ActionClick => SplitButtonEvent::ActionClick,
                PopupMenuEvent::Select { item_id, label } => {
                    SplitButtonEvent::Select { item_id: item_id.clone(), label: label.clone() }
                }
                PopupMenuEvent::OpenChanged { open } => SplitButtonEvent::OpenChanged { open: *open },
                PopupMenuEvent::Dismiss => SplitButtonEvent::Dismiss,
                PopupMenuEvent::FocusChanged { focused } => SplitButtonEvent::FocusChanged { focused: *focused },
                PopupMenuEvent::HoverChanged { hovered } => SplitButtonEvent::HoverChanged { hovered: *hovered },
                PopupMenuEvent::EnabledChanged { enabled } => SplitButtonEvent::EnabledChanged { enabled: *enabled },
            };
            cx.emit(event);
            cx.notify();
        });
        Self { popup, attachments: Default::default(), _events: events }
    }
    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_label(label, cx));
        cx.notify();
    }
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_enabled(enabled, cx));
        cx.notify();
    }
    /// Draw both faces as `:invalid`.
    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_invalid(invalid, cx));
        cx.notify();
    }

    /// Enable or disable only the primary action; menu access stays available.
    pub fn set_primary_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_primary_enabled(enabled, cx));
        cx.notify();
    }
    pub fn set_tab_stop(&mut self, tab_stop: bool, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_tab_stop(tab_stop, cx));
        cx.notify();
    }
    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = crate::infra::menu_item::MenuItem>,
        cx: &mut Context<Self>,
    ) {
        self.popup.update(cx, |popup, cx| popup.set_items(items, cx));
        cx.notify();
    }
    pub fn set_placement(&mut self, placement: PopupMenuPlacement, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_placement(placement, cx));
        cx.notify();
    }
    pub fn set_trigger_style(&mut self, style: PopupMenuTriggerStyle, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_trigger_style(style, cx));
        cx.notify();
    }
    pub fn set_presenter(
        &mut self,
        content: crate::infra::presenter::ControlPresenter<SplitButtonTriggerModel>,
        cx: &mut Context<Self>,
    ) {
        self.popup.update(cx, |popup, cx| popup.set_presenter(content, cx));
        cx.notify();
    }
    /// Replace primary face content with an icon while preserving split geometry.
    pub fn set_icon(&mut self, icon: impl Into<crate::controls::button::ControlIcon>, cx: &mut Context<Self>) {
        self.set_presenter(crate::controls::popup_menu::icon_content(icon), cx);
    }
    pub fn set_end_icon(&mut self, icon: Option<lucide_svg_static::Icon>, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.set_end_icon(icon, cx));
        cx.notify();
    }
    /// Activate the primary action, honoring global and primary-only disabling.
    pub fn activate(&mut self, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.activate_primary(cx));
    }
    /// Close an open menu, emitting OpenChanged(false) followed by Dismiss.
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| popup.dismiss(cx));
    }
    pub fn set_template(&mut self, template: std::sync::Arc<dyn SplitButtonTemplate>, cx: &mut Context<Self>) {
        self.popup.update(cx, |popup, cx| {
            popup.set_template(std::sync::Arc::new(super::template::EngineTemplate(template)), cx)
        });
        cx.notify();
    }
}
impl Focusable for SplitButton {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.popup.read(cx).focus_handle(cx)
    }
}
impl Render for SplitButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.popup.read(cx).attachment_state(window);
        let root = div().id(("split-button", cx.entity_id())).child(self.popup.clone());
        self.attachments.render(root, state, cx)
    }
}
impl AttachmentTarget for SplitButton {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
}
