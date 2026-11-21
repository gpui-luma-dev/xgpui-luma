mod callback;
pub use callback::{ToggleCallbacks, ToggleEventExt};

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::motion::{Transition, VisualTransition};
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::button::{Button, ButtonBuilder, ButtonContentContext, ButtonEvent, ButtonTemplate};
use crate::infra::presenter::{ControlPresenter, HasPresenter};

pub type Toggle = Entity<ToggleControl>;

/// Typed payload for [`Button<ToggleData>`] / [`ButtonTemplate<ToggleData>`].
///
/// `selected` is the settled semantic value; `progress` is the continuous visual factor
/// (`0.0`..`1.0`) driven by [`VisualTransition`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleData {
    pub selected: bool,
    pub progress: f32,
}

impl ToggleData {
    pub fn new(selected: bool) -> Self {
        Self { selected, progress: if selected { 1.0 } else { 0.0 } }
    }
}

impl Default for ToggleData {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ToggleEvent {
    Change { selected: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct ToggleControl {
    selected: bool,
    motion_enabled: bool,
    transition: VisualTransition,
    /// Set on this instance; replaces the template's `transition`.
    transition_override: Option<Transition>,
    /// The button's template, kept here so transitions resolve without reading the button.
    template: Arc<dyn ButtonTemplate<ToggleData>>,
    button: Entity<Button<ToggleData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ToggleEvent> for ToggleControl {}

impl ToggleControl {
    /// The transition toward `selected`: this instance's override, else the template's `transition`
    /// (`.lss`), else the SDK toggle's.
    fn transition_toward(
        template: &dyn ButtonTemplate<ToggleData>,
        transition_override: Option<&Transition>,
        selected: bool,
    ) -> Transition {
        let data = ToggleData::new(selected);
        transition_override
            .cloned()
            .or_else(|| template.transition(&data))
            .or_else(|| default_toggle_template().transition(&data))
            .unwrap_or_else(Transition::instant)
    }

    /// The transition the next selection change animates with.
    pub fn current_transition(&self) -> Transition {
        Self::transition_toward(self.template.as_ref(), self.transition_override.as_ref(), self.selected)
    }

    fn retarget(&mut self) {
        self.transition.set_transition(self.current_transition().enabled(self.motion_enabled));
        self.transition.set_target(if self.selected { 1.0 } else { 0.0 });
    }

    fn from_builder(builder: ToggleBuilder, cx: &mut Context<Self>) -> Self {
        let selected = builder.button.model.data.selected;
        let motion_enabled = builder.motion_enabled;
        let transition_override = builder.transition;
        let progress = if selected { 1.0 } else { 0.0 };
        let template = builder.button.model.template.clone();
        let initial = Self::transition_toward(template.as_ref(), transition_override.as_ref(), selected);
        let transition = VisualTransition::with_transition(progress, initial.enabled(motion_enabled));

        let mut button_builder = builder.button;
        button_builder.model.data = ToggleData { selected, progress };
        button_builder.model.role = ButtonFamilyRole::Toggle { selected };
        let button = button_builder.spawn(cx);
        button.update(cx, |button, _| {
            let host = crate::infra::attachments::AttachmentTarget::attachments_mut(button);
            host.set_accessible_role(gpui::Role::Button);
            host.set_checked(selected);
        });
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self {
            selected,
            motion_enabled,
            transition,
            transition_override,
            template,
            button,
            _subscriptions: vec![subscription],
        }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let data = ToggleData { selected: self.selected, progress: self.transition.progress() };
        let role = ButtonFamilyRole::Toggle { selected: self.selected };
        self.button.update(cx, |button, cx| {
            crate::infra::attachments::AttachmentTarget::attachments_mut(button).set_checked(self.selected);
            let role_changed = button.model.role != role;
            let data_changed = button.data() != &data;
            if !role_changed && !data_changed {
                return;
            }
            button.model.role = role;
            button.set_data(data, cx);
        });
    }

    pub fn set_data(&mut self, selected: bool, cx: &mut Context<Self>) {
        if self.selected == selected {
            return;
        }
        self.selected = selected;
        self.retarget();
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.selected
    }

    pub fn motion_enabled(&self) -> bool {
        self.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        self.retarget();
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(
        &mut self,
        content: ControlPresenter<ButtonContentContext<ToggleData>>,
        cx: &mut Context<Self>,
    ) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<ToggleData>>, cx: &mut Context<Self>) {
        self.template = Arc::clone(&template);
        self.transition.set_transition(self.current_transition().enabled(self.motion_enabled));
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<ToggleData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let selected = !self.selected;
                self.selected = selected;
                self.retarget();
                self.push_button_data(cx);
                cx.emit(ToggleEvent::Change { selected });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(ToggleEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(ToggleEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(ToggleEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for ToggleControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for ToggleControl {
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

impl IntoElement for ToggleControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

pub struct ToggleBuilder {
    callbacks: ToggleCallbacks,
    button: ButtonBuilder<ToggleData>,
    motion_enabled: bool,
    transition: Option<Transition>,
}

impl ToggleBuilder {
    /// Animate selection changes with `transition` instead of the template's `transition` (`.lss`).
    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
    }

    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ToggleEvent, &mut Context<M>) + 'static,
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
    pub fn with_callbacks(mut self, callbacks: ToggleCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn icon(self, icon: impl Into<crate::controls::button::ControlIcon>) -> Self {
        Self { button: self.button.icon(icon), ..self }
    }

    pub fn with_data(self, selected: bool) -> Self {
        Self { button: self.button.with_data(ToggleData::new(selected)), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { button: self.button.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { button: self.button.tab_stop(tab_stop), ..self }
    }

    pub fn role(self, role: ButtonFamilyRole) -> Self {
        Self { button: self.button.role(role), ..self }
    }

    pub fn round(self, round: bool) -> Self {
        Self { button: self.button.round(round), ..self }
    }

    pub fn without_elevation(self) -> Self {
        Self { button: self.button.without_elevation(), ..self }
    }

    pub fn compact(self) -> Self {
        Self { button: self.button.compact(), ..self }
    }

    /// Enables or disables the selected/unselected visual transition (default `true`).
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<ToggleData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Toggle {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = cx.new(|cx| ToggleControl::from_builder(self, cx));
        callbacks.bind(&control, cx);
        control
    }
}

impl HasPresenter<ButtonContentContext<ToggleData>> for ToggleBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<ToggleData>>) {
        self.button.set_presenter(content);
    }
}

fn button_builder(id: impl Into<SharedString>) -> ButtonBuilder<ToggleData> {
    ButtonBuilder::new(id)
        .typed(ToggleData::default())
        .role(ButtonFamilyRole::Toggle { selected: false })
        .template(default_toggle_template())
}

pub fn new(id: impl Into<SharedString>) -> ToggleBuilder {
    ToggleBuilder {
        callbacks: ToggleCallbacks::default(),
        button: button_builder(id),
        motion_enabled: true,
        transition: None,
    }
}

/// The SDK toggle: the `.lss` button template drawing the `toggle` control (issue #285).
pub fn default_toggle_template() -> Arc<dyn ButtonTemplate<ToggleData>> {
    static TEMPLATE: std::sync::OnceLock<Arc<dyn ButtonTemplate<ToggleData>>> = std::sync::OnceLock::new();
    TEMPLATE
        .get_or_init(|| Arc::new(crate::controls::button::StyledButtonTemplate::<ToggleData>::sdk()))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_builder_animated_option() {
        assert!(new("animated-default").motion_enabled);
        assert!(!new("animated-off").motion_enabled(false).motion_enabled);
    }

    #[test]
    fn toggle_data_seeds_progress_from_selected() {
        assert_eq!(ToggleData::new(true).progress, 1.0);
        assert_eq!(ToggleData::new(false).progress, 0.0);
    }
}

crate::infra::attachments::delegate_tooltips!(ToggleControl, gpui::Role::Button);

#[cfg(all(test, feature = "test-support"))]
mod callback_tests;

crate::theme::stylesheet::geometry::geometry_family!(
    ToggleStylesheet,
    ToggleGeometryStylesheet,
    ToggleGeometry,
    ResolvedToggleGeometry,
    "toggle",
    [height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);
