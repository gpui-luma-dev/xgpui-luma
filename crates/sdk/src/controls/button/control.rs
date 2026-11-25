use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, Render, SharedString, Subscription, Window, div, prelude::*,
};

pub use crate::infra::presenter::{ControlPresenter, HasPresenter};

use super::{ButtonBuilder, ButtonContentContext, ButtonRenderModel};
pub use super::core::{CommandCore, CommandEvent as ButtonEvent};
use crate::key_handling::{ActivateControl, ControlKeyProfile};
use super::model::ControlIcon;
use crate::controls::button_family::ButtonFamilyRole;
use crate::theme::observe_theme_revision;

pub struct Button<D = ()> {
    pub(crate) model: super::model::ButtonModel<D>,
    command: CommandCore,
    attachments: AttachmentHost,
    pub(super) callback_subscriptions: Vec<Subscription>,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
}

impl<D: 'static> EventEmitter<ButtonEvent> for Button<D> {}

impl Button<()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<()> {
        ButtonBuilder::new(id)
    }

    pub fn icon(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        let icon = icon.into();
        let mut builder = ButtonBuilder::new(id).role(ButtonFamilyRole::Icon).round(true);
        builder.model.icon = Some(icon.clone());
        builder.content(|_, _| gpui::div().into_any_element())
    }
}

impl<D: Clone + 'static> Button<D> {
    pub(crate) fn from_builder(builder: ButtonBuilder<D>, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let tab_stop = builder.model.tab_stop;
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            attachments: AttachmentHost::with_accessible_role(gpui::Role::Button),
            command: CommandCore::new_with_tab_stop(enabled, tab_stop, cx),
            callback_subscriptions: Vec::new(),
            focus_in_subscription: None,
            focus_out_subscription: None,
        }
    }

    pub fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<D>>, cx: &mut Context<Self>) {
        self.model.content = content;
        cx.notify();
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>)
    where
        D: Default + 'static,
    {
        let label = label.into();
        self.model.content = Arc::new(move |_, _| div().child(label.clone()).into_any_element());
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::template::ButtonTemplate<D>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_data(&mut self, data: D, cx: &mut Context<Self>) {
        self.model.data = data;
        cx.notify();
    }

    /// Whether the button takes input.
    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        if self.command.set_enabled(enabled, cx) {
            cx.notify();
        }
    }

    pub fn data(&self) -> &D {
        &self.model.data
    }

    pub fn render_model(&self, window: &Window) -> ButtonRenderModel<D> {
        ButtonRenderModel {
            id: self.model.id.clone(),
            data: self.model.data.clone(),
            content: self.model.content.clone(),
            icon: self.model.icon.clone(),
            role: self.model.role,
            state: self.command.render_state(self.model.enabled, window),
            round: self.model.round,
            radius_override: std::cell::Cell::new(None),
            elevation: self.model.elevation,
            compact: self.model.compact,
            switch_track_width_extra: self.model.switch_track_width_extra,
            switch_track_width: self.model.switch_track_width,
            switch_track_height: self.model.switch_track_height,
            switch_thumb_size: self.model.switch_thumb_size,
            switch_orientation: self.model.switch_orientation,
            switch_track_content: self.model.switch_track_content.clone(),
            switch_thumb_content: self.model.switch_thumb_content.clone(),
            look: self.model.look.clone(),
        }
    }

    fn handle_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && !event.is_keyboard() {
            self.attachments.dismiss(cx);
        }
        self.command.handle_click(self.model.enabled, event, cx);
    }

    fn handle_activate_control(&mut self, event: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled {
            self.attachments.dismiss(cx);
        }
        self.command.handle_activate_control(self.model.enabled, event, cx);
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_hover(self.model.enabled, *hovered, cx) {
            cx.notify();
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_focus_changed(self.model.enabled, true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_focus_changed(self.model.enabled, false, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_mouse_down(self.model.enabled, event, window, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_mouse_up(event) {
            cx.notify();
        }
    }
}

impl<D: 'static> Focusable for Button<D> {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.command.focus_handle().clone()
    }
}

impl<D: Clone + 'static> Render for Button<D> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.command.focus_handle().clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.command.focus_handle().clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let model = self.render_model(window);

        div()
            .child(
                self.attachments
                    .render(self.model.template.render(&model, window, cx), model.state, cx)
                    .track_focus(self.command.focus_handle())
                    .key_context(ControlKeyProfile::Command.context())
                    .on_action(cx.listener(Self::handle_activate_control))
                    .on_hover(cx.listener(Self::handle_hover))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_click(cx.listener(Self::handle_click)),
            )
            .into_any_element()
    }
}

impl<D: Clone + 'static> IntoElement for Button<D> {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

impl<D: Clone + 'static> From<Button<D>> for AnyElement {
    fn from(button: Button<D>) -> Self {
        button.into_element()
    }
}

impl<D: 'static> AttachmentTarget for Button<D> {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "callback_tests.rs"]
mod callback_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "perf_tests.rs"]
mod perf_tests;
