//! Look-owned builder in the shape of `xgpui_luma_look_radix::TextField`: Radix
//! look, variant and size axes plus the SDK `TextFieldBuilder`'s options and
//! callbacks, so the control can later replace the SDK text field and text area
//! without changing call sites.
//!
//! The look resolves as Radix's does: an explicit [`look`](ParleyTextFieldBuilder::look),
//! else the application's ambient `Look` global (as a shell installs it), else
//! `Look::built_in()`.
use std::sync::Arc;

use gpui::{App, AppContext, Context, Div, Entity, Font, SharedString, Stateful, Window};
use xgpui_luma::{
    controls::textfield::{
        TextFieldLook, TextFieldLookOverride, TextFieldRenderModel, TextFieldState, TextFieldTemplate,
        TextFieldTemplateHandlers, TextFieldTheme, ThemedTextFieldTemplate,
    },
    infra::callbacks::EventCallbacks,
    theme::ControlSize,
};
use xgpui_luma_look_radix::{Look, TextFieldSize, TextFieldVariant, textfield_template, textfield_theme_with};

/// Adjusts the template's root element, as `TextFieldBuilder::with_template_modifier`.
pub type TemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

use super::{ParleyTextField, ParleyTextFieldEvent};

pub struct ParleyTextFieldBuilder {
    callbacks: EventCallbacks<ParleyTextFieldEvent>,
    pub(super) id: SharedString,
    look: Option<Look>,
    variant: TextFieldVariant,
    /// SDK-level theme that bypasses look resolution.
    theme: Option<Arc<dyn TextFieldTheme>>,
    pub(super) font: Font,
    pub(super) rows: Option<usize>,
    pub(super) value: String,
    pub(super) placeholder: SharedString,
    pub(super) size: TextFieldSize,
    pub(super) enabled: bool,
    pub(super) look_override: Option<TextFieldLookOverride>,
    modifiers: Vec<TemplateModifier>,
}

// Mirrors the SDK `TextFieldBuilder`; the lab page uses only part of it.
#[allow(dead_code)]
impl ParleyTextFieldBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: EventCallbacks::default(),
            id: id.into(),
            look: None,
            variant: TextFieldVariant::default(),
            theme: None,
            font: gpui::font(".SystemUIFont"),
            rows: None,
            value: String::new(),
            placeholder: SharedString::default(),
            size: TextFieldSize::default(),
            enabled: true,
            look_override: None,
            modifiers: Vec::new(),
        }
    }

    /// The Radix look to draw with, instead of the ambient or built-in one.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(TextFieldVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(TextFieldVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(TextFieldVariant::Soft)
    }

    /// Adjust the resolved look on every frame, as `TextFieldBuilder::look_override`.
    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static,
    {
        self.look_override = Some(Arc::new(look_override));
        self
    }

    /// Adjust the template's root element on every frame, in registration order.
    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn resolve_look(&self, cx: &App) -> Look {
        self.look.clone().or_else(|| cx.try_global::<Look>().cloned()).unwrap_or_else(Look::built_in)
    }

    /// The SDK theme this builder draws with: an explicit theme, else the
    /// variant of the explicit, ambient or built-in look.
    pub(super) fn resolve_theme(&self, cx: &App) -> Arc<dyn TextFieldTheme> {
        match &self.theme {
            Some(theme) => theme.clone(),
            None => textfield_theme_with(&self.resolve_look(cx), self.variant),
        }
    }

    /// The SDK template that draws the field's chrome, as the look's own text
    /// field uses it, with this builder's modifiers applied.
    pub(super) fn take_template(&mut self, cx: &App) -> Arc<dyn TextFieldTemplate> {
        let base: Arc<dyn TextFieldTemplate> = match &self.theme {
            Some(theme) => Arc::new(ThemedTextFieldTemplate::new(theme.clone())),
            None => textfield_template(&self.resolve_look(cx), self.variant),
        };
        if self.modifiers.is_empty() {
            return base;
        }
        Arc::new(ModifiedTemplate { base, modifiers: std::mem::take(&mut self.modifiers) })
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: TextFieldSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Make this a text area showing `rows` lines, as `TextAreaBuilder::rows`.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    /// Escape hatch: draw with an SDK text field theme directly, skipping the look.
    pub fn theme(mut self, theme: Arc<dyn TextFieldTheme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Font and fallbacks; size and weight come from the theme's typography.
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Append an owner-bound handler for every semantic event.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ParleyTextFieldEvent, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    pub fn on_value_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &String, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ParleyTextFieldEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    pub fn on_submit<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &String, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ParleyTextFieldEvent::Submit { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ParleyTextFieldEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ParleyTextFieldEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<ParleyTextField> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = cx.new(|cx| ParleyTextField::from_builder(self, cx));
        callbacks.bind(&control, cx);
        control
    }
}

/// A template with root modifiers, as the SDK's private `ModifiedTextFieldTemplate`.
struct ModifiedTemplate {
    base: Arc<dyn TextFieldTemplate>,
    modifiers: Vec<TemplateModifier>,
}

impl TextFieldTemplate for ModifiedTemplate {
    fn resolve_look(&self, state: TextFieldState, enabled: bool, size: ControlSize) -> TextFieldLook {
        self.base.resolve_look(state, enabled, size)
    }

    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale_factor: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        self.base.resolve_look_with_scale(state, enabled, size, scale_factor, cx)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.modifiers.iter().fold(root, |root, modifier| modifier(root, model))
    }
}
