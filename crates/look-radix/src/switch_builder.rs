//! Look-owned switch builder. Spawn synthesizes the SDK [`xgpui_luma::controls::switch::Switch`].

use xgpui_luma::controls::switch::{SwitchCallbacks, SwitchEvent};
use xgpui_luma::infra::attachments::TooltipEntityExt;
use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use xgpui_luma::controls::switch::{SwitchBuilder, SwitchData, SwitchOrientation};

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::{Look, resolve_look};
use crate::switch::{SwitchSize, SwitchVariant, switch_template_for};
use crate::tone::Tone;

/// Builder in the guise of a switch: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Switch {
    callbacks: SwitchCallbacks,
    id: SharedString,
    look: Option<Look>,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
    checked: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    motion_enabled: bool,
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::switch::SwitchMotion>,
    orientation: Option<SwitchOrientation>,
    content: Option<ControlPresenter<ButtonContentContext<SwitchData>>>,
}

impl Switch {
    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &SwitchEvent, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Append an owner-bound callback; registrations are additive and owners are weak.
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

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: SwitchCallbacks::default(),
            id: id.into(),
            look: None,
            variant: SwitchVariant::default(),
            paint: Paint::accent(),
            size: SwitchSize::default(),
            radius: Radius::default(),
            checked: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            motion_enabled: true,
            motion: Default::default(),
            orientation: None,
            content: None,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: SwitchVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(SwitchVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(SwitchVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(SwitchVariant::Soft)
    }

    pub fn accent(mut self) -> Self {
        self.paint.tone = Tone::Accent;
        self
    }

    pub fn gray(mut self) -> Self {
        self.paint.tone = Tone::Gray;
        self
    }

    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.paint.high_contrast = high_contrast;
        self
    }

    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// SDK payload seam (`checked`). Switch is not generic over `D`.
    pub fn with_data(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Same field as [`Self::with_data`].
    pub fn checked(self, checked: bool) -> Self {
        self.with_data(checked)
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.without_elevation = true;
        self
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::switch::SwitchMotion, edits motion);

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn orientation(mut self, orientation: SwitchOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SwitchOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SwitchOrientation::Vertical)
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::switch::Switch {
        let look = self.resolve_look(cx);
        let theme = crate::tooltip::tooltip_theme(&look);
        self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> SwitchBuilder {
        let template = switch_template_for(&look, self.variant, self.paint, self.size, self.radius);
        let mut builder = xgpui_luma::controls::switch::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.checked)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .motion_enabled(self.motion_enabled)
            .motion_defaults(look.motion_defaults().switch)
            .with_motion_overrides(self.motion)
            .with_callbacks(self.callbacks);
        if let Some(orientation) = self.orientation {
            builder = builder.orientation(orientation);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        } else {
            builder = builder.without_label();
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<SwitchData>> for Switch {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<SwitchData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button::HasPresenter;
    use xgpui_luma::theme::ControlSize;

    #[test]
    fn independent_look_copy_keeps_its_motion_defaults() {
        use xgpui_luma::motion::{Easing, MotionState, Transition};
        use std::time::Duration;
        let look = Look::built_in();
        look.motion(|m| m.switch.state = Transition::timed(Duration::from_millis(400), Easing::Linear));
        let copy = look.fork();
        look.motion(|m| m.switch.state = Transition::instant());
        assert_eq!(
            copy.motion_defaults()
                .switch
                .state
                .start(MotionState::default(), 1.0)
                .sample(Duration::from_millis(100))
                .state
                .value,
            0.25
        );
        assert!(
            look.motion_defaults()
                .switch
                .state
                .start(MotionState::default(), 1.0)
                .sample(Duration::ZERO)
                .complete
        );
    }

    #[test]
    fn switch_inherits_look_motion_and_forwards_sdk_overrides() {
        use xgpui_luma::motion::{Easing, MotionState, Transition};
        use std::time::Duration;
        let look = Look::built_in();
        look.motion(|m| m.switch.state = Transition::timed(Duration::from_millis(400), Easing::Linear));
        let inherited = Switch::new("inherited").into_sdk_builder(look.clone());
        let edited =
            Switch::new("edited").motion(|m| m.state = Transition::instant()).look(&look).into_sdk_builder(look);
        let sample = inherited
            .motion_profile()
            .state
            .start(MotionState::default(), 1.0)
            .sample(Duration::from_millis(100));
        assert_eq!(sample.state.value, 0.25);
        assert!(edited.motion_profile().state.start(MotionState::default(), 1.0).sample(Duration::ZERO).complete);
    }

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Switch::new("s").size(SwitchSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Switch::new("s").size(SwitchSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Switch::new("s").size(SwitchSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_radix_axes() {
        let switch = Switch::new("notify").soft().gray().size(SwitchSize::Three).with_data(true);
        assert!(matches!(switch.variant, SwitchVariant::Soft));
        assert_eq!(switch.paint.tone, Tone::Gray);
        assert_eq!(switch.size, SwitchSize::Three);
        assert!(switch.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Switch::new("notify").checked(true).checked);
        assert!(Switch::new("notify").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Switch::new("ok")
            .look(&look)
            .classic()
            .high_contrast(true)
            .with_data(true)
            .label("On")
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod callback_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    struct Owner {
        control: xgpui_luma::controls::switch::Switch,
        log: Vec<bool>,
    }
    #[test]
    fn look_builder_preserves_every_callback_and_payload() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let control = Switch::new("choice")
                    .on_event(cx, |owner: &mut Owner, _, _| owner.log.push(false))
                    .on_value_changed(cx, |owner, value, _| owner.log.push(value))
                    .on_focus_changed(cx, |owner, value, _| owner.log.push(value))
                    .on_hover_changed(cx, |owner, value, _| owner.log.push(value))
                    .on_enabled_changed(cx, |owner, value, _| owner.log.push(value))
                    .with_data(true)
                    .spawn(cx);
                Owner { control, log: vec![] }
            })
        });
        let control = app.read(|cx| owner.read(cx).control.clone());
        app.read(|cx| assert!(*control.read(cx).data()));
        app.update(|cx| {
            control.update(cx, |_, cx| {
                cx.emit(SwitchEvent::Change { on: true });
                cx.emit(SwitchEvent::FocusChanged { focused: true });
                cx.emit(SwitchEvent::HoverChanged { hovered: true });
                cx.emit(SwitchEvent::EnabledChanged { enabled: true });
            })
        });
        app.read(|cx| assert_eq!(owner.read(cx).log, [false, true, false, true, false, true, false, true]));
    }
}
