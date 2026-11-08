//! Look-owned button builder. Spawn synthesizes the SDK [`xgpui_luma::controls::button::Button`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::button::{
    ButtonCallbacks, ButtonEvent, ButtonBuilder, ButtonContentContext, ButtonRenderModel, ButtonTemplateModifier,
    ControlIcon, ControlPresenter, HasPresenter,
};
use xgpui_luma::controls::button_family::ButtonFamilyRole;

use crate::button::{Paint, ButtonVariant, button_look_for, button_template};
use crate::button_layout::{ButtonSize, Radius};
use crate::look::{Look, resolve_look};
use crate::tone::Tone;

/// Builder in the guise of a button: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Button<D = ()> {
    id: SharedString,
    look: Option<Look>,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
    radius: Radius,
    icon: Option<ControlIcon>,
    icon_size: Option<f32>,
    role: ButtonFamilyRole,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
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
            variant: ButtonVariant::default(),
            paint: Paint::accent(),
            size: ButtonSize::default(),
            radius: Radius::default(),
            icon: None,
            icon_size: None,
            role: ButtonFamilyRole::Text,
            enabled: true,
            tab_stop: true,
            compact: false,
            data: (),
            content: None,
            modifiers: Vec::new(),
            callbacks: ButtonCallbacks::default(),
        }
    }

    /// Pick a typed payload. Call once, usually at the start of the chain.
    ///
    /// Resets presenter content and queued template modifiers (they are typed on `D`).
    /// Owner-bound event callbacks are preserved.
    /// Radix axes and SDK settings (`icon`, `role`, `enabled`, `tab_stop`, `compact`) are kept.
    pub fn typed<D: Clone + 'static>(self, data: D) -> Button<D> {
        Button {
            id: self.id,
            look: self.look,
            variant: self.variant,
            paint: self.paint,
            size: self.size,
            radius: self.radius,
            icon: self.icon,
            icon_size: self.icon_size,
            role: self.role,
            enabled: self.enabled,
            tab_stop: self.tab_stop,
            compact: self.compact,
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
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(ButtonVariant::Classic)
    }

    pub fn solid(self) -> Self {
        self.variant(ButtonVariant::Solid)
    }

    pub fn soft(self) -> Self {
        self.variant(ButtonVariant::Soft)
    }

    pub fn surface(self) -> Self {
        self.variant(ButtonVariant::Surface)
    }

    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }

    /// Content-only action: no background, border, shadow, or focus-ring chrome.
    pub fn content_only(self) -> Self {
        self.ghost_quiet()
    }

    pub fn ghost_quiet(self) -> Self {
        self.variant(ButtonVariant::GhostQuiet)
    }

    pub fn page(self) -> Self {
        self.variant(ButtonVariant::Page)
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

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Override the icon and its SDK layout slot in logical pixels.
    /// The button's outer dimensions still follow its size.
    pub fn icon_size(mut self, size: f32) -> Self {
        if size.is_finite() && size >= 0.0 {
            self.icon_size = Some(size);
        }
        self
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.role = role;
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

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::button::Button<D>> {
        let look = self.resolve_look(cx);
        let theme = crate::tooltip::tooltip_theme(&look);
        let entity = self.into_sdk_builder(look).spawn(cx);
        entity.update(cx, |control, _| {
            xgpui_luma::infra::attachments::AttachmentTarget::attachments_mut(control).set_tooltip_theme(theme);
        });
        entity
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn look_source(&self, look: &Look) -> xgpui_luma::controls::button::ButtonLookSource<D> {
        let geometry = button_look_for(look, self.variant, self.paint, self.size, self.radius);
        let icon_size = self.icon_size;
        std::sync::Arc::new(move |model| {
            let mut resolved = geometry(model);
            if let Some(size) = icon_size {
                resolved.icon_size = size;
            }
            resolved
        })
    }

    fn into_sdk_builder(self, look: Look) -> ButtonBuilder<D> {
        let template = button_template(&look, self.variant, self.paint);
        let geometry = self.look_source(&look);
        let mut builder = xgpui_luma::controls::button::Button::new(self.id)
            .typed(self.data)
            .with_callbacks(self.callbacks)
            .template(template)
            .with_look(move |model| geometry(model))
            .size(self.size.control_size())
            .role(self.role)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop);
        if let Some(icon) = self.icon {
            builder = builder.icon(icon);
        }
        if self.compact {
            builder = builder.compact();
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
    use crate::button_layout::button_box_for;
    use crate::look::resolve_look;
    use xgpui_luma::controls::button::HasPresenter;
    use xgpui_luma::theme::ControlSize;

    fn primary(look: &Look) -> gpui::Hsla {
        look.resolve_role(crate::semantic::SemanticRole::Primary).hsla()
    }

    #[test]
    fn icon_size_override_updates_the_sdk_slot_without_resizing_the_button() {
        let look = Look::built_in();
        let button = Button::new("toolbar-icon")
            .ghost_quiet()
            .gray()
            .size(ButtonSize::One)
            .role(ButtonFamilyRole::Icon)
            .icon_size(20.0)
            .typed(());
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for focused in [false, true] {
                let model = ButtonRenderModel::<()> {
                    role: ButtonFamilyRole::Icon,
                    state: xgpui_luma::theme::InteractionState { focused, ..Default::default() },
                    ..Default::default()
                };
                let resolved = button.look_source(&look)(&model);
                assert_eq!(resolved.icon_size, 20.0);
                assert_eq!(resolved.height, 24.0);
                assert!(resolved.border.is_none());
            }
        }
    }

    #[test]
    fn missing_look_and_global_falls_back_to_built_in() {
        let resolved = resolve_look(None, None);
        assert_eq!(resolved.palettes(), Look::built_in().palettes());
        assert_eq!(primary(&resolved), primary(&Look::built_in()));
    }

    #[test]
    fn explicit_look_wins_over_ambient() {
        let ambient = Look::built_in();
        let fork = ambient.fork();
        fork.set_accent_seed(xgpui_luma::color::gpui_bridge::from_hsla(gpui::hsla(0.05, 0.9, 0.5, 1.0)))
            .unwrap();
        let resolved = resolve_look(Some(&fork), Some(&ambient));
        assert_eq!(primary(&resolved), primary(&fork));
        assert_ne!(primary(&resolved), primary(&ambient));
    }

    #[test]
    fn ambient_is_used_when_look_is_omitted() {
        let ambient = Look::built_in().fork();
        ambient
            .set_accent_seed(xgpui_luma::color::gpui_bridge::from_hsla(gpui::hsla(0.12, 0.8, 0.4, 1.0)))
            .unwrap();
        let resolved = resolve_look(None, Some(&ambient));
        assert_eq!(primary(&resolved), primary(&ambient));
        assert_ne!(primary(&resolved), primary(&Look::built_in()));
    }

    #[test]
    fn classic_bind_selects_classic_template() {
        let button = Button::new("classic").classic();
        assert!(matches!(button.variant, ButtonVariant::Classic));
    }

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Button::new("s").size(ButtonSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Button::new("s").size(ButtonSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Button::new("s").size(ButtonSize::Three).size.control_size(), ControlSize::Lg);
        assert_eq!(Button::new("s").size(ButtonSize::Four).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn bind_geometry_uses_size_and_radius() {
        let look = Look::built_in();
        let button = Button::new("sized").size(ButtonSize::Four).radius(Radius::None);
        let geometry = button_look_for::<()>(&look, button.variant, button.paint, button.size, button.radius);
        let model = ButtonRenderModel::<()> { size: button.size.control_size(), ..Default::default() };
        let family = geometry(&model);
        assert_eq!(family.radius, button_box_for(ButtonSize::Four, Radius::None).radius);
    }

    #[test]
    fn modifiers_queue_in_call_order() {
        let button = Button::new("mod").with_template_modifier(|root, _| root).with_template_modifier(|root, _| root);
        assert_eq!(button.modifiers.len(), 2);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Button::new("ok")
            .look(&look)
            .solid()
            .high_contrast(true)
            .gray()
            .label("Submit")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }

    #[test]
    fn typed_keeps_radix_axes_and_resets_d_typed_seams() {
        let button = Button::new("counter")
            .solid()
            .gray()
            .size(ButtonSize::Three)
            .label("ignored")
            .with_template_modifier(|root, _| root)
            .typed(7u32);
        assert!(matches!(button.variant, ButtonVariant::Solid));
        assert_eq!(button.paint.tone, Tone::Gray);
        assert_eq!(button.size, ButtonSize::Three);
        assert_eq!(button.data, 7);
        assert!(button.content.is_none());
        assert!(button.modifiers.is_empty());
    }

    #[test]
    fn typed_bind_does_not_panic() {
        let look = Look::built_in();
        let _builder = Button::new("counter").look(&look).classic().typed(true).label("On").into_sdk_builder(look);
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
