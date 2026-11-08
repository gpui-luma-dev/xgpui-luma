//! Look-owned button builder. Spawn synthesizes the SDK [`xgpui_luma::controls::button::Button`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::button::{
    ButtonCallbacks, ButtonEvent, ButtonBuilder, ButtonContentContext, ButtonRenderModel, ButtonTemplateModifier,
    ControlIcon, ControlPresenter, DefaultButtonTemplate, HasPresenter,
};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use super::button::{ButtonRadiusPreset, ShadcnButtonStyle, button_look_with_stylesheet};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a button: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Button<D = ()> {
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
    radius: Option<ButtonRadiusPreset>,
    icon: Option<ControlIcon>,
    role: ButtonFamilyRole,
    round: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    data: D,
    content: Option<ControlPresenter<ButtonContentContext<D>>>,
    modifiers: Vec<ButtonTemplateModifier<D>>,
    callbacks: ButtonCallbacks,
}

impl Button<()> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            style: ShadcnButtonStyle::Secondary,
            size: ShadcnSize::Md,
            radius: None,
            icon: None,
            role: ButtonFamilyRole::Text,
            round: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            data: (),
            content: None,
            modifiers: Vec::new(),
            callbacks: ButtonCallbacks::default(),
        }
    }

    /// Icon-only command button (square chrome).
    pub fn icon_button(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Self {
        Self::new(id).role(ButtonFamilyRole::Icon).round(true).icon(icon)
    }

    /// Pick a typed payload. Call once, usually at the start of the chain.
    ///
    /// Resets presenter content and queued template modifiers (they are typed on `D`).
    /// Owner-bound event callbacks are preserved.
    /// Shadcn axes and SDK settings (`icon`, `role`, `enabled`, `tab_stop`, `compact`) are kept.
    pub fn typed<D: Clone + 'static>(self, data: D) -> Button<D> {
        Button {
            id: self.id,
            look: self.look,
            style: self.style,
            size: self.size,
            radius: self.radius,
            icon: self.icon,
            role: self.role,
            round: self.round,
            enabled: self.enabled,
            tab_stop: self.tab_stop,
            compact: self.compact,
            without_elevation: self.without_elevation,
            data,
            content: None,
            modifiers: Vec::new(),
            callbacks: self.callbacks,
        }
    }
}

