//! Look-owned radio builder. Spawn synthesizes the SDK [`xgpui_luma::controls::radio_button::RadioButton`].

use xgpui_luma::infra::attachments::TooltipEntityExt;
use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::radio_button::{RadioButtonBuilder, RadioButtonData};

use crate::button::Paint;
use crate::look::{Look, resolve_look};
use crate::radio::{RadioSize, RadioVariant, radio_template_for};
use crate::tone::Tone;

/// Builder in the guise of a radio: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Radio {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::radio_button::RadioMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::radio_button::RadioButtonEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    selected: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    motion_enabled: bool,
    role: Option<ButtonFamilyRole>,
    content: Option<ControlPresenter<ButtonContentContext<RadioButtonData>>>,
}

impl Radio {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::radio_button::RadioButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `RadioButtonEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_button::RadioButtonEvent::Change { selected } = event {
                callback(owner, *selected, cx);
            }
        })
    }

    /// Handle `RadioButtonEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_button::RadioButtonEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `RadioButtonEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_button::RadioButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    /// Handle `RadioButtonEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::radio_button::RadioButtonEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            motion: Default::default(),
            variant: RadioVariant::default(),
            paint: Paint::accent(),
            size: RadioSize::default(),
            selected: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            motion_enabled: true,
            role: None,
            content: None,
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::radio_button::RadioMotion, edits motion);

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: RadioVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(RadioVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(RadioVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(RadioVariant::Soft)
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

    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }

    /// SDK payload seam (`selected`). Radio is not generic over `D`.
    pub fn with_data(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Same field as [`Self::with_data`].
    pub fn selected(self, selected: bool) -> Self {
        self.with_data(selected)
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

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::radio_button::RadioButton {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            let theme = crate::tooltip::tooltip_theme(&look);
            self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> RadioButtonBuilder {
        let template = radio_template_for(&look, self.variant, self.paint, self.size);
        let mut builder = xgpui_luma::controls::radio_button::new(self.id)
            .motion_defaults(look.motion_defaults().radio)
            .with_motion_overrides(self.motion)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.selected)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .motion_enabled(self.motion_enabled);
        if let Some(role) = self.role {
            builder = builder.role(role);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<RadioButtonData>> for Radio {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<RadioButtonData>>) {
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
        assert_eq!(Radio::new("s").size(RadioSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Radio::new("s").size(RadioSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Radio::new("s").size(RadioSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_radix_axes() {
        let radio = Radio::new("opt").soft().gray().size(RadioSize::One).with_data(true);
        assert!(matches!(radio.variant, RadioVariant::Soft));
        assert_eq!(radio.paint.tone, Tone::Gray);
        assert_eq!(radio.size, RadioSize::One);
        assert!(radio.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Radio::new("opt").selected(true).selected);
        assert!(Radio::new("opt").with_data(true).selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Radio::new("ok")
            .look(&look)
            .classic()
            .high_contrast(true)
            .with_data(true)
            .label("A")
            .into_sdk_builder(look);
    }
}
