mod callback;
pub use callback::{SwitchCallbacks, SwitchEventExt};

mod template;
pub(crate) mod theme;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};
pub use theme::{DefaultSwitchTheme, SwitchLook, SwitchPalette, SwitchScale, SwitchTheme, default_switch_theme};

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Pixels, Render, SharedString, Subscription, Window, div,
};

use crate::motion::{Transition, VisualTransition};
use crate::controls::button_family::ButtonSize;
use crate::controls::button::{Button, ButtonBuilder, ButtonContentContext, ButtonEvent, ButtonTemplate};
use crate::infra::presenter::{ControlPresenter, HasPresenter};

pub type Switch = Entity<SwitchControl>;

// State progress drives thumb position and bounded colors.
crate::motion_profile!(SwitchMotion => switch {
    state: Transition,
});

/// Typed payload for [`Button<SwitchData>`] / [`ButtonTemplate<SwitchData>`].
///
/// `checked` is the settled semantic value; `progress` is the continuous visual factor
/// driven by [`VisualTransition`]. Springs may overshoot the endpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchData {
    pub checked: bool,
    pub progress: f32,
}

impl SwitchData {
    pub fn new(checked: bool) -> Self {
        Self { checked, progress: if checked { 1.0 } else { 0.0 } }
    }
}

