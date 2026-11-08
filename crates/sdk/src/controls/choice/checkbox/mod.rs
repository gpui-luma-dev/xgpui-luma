crate::motion_profile!(CheckboxMotion => checkbox {
    state: crate::motion::Transition,
});

mod callback;
pub use callback::{CheckboxCallbacks, CheckboxEventExt};

mod template;
pub(crate) mod theme;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use theme::{CheckboxLook, CheckboxPalette, CheckboxScale, CheckboxTheme, DefaultCheckboxTheme, default_checkbox_theme};

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::motion::VisualTransition;
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::button::{Button, ButtonBuilder, ButtonContentContext, ButtonEvent, ButtonTemplate};
use crate::infra::presenter::{ControlPresenter, HasPresenter};
use crate::infra::icon::SelectionStatusIcons;
use crate::theme::ControlSize;

pub type Checkbox = Entity<CheckboxControl>;

/// Typed payload for [`Button<CheckboxData>`] / [`ButtonTemplate<CheckboxData>`].
///
/// `checked` is the settled semantic value; `progress` is the continuous visual factor
/// (`0.0`..`1.0`) driven by [`VisualTransition`].
#[derive(Clone, Debug)]
pub struct CheckboxData {
    pub checked: bool,
    pub progress: f32,
    pub icons: SelectionStatusIcons,
}

impl PartialEq for CheckboxData {
    fn eq(&self, other: &Self) -> bool {
        self.checked == other.checked && self.progress == other.progress
    }
}

impl CheckboxData {
    pub fn new(checked: bool) -> Self {
        Self { checked, progress: if checked { 1.0 } else { 0.0 }, icons: SelectionStatusIcons::default() }
    }
}

