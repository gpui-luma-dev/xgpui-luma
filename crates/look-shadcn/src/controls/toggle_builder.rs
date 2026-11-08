//! Look-owned toggle builder. Spawn synthesizes the SDK [`xgpui_luma::controls::toggle::Toggle`].

use xgpui_luma::controls::toggle::{ToggleCallbacks, ToggleEvent};
use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlIcon, ControlPresenter, HasPresenter};
use xgpui_luma::controls::toggle::{ToggleBuilder, ToggleData};
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a toggle: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Toggle {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::toggle::ToggleMotion>,
    callbacks: ToggleCallbacks,
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
    round: bool,
    icon: Option<ControlIcon>,
    content: Option<ControlPresenter<ButtonContentContext<ToggleData>>>,
}

impl Toggle {
    /// Append an owner-bound callback; registrations are additive and owners are weak.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ToggleEvent, &mut Context<M>) + 'static,
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
            callbacks: ToggleCallbacks::default(),
            id: id.into(),
            look: None,
            motion: Default::default(),
            style: ShadcnButtonStyle::Secondary,
            size: ShadcnSize::Md,
            selected: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            motion_enabled: true,
            round: false,
            icon: None,
            content: None,
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::toggle::ToggleMotion, edits motion);

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

    /// SDK payload seam (`selected`). Toggle is not generic over `D`.
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

    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::toggle::Toggle {
        let look = self.resolve_look(cx);
        use xgpui_luma::infra::attachments::TooltipEntityExt;
        let theme = crate::tooltip_theme(&look);
        self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ToggleBuilder {
        let template = look.toggle_template(self.style);
        let mut builder = xgpui_luma::controls::toggle::new(self.id)
            .motion_defaults(look.motion_defaults().toggle)
            .with_motion_overrides(self.motion)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.selected)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .motion_enabled(self.motion_enabled)
            .with_callbacks(self.callbacks);
        if let Some(icon) = self.icon {
            builder = builder.icon(icon);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if self.round {
            builder = builder.round(true);
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<ToggleData>> for Toggle {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<ToggleData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button::HasPresenter;

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let toggle = Toggle::new("mode").ghost().size(ShadcnSize::Sm).with_data(true);
        assert!(matches!(toggle.style, ShadcnButtonStyle::Ghost));
        assert_eq!(toggle.size, ShadcnSize::Sm);
        assert!(toggle.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Toggle::new("t").selected(true).selected);
        assert!(Toggle::new("t").with_data(true).selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Toggle::new("ok").look(&look).primary().with_data(true).label("On").into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod callback_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    struct Owner {
        control: xgpui_luma::controls::toggle::Toggle,
        log: Vec<bool>,
    }
    #[test]
    fn look_builder_preserves_every_callback_and_payload() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let control = Toggle::new("choice")
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
                cx.emit(ToggleEvent::Change { selected: true });
                cx.emit(ToggleEvent::FocusChanged { focused: true });
                cx.emit(ToggleEvent::HoverChanged { hovered: true });
                cx.emit(ToggleEvent::EnabledChanged { enabled: true });
            })
        });
        app.read(|cx| assert_eq!(owner.read(cx).log, [false, true, false, true, false, true, false, true]));
    }
}