impl Default for SwitchData {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SwitchEvent {
    Change { on: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct SwitchControl {
    on: bool,
    motion_enabled: bool,
    transition: VisualTransition,
    motion: SwitchMotion,
    button: Entity<Button<SwitchData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SwitchEvent> for SwitchControl {}

impl SwitchControl {
    /// Motion roles captured when this control was created.
    pub fn motion_profile(&self) -> &SwitchMotion {
        &self.motion
    }

    fn from_builder(builder: SwitchBuilder, cx: &mut Context<Self>) -> Self {
        let on = builder.button.model.data.checked;
        let motion_enabled = builder.motion_enabled;
        let motion = builder.motion_profile();
        let progress = if on { 1.0 } else { 0.0 };
        let transition = VisualTransition::with_transition(
            progress,
            if motion_enabled {
                motion.state.clone()
            } else {
                Transition::instant()
            },
        );

        let mut button_builder = builder.button;
        button_builder.model.data = SwitchData { checked: on, progress };
        let button = button_builder.spawn(cx);
        button.update(cx, |button, _| {
            let host = crate::infra::attachments::AttachmentTarget::attachments_mut(button);
            host.set_accessible_role(gpui::Role::Switch);
            host.set_checked(on);
        });
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { on, motion_enabled, transition, motion, button, _subscriptions: vec![subscription] }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let data = SwitchData { checked: self.on, progress: self.transition.progress() };
        self.button.update(cx, |button, cx| {
            crate::infra::attachments::AttachmentTarget::attachments_mut(button).set_checked(self.on);
            if button.data() == &data {
                return;
            }
            button.set_data(data, cx);
        });
    }

    pub fn set_data(&mut self, on: bool, cx: &mut Context<Self>) {
        if self.on == on {
            return;
        }
        self.on = on;
        self.transition.set_target(if on { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.on
    }

    pub fn motion_enabled(&self) -> bool {
        self.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        self.transition.set_transition(if motion_enabled {
            self.motion.state.clone()
        } else {
            Transition::instant()
        });
        self.transition.set_target(if self.on { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(
        &mut self,
        content: ControlPresenter<ButtonContentContext<SwitchData>>,
        cx: &mut Context<Self>,
    ) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<SwitchData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    pub fn set_switch_orientation(&mut self, orientation: SwitchOrientation, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_switch_orientation(orientation, cx));
        cx.notify();
    }

    pub fn set_switch_track_length_extra(&mut self, extra_length: f32, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_switch_track_length_extra(extra_length, cx));
        cx.notify();
    }

    pub fn set_switch_track_width_extra(&mut self, extra_width: f32, cx: &mut Context<Self>) {
        self.set_switch_track_length_extra(extra_width, cx);
    }

    pub fn set_switch_track_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.update(cx, |button, cx| button.set_switch_track_content(builder, cx));
        cx.notify();
    }

    pub fn set_switch_thumb_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.update(cx, |button, cx| button.set_switch_thumb_content(builder, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<SwitchData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let on = !self.on;
                self.on = on;
                self.transition.set_target(if on { 1.0 } else { 0.0 });
                self.push_button_data(cx);
                cx.emit(SwitchEvent::Change { on });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(SwitchEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(SwitchEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(SwitchEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for SwitchControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for SwitchControl {
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

impl IntoElement for SwitchControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Builder for [`Switch`] controls.
///
/// Apps should spawn through a look-owned builder (for example
/// `xgpui_luma_look_shadcn::Switch::new(...)`) rather than constructing this type
/// directly or styling [`ButtonBuilder<SwitchData>`] as a checkbox.
pub struct SwitchBuilder {
    callbacks: SwitchCallbacks,
    button: ButtonBuilder<SwitchData>,
    motion_enabled: bool,
    motion: crate::motion::MotionConfig<SwitchMotion>,
}

impl SwitchBuilder {
    crate::motion_methods!(SwitchMotion, motion);

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &SwitchEvent, &mut Context<M>) + 'static,
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
    pub fn with_callbacks(mut self, callbacks: SwitchCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// Render only the switch track, without the label or its spacing.
    pub fn without_label(self) -> Self {
        Self { button: self.button.role(crate::controls::button_family::ButtonFamilyRole::Icon), ..self }
    }

    pub fn with_data(self, checked: bool) -> Self {
        Self { button: self.button.with_data(SwitchData::new(checked)), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { button: self.button.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { button: self.button.tab_stop(tab_stop), ..self }
    }

    pub fn without_elevation(self) -> Self {
        Self { button: self.button.without_elevation(), ..self }
    }

    pub fn compact(self) -> Self {
        Self { button: self.button.compact(), ..self }
    }

    /// Enables or disables the on/off visual transition (default `true`).
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn orientation(mut self, orientation: SwitchOrientation) -> Self {
        self.button.model.switch_orientation = orientation;
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SwitchOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SwitchOrientation::Vertical)
    }

    /// Adds extra length along the switch track's movement axis.
    pub fn track_length_extra(mut self, extra_length: f32) -> Self {
        self.button.model.switch_track_width_extra = extra_length.max(0.0);
        self
    }

    /// Adds extra width to a horizontal switch track for interior content such as ON/OFF labels.
    pub fn track_width_extra(self, extra_width: f32) -> Self {
        self.track_length_extra(extra_width)
    }

    /// Sets fixed logical-pixel geometry for a special-purpose switch composition.
    /// Semantic switch sizes remain the preferred API for ordinary controls.
    pub fn fixed_geometry(
        mut self,
        track_length: impl Into<Pixels>,
        track_thickness: impl Into<Pixels>,
        thumb_size: impl Into<Pixels>,
    ) -> Self {
        self.button.model.switch_track_width = Some(f32::from(track_length.into()).max(0.0));
        self.button.model.switch_track_height = Some(f32::from(track_thickness.into()).max(0.0));
        self.button.model.switch_thumb_size = Some(f32::from(thumb_size.into()).max(0.0));
        self
    }

    /// Renders content inside the switch track behind the thumb.
    pub fn track_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    /// Renders content inside the moving switch thumb.
    pub fn thumb_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    pub fn size(self, size: ButtonSize) -> Self {
        Self { button: self.button.size(size), ..self }
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<SwitchData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Switch {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = cx.new(|cx| SwitchControl::from_builder(self, cx));
        callbacks.bind(&control, cx);
        control
    }
}

impl HasPresenter<ButtonContentContext<SwitchData>> for SwitchBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<SwitchData>>) {
        self.button.set_presenter(content);
    }
}

impl Button<SwitchData> {
    pub fn set_switch_orientation(&mut self, orientation: SwitchOrientation, cx: &mut Context<Self>) {
        self.model.switch_orientation = orientation;
        cx.notify();
    }

    pub fn set_switch_track_length_extra(&mut self, extra_length: f32, cx: &mut Context<Self>) {
        self.model.switch_track_width_extra = extra_length.max(0.0);
        cx.notify();
    }

    pub fn set_switch_track_width_extra(&mut self, extra_width: f32, cx: &mut Context<Self>) {
        self.set_switch_track_length_extra(extra_width, cx);
    }

    pub fn set_switch_track_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }

    pub fn set_switch_thumb_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonContentContext<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }
}

pub fn new(id: impl Into<SharedString>) -> SwitchBuilder {
    SwitchBuilder {
        callbacks: SwitchCallbacks::default(),
        button: ButtonBuilder::new(id).typed(SwitchData::default()).template(default_switch_template()),
        motion_enabled: true,
        motion: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn switch_without_label_preserves_checked_state() {
        let builder = new("unlabeled").with_data(true).without_label();
        assert_eq!(builder.button.model.role, crate::controls::button_family::ButtonFamilyRole::Icon);
        assert_eq!(builder.button.model.data, SwitchData::new(true));
        assert_eq!(
            new("labeled").label("Notifications").button.model.role,
            crate::controls::button_family::ButtonFamilyRole::Text
        );
    }

    #[test]
    fn switch_builder_sets_slot_content_track_length_and_orientation() {
        let builder = new("slot-switch")
            .vertical()
            .track_length_extra(12.0)
            .track_content(|_, _| div())
            .thumb_content(|_, _| div());

        assert_eq!(builder.button.model.switch_track_width_extra, 12.0);
        assert_eq!(builder.button.model.switch_orientation, SwitchOrientation::Vertical);
        assert!(builder.button.model.switch_track_content.is_some());
        assert!(builder.button.model.switch_thumb_content.is_some());
    }

    #[test]
    fn switch_builder_animated_option() {
        assert!(new("animated-default").motion_enabled);
        assert!(!new("animated-off").motion_enabled(false).motion_enabled);
    }

    #[test]
    fn switch_data_seeds_progress_from_checked() {
        assert_eq!(SwitchData::new(true).progress, 1.0);
        assert_eq!(SwitchData::new(false).progress, 0.0);
    }
}

crate::infra::attachments::delegate_tooltips!(SwitchControl, gpui::Role::Switch);

#[cfg(all(test, feature = "test-support"))]
mod callback_tests;