impl Default for CheckboxData {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CheckboxEvent {
    Change { checked: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct CheckboxControl {
    checked: bool,
    motion_enabled: bool,
    transition: VisualTransition,
    motion: CheckboxMotion,
    button: Entity<Button<CheckboxData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<CheckboxEvent> for CheckboxControl {}

impl CheckboxControl {
    /// Motion roles captured when this control was created.
    pub fn motion_profile(&self) -> &CheckboxMotion {
        &self.motion
    }

    fn from_builder(builder: CheckboxBuilder, cx: &mut Context<Self>) -> Self {
        let checked = builder.button.model.data.checked;
        let motion_enabled = builder.motion_enabled;
        let motion = builder.motion_profile();
        let progress = if checked { 1.0 } else { 0.0 };
        let transition = VisualTransition::with_transition(progress, motion.state.enabled(motion_enabled));

        let mut button_builder = builder.button;
        button_builder.model.data = CheckboxData { checked, progress, icons: button_builder.model.data.icons.clone() };
        let button = button_builder.spawn(cx);
        button.update(cx, |button, _| {
            let host = crate::infra::attachments::AttachmentTarget::attachments_mut(button);
            host.set_accessible_role(gpui::Role::CheckBox);
            host.set_checked(checked);
        });
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { checked, motion_enabled, transition, motion, button, _subscriptions: vec![subscription] }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let icons = self.button.read(cx).data().icons.clone();
        let data = CheckboxData { checked: self.checked, progress: self.transition.progress(), icons };
        self.button.update(cx, |button, cx| {
            crate::infra::attachments::AttachmentTarget::attachments_mut(button).set_checked(self.checked);
            if button.data() == &data {
                return;
            }
            button.set_data(data, cx);
        });
    }

    pub fn set_data(&mut self, checked: bool, cx: &mut Context<Self>) {
        if self.checked == checked {
            return;
        }
        self.checked = checked;
        self.transition.set_target(if checked { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.checked
    }

    pub fn motion_enabled(&self) -> bool {
        self.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        self.transition.set_transition(self.motion.state.enabled(motion_enabled));
        self.transition.set_target(if self.checked { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(
        &mut self,
        content: ControlPresenter<ButtonContentContext<CheckboxData>>,
        cx: &mut Context<Self>,
    ) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<CheckboxData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<CheckboxData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let checked = !self.checked;
                self.checked = checked;
                self.transition.set_target(if checked { 1.0 } else { 0.0 });
                self.push_button_data(cx);
                cx.emit(CheckboxEvent::Change { checked });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(CheckboxEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(CheckboxEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(CheckboxEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for CheckboxControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for CheckboxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_animating = self.transition.is_animating();
        let _ = self.transition.sync();
        self.push_button_data(cx);
        self.transition.schedule_frame(window, cx);
        if was_animating || self.transition.is_animating() {
            cx.notify();
        }

        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for CheckboxControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

/// Builder for [`Checkbox`] controls.
///
/// Apps should spawn through a look-owned builder (for example
/// `xgpui_luma_look_shadcn::Checkbox::new(...)`) rather than constructing this type
/// directly or styling [`ButtonBuilder<CheckboxData>`] as a switch.
pub struct CheckboxBuilder {
    callbacks: CheckboxCallbacks,
    button: ButtonBuilder<CheckboxData>,
    motion_enabled: bool,
    motion: crate::motion::MotionConfig<CheckboxMotion>,
    icons_explicit: bool,
    icons: SelectionStatusIcons,
}

impl CheckboxBuilder {
    crate::motion_methods!(CheckboxMotion, motion);

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &CheckboxEvent, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Append a user-change callback. `set_data` is silent; the owner is held weakly.
    pub fn on_value_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_value_changed(cx, callback);
        self
    }

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_focus_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_focus_changed(cx, callback);
        self
    }

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_hover_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_hover_changed(cx, callback);
        self
    }

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_enabled_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_enabled_changed(cx, callback);
        self
    }

    /// Transfer callbacks from a Look-owned builder.
    #[doc(hidden)]
    pub fn with_callbacks(mut self, callbacks: CheckboxCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn with_data(self, checked: bool) -> Self {
        Self { button: self.button.with_data(CheckboxData::new(checked)), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { button: self.button.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { button: self.button.tab_stop(tab_stop), ..self }
    }

    pub fn size(self, size: ControlSize) -> Self {
        Self { button: self.button.size(size), ..self }
    }

    /// Renders only the checkbox indicator (no label slot). Use in tables and list rows.
    pub fn indicator_only(self) -> Self {
        Self { button: self.button.role(ButtonFamilyRole::Icon), ..self }
    }

    pub fn without_elevation(self) -> Self {
        Self { button: self.button.without_elevation(), ..self }
    }

    pub fn compact(self) -> Self {
        Self { button: self.button.compact(), ..self }
    }

    /// Enables or disables the checked/unchecked visual transition (default `true`).
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    /// Capture shared selection glyphs without replacing explicit icons.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.icons_explicit {
            self.icons = SelectionStatusIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.icons = icons.clone();
        self.button.model.data.icons = icons;
        self
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<CheckboxData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Checkbox {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = cx.new(|cx| CheckboxControl::from_builder(self, cx));
        callbacks.bind(&control, cx);
        control
    }
}

impl HasPresenter<ButtonContentContext<CheckboxData>> for CheckboxBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<CheckboxData>>) {
        self.button.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> CheckboxBuilder {
    CheckboxBuilder {
        callbacks: CheckboxCallbacks::default(),
        button: ButtonBuilder::new(id).typed(CheckboxData::default()).template(default_checkbox_template()),
        motion_enabled: true,
        motion: Default::default(),
        icons_explicit: false,
        icons: SelectionStatusIcons::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_builder_animated_option() {
        assert!(new("animated-default").motion_enabled);
        assert!(!new("animated-off").motion_enabled(false).motion_enabled);
    }

    #[test]
    fn checkbox_data_seeds_progress_from_checked() {
        assert_eq!(CheckboxData::new(true).progress, 1.0);
        assert_eq!(CheckboxData::new(false).progress, 0.0);
    }
}

crate::infra::attachments::delegate_tooltips!(CheckboxControl, gpui::Role::CheckBox);

#[cfg(all(test, feature = "test-support"))]
mod callback_tests;
