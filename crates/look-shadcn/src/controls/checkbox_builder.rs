//! Look-owned checkbox builder. Spawn synthesizes the SDK [`xgpui_luma::controls::checkbox::Checkbox`].

use xgpui_luma::controls::checkbox::{CheckboxCallbacks, CheckboxEvent};
use gpui::{App, Context, SharedString};
use xgpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use xgpui_luma::controls::checkbox::{CheckboxBuilder, CheckboxData};
use xgpui_luma::infra::icon::SelectionStatusIcons;
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a checkbox: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Checkbox {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::checkbox::CheckboxMotion>,
    callbacks: CheckboxCallbacks,
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
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
            style: ShadcnButtonStyle::Primary,
            size: ShadcnSize::Md,
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
        use xgpui_luma::infra::attachments::TooltipEntityExt;
        let theme = crate::tooltip_theme(&look);
        self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> CheckboxBuilder {
        let template = look.checkbox_template(self.style);
        let mut builder = xgpui_luma::controls::checkbox::new(self.id)
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
        builder = builder.icons(SelectionStatusIcons::from_stylesheet(&look.stylesheet().common));
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

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let box_ = Checkbox::new("task").outline().size(ShadcnSize::Lg).with_data(true);
        assert!(matches!(box_.style, ShadcnButtonStyle::Outline));
        assert_eq!(box_.size, ShadcnSize::Lg);
        assert!(box_.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Checkbox::new("task").checked(true).checked);
        assert!(Checkbox::new("task").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Checkbox::new("ok").look(&look).primary().with_data(true).label("Done").into_sdk_builder(look);
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