impl<D: Clone + 'static> Button<D> {
    /// Append an owner-bound callback for every Button event.
    ///
    /// Repeated registrations append in order. The owner is held weakly; the
    /// Button retains the callback until dropped. Events use the existing SDK
    /// event stream, including programmatic state changes that emit events.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ButtonEvent, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Append an owner-bound activation callback.
    ///
    /// Repeated registrations append in order. The owner is held weakly; the
    /// Button retains the callback until dropped. Events use the existing SDK
    /// event stream, including programmatic state changes that emit events.
    pub fn on_click<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_click(cx, callback);
        self
    }

    /// Append an owner-bound focus-state callback.
    ///
    /// Repeated registrations append in order. The owner is held weakly; the
    /// Button retains the callback until dropped. Events use the existing SDK
    /// event stream, including programmatic state changes that emit events.
    pub fn on_focus_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_focus_changed(cx, callback);
        self
    }

    /// Append an owner-bound hover-state callback.
    ///
    /// Repeated registrations append in order. The owner is held weakly; the
    /// Button retains the callback until dropped. Events use the existing SDK
    /// event stream, including programmatic state changes that emit events.
    pub fn on_hover_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_hover_changed(cx, callback);
        self
    }

    /// Append an owner-bound enabled-state callback.
    ///
    /// Repeated registrations append in order. The owner is held weakly; the
    /// Button retains the callback until dropped. Events use the existing SDK
    /// event stream, including programmatic state changes that emit events.
    pub fn on_enabled_changed<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_enabled_changed(cx, callback);
        self
    }

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

    pub fn radius(mut self, radius: ButtonRadiusPreset) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn with_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Semantic leading icon on a labeled button.
    pub fn icon(self, icon: impl Into<ControlIcon>) -> Self {
        self.with_icon(icon)
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.role = role;
        self
    }

    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
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

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::button::Button<D>> {
        let look = self.resolve_look(cx);
        let theme = crate::tooltip_theme(&look);
        let entity = self.into_sdk_builder(look).spawn(cx);
        entity.update(cx, |control, _| {
            xgpui_luma::infra::attachments::AttachmentTarget::attachments_mut(control).set_tooltip_theme(theme);
        });
        entity
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ButtonBuilder<D> {
        let style = self.style;
        let radius = self.radius;
        let template = std::sync::Arc::new(DefaultButtonTemplate::new(super::templates::styled_button_family_theme(
            look.clone(),
            style,
        )));
        let mut builder = xgpui_luma::controls::button::Button::new(self.id)
            .typed(self.data)
            .with_callbacks(self.callbacks)
            .template(template)
            .size(self.size.control_size())
            .role(self.role)
            .round(self.round)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop);
        if let Some(radius) = radius {
            let geometry_look = look.clone();
            builder = builder.with_look(move |model| {
                button_look_with_stylesheet(
                    geometry_look.mode_tokens().as_ref(),
                    geometry_look.stylesheet().as_ref(),
                    geometry_look.mode(),
                    style,
                    model.role,
                    model.size,
                    Some(radius),
                    model.state,
                )
            });
        }
        if let Some(icon) = self.icon {
            builder = builder.icon(icon);
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
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

impl<D: 'static> HasPresenter<ButtonContentContext<D>> for Button<D> {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<D>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button::HasPresenter;

    #[test]
    fn default_style_is_secondary() {
        assert!(matches!(Button::new("ok").style, ShadcnButtonStyle::Secondary));
    }

    #[test]
    fn style_helpers_set_axes() {
        assert!(matches!(Button::new("ok").primary().style, ShadcnButtonStyle::Primary));
        assert!(matches!(Button::new("ok").outline().style, ShadcnButtonStyle::Outline));
        assert!(matches!(Button::new("ok").ghost().style, ShadcnButtonStyle::Ghost));
        let content_only = Button::new("ok").content_only();
        assert!(matches!(content_only.style, ShadcnButtonStyle::ContentOnly));
        assert!(content_only.without_elevation);
    }

    #[test]
    fn typed_keeps_shadcn_axes_and_resets_d_typed_seams() {
        let button = Button::new("counter")
            .primary()
            .size(ShadcnSize::Lg)
            .label("ignored")
            .with_template_modifier(|root, _| root)
            .typed(7u32);
        assert!(matches!(button.style, ShadcnButtonStyle::Primary));
        assert_eq!(button.size, ShadcnSize::Lg);
        assert_eq!(button.data, 7);
        assert!(button.content.is_none());
        assert!(button.modifiers.is_empty());
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Button::new("ok")
            .look(&look)
            .primary()
            .label("Submit")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod callback_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    struct Owner {
        button: Entity<xgpui_luma::controls::button::Button<u32>>,
        log: Vec<String>,
    }

    #[test]
    fn look_forwards_all_callbacks_and_preserves_them_through_typed() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let button = Button::new("callbacks")
                    .on_event(cx, |owner: &mut Owner, event, _| {
                        if event.is_click() {
                            owner.log.push("all".into());
                        }
                    })
                    .on_click(cx, |owner, _| owner.log.push("click".into()))
                    .typed(42_u32)
                    .on_focus_changed(cx, |owner, value, _| owner.log.push(format!("focus:{value}")))
                    .on_hover_changed(cx, |owner, value, _| owner.log.push(format!("hover:{value}")))
                    .on_enabled_changed(cx, |owner, value, _| owner.log.push(format!("enabled:{value}")))
                    .spawn(cx);
                Owner { button, log: Vec::new() }
            })
        });
        let button = app.read(|cx| owner.read(cx).button.clone());
        app.update(|cx| {
            button.update(cx, |_, cx| {
                cx.emit(ButtonEvent::Click);
                cx.emit(ButtonEvent::FocusChanged { focused: true });
                cx.emit(ButtonEvent::HoverChanged { hovered: true });
            })
        });
        app.update(|cx| button.update(cx, |button, cx| button.set_enabled(false, cx)));
        app.read(|cx| assert_eq!(owner.read(cx).log, ["all", "click", "focus:true", "hover:true", "enabled:false"]));
    }
}
