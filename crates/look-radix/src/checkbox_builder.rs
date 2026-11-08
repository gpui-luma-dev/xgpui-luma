//! Look-owned checkbox builder. Spawn synthesizes the SDK [`xgpui_luma::controls::checkbox::Checkbox`].

use xgpui_luma::controls::checkbox::{CheckboxCallbacks, CheckboxEvent};
use xgpui_luma::infra::attachments::TooltipEntityExt;
use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use xgpui_luma::controls::checkbox::{CheckboxBuilder, CheckboxData};
use xgpui_luma::infra::icon::SelectionStatusIcons;

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::checkbox::{CheckboxSize, CheckboxVariant, checkbox_template_for};
use crate::look::{Look, resolve_look};
use crate::tone::Tone;

/// Builder in the guise of a checkbox: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Checkbox {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::checkbox::CheckboxMotion>,
    callbacks: CheckboxCallbacks,
    id: SharedString,
    look: Option<Look>,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
    checked: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    indicator_only: bool,
    motion_enabled: bool,
    icons: Option<SelectionStatusIcons>,
    content: Option<ControlPresenter<ButtonContentContext<CheckboxData>>>,
}

impl Checkbox {
    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &CheckboxEvent, &mut Context<M>) + 'static,
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
            callbacks: CheckboxCallbacks::default(),
            id: id.into(),
            look: None,
            motion: Default::default(),
            variant: CheckboxVariant::default(),
            paint: Paint::accent(),
            size: CheckboxSize::default(),
            radius: Radius::default(),
            checked: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            indicator_only: false,
            motion_enabled: true,
            icons: None,
            content: None,
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::checkbox::CheckboxMotion, edits motion);

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: CheckboxVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(CheckboxVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(CheckboxVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(CheckboxVariant::Soft)
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

    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }

    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// SDK payload seam (`checked`). Checkbox is not generic over `D`.
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

    pub fn indicator_only(mut self) -> Self {
        self.indicator_only = true;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.icons = Some(icons);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::checkbox::Checkbox {
        let look = self.resolve_look(cx);
        let theme = crate::tooltip::tooltip_theme(&look);
        self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> CheckboxBuilder {
        let template = checkbox_template_for(&look, self.variant, self.paint, self.size, self.radius);
        let mut builder = xgpui_luma::controls::checkbox::new(self.id)
            .stylesheet(&look.common_stylesheet())
            .motion_defaults(look.motion_defaults().checkbox)
            .with_motion_overrides(self.motion)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.checked)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .motion_enabled(self.motion_enabled)
            .with_callbacks(self.callbacks);
        if self.indicator_only {
            builder = builder.indicator_only();
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(icons) = self.icons {
            builder = builder.icons(icons);
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<CheckboxData>> for Checkbox {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<CheckboxData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button::HasPresenter;
    use xgpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Checkbox::new("s").size(CheckboxSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Checkbox::new("s").size(CheckboxSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Checkbox::new("s").size(CheckboxSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_radix_axes() {
        let box_ = Checkbox::new("task").soft().gray().size(CheckboxSize::Three).with_data(true);
        assert!(matches!(box_.variant, CheckboxVariant::Soft));
        assert_eq!(box_.paint.tone, Tone::Gray);
        assert_eq!(box_.size, CheckboxSize::Three);
        assert!(box_.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Checkbox::new("task").checked(true).checked);
        assert!(Checkbox::new("task").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Checkbox::new("ok")
            .look(&look)
            .classic()
            .high_contrast(true)
            .with_data(true)
            .label("Done")
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod callback_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    struct Owner {
        control: xgpui_luma::controls::checkbox::Checkbox,
        log: Vec<bool>,
    }
    #[test]
    fn look_builder_preserves_every_callback_and_payload() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let control = Checkbox::new("choice")
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
                cx.emit(CheckboxEvent::Change { checked: true });
                cx.emit(CheckboxEvent::FocusChanged { focused: true });
                cx.emit(CheckboxEvent::HoverChanged { hovered: true });
                cx.emit(CheckboxEvent::EnabledChanged { enabled: true });
            })
        });
        app.read(|cx| assert_eq!(owner.read(cx).log, [false, true, false, true, false, true, false, true]));
    }
}
