//! Look-owned radio builder. Spawn synthesizes the SDK [`xgpui_luma::controls::radio_button::RadioButton`].

use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::radio_button::{RadioButtonBuilder, RadioButtonData};
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a radio: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Radio {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::radio_button::RadioMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::radio_button::RadioButtonEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
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
            style: ShadcnButtonStyle::Primary,
            size: ShadcnSize::Md,
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
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn style(mut self, style: ShadcnButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn primary(self) -> Self {
        self.style(ShadcnButtonStyle::Primary)
    }

    pub fn secondary(self) -> Self {
        self.style(ShadcnButtonStyle::Secondary)
    }

    pub fn outline(self) -> Self {
        self.style(ShadcnButtonStyle::Outline)
    }

    pub fn ghost(self) -> Self {
        self.style(ShadcnButtonStyle::Ghost)
    }

    pub fn content_only(self) -> Self {
        self.style(ShadcnButtonStyle::ContentOnly).without_elevation()
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
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
            use xgpui_luma::infra::attachments::TooltipEntityExt;
            let theme = crate::tooltip_theme(&look);
            self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> RadioButtonBuilder {
        let template = look.radio_button_template(self.style);
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

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let radio = Radio::new("opt").outline().size(ShadcnSize::Sm).with_data(true);
        assert!(matches!(radio.style, ShadcnButtonStyle::Outline));
        assert_eq!(radio.size, ShadcnSize::Sm);
        assert!(radio.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Radio::new("opt").selected(true).selected);
        assert!(Radio::new("opt").with_data(true).selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Radio::new("ok").look(&look).primary().with_data(true).label("A").into_sdk_builder(look);
    }
}
