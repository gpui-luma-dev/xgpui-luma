//! Look-owned text-area builder. Spawn synthesizes the SDK [`xgpui_luma::controls::textarea::TextArea`].

use std::sync::Arc;

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::textarea::{
    TextAreaBuilder, TextAreaLook, TextAreaRenderModel, ThemedTextAreaTemplate, Validator,
};
use xgpui_luma::infra::icon::IconSource;
use super::textfield::ShadcnTextFieldStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type TextAreaModifier = Box<dyn Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a text area: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct TextArea {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::textarea::TextAreaEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnTextFieldStyle,
    size: ShadcnSize,
    placeholder: SharedString,
    value: SharedString,
    enabled: bool,
    pointer_focus: xgpui_luma::interaction::PointerFocusPolicy,
    scroll_interaction: xgpui_luma::interaction::ScrollInteraction,
    full_width: bool,
    rows: usize,
    clean_on_escape: bool,
    select_all_on_tab_focus: bool,
    max_clipboard_paste_bytes: Option<usize>,
    resize_handle_icon: Option<IconSource>,
    validator: Option<Validator>,
    look_override: Option<Arc<dyn Fn(TextAreaLook) -> TextAreaLook + Send + Sync>>,
    modifiers: Vec<TextAreaModifier>,
}

impl TextArea {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::textarea::TextAreaEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `TextAreaEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &String, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textarea::TextAreaEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `TextAreaEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textarea::TextAreaEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TextAreaEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textarea::TextAreaEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            style: ShadcnTextFieldStyle::Primary,
            size: ShadcnSize::Md,
            placeholder: SharedString::default(),
            value: SharedString::default(),
            enabled: true,
            pointer_focus: Default::default(),
            scroll_interaction: xgpui_luma::interaction::ScrollInteraction::DOCUMENT,
            full_width: false,
            rows: 4,
            clean_on_escape: false,
            select_all_on_tab_focus: false,
            max_clipboard_paste_bytes: Some(xgpui_luma::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES),
            resize_handle_icon: None,
            validator: None,
            look_override: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn style(mut self, style: ShadcnTextFieldStyle) -> Self {
        self.style = style;
        self
    }

    pub fn primary(self) -> Self {
        self.style(ShadcnTextFieldStyle::Primary)
    }

    pub fn outline(self) -> Self {
        self.style(ShadcnTextFieldStyle::Outline)
    }

    pub fn surface(self) -> Self {
        self.style(ShadcnTextFieldStyle::Surface)
    }

    pub fn input(self) -> Self {
        self.style(ShadcnTextFieldStyle::Input)
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    /// Override the SDK interaction policy.
    /// Override pointer_focus independently of wheel routing.
    pub fn pointer_focus_policy(mut self, policy: xgpui_luma::interaction::PointerFocusPolicy) -> Self {
        self.pointer_focus = policy;
        self
    }

    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.scroll_interaction.wheel = policy;
        self
    }

    /// Override the SDK interaction policy.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.scroll_interaction.boundary = policy;
        self
    }

    /// Override the SDK interaction policy.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    pub fn clean_on_escape(mut self, clean: bool) -> Self {
        self.clean_on_escape = clean;
        self
    }

    pub fn select_all_on_tab_focus(mut self, select_all: bool) -> Self {
        self.select_all_on_tab_focus = select_all;
        self
    }

    /// Limit clipboard paste to `max_bytes` UTF-8 bytes (`None` keeps the full clipboard).
    pub fn max_clipboard_paste_bytes(mut self, max_bytes: Option<usize>) -> Self {
        self.max_clipboard_paste_bytes = max_bytes;
        self
    }

    pub fn resize_handle_icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.resize_handle_icon = Some(icon.into());
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextAreaLook) -> TextAreaLook + Send + Sync + 'static,
    {
        self.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::textarea::TextArea> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TextAreaBuilder {
        let (template, theme) = match self.style {
            ShadcnTextFieldStyle::Outline => (look.textarea_template(), look.textarea_theme()),
            ShadcnTextFieldStyle::Input => (look.input_textarea_template(), look.input_textarea_theme()),
            ShadcnTextFieldStyle::Primary => (look.primary_textarea_template(), look.primary_textarea_theme()),
            ShadcnTextFieldStyle::Surface => {
                let theme = look.surface_textarea_theme();
                let template: Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> =
                    Arc::new(ThemedTextAreaTemplate::new(theme.clone()));
                (template, theme)
            }
        };
        let mut builder = xgpui_luma::controls::textarea::TextArea::new(self.id)
            .stylesheet(&look.stylesheet().common)
            .template(template)
            .theme(theme)
            .placeholder(self.placeholder)
            .value(self.value)
            .enabled(self.enabled)
            .pointer_focus_policy(self.pointer_focus)
            .wheel_scroll_policy(self.scroll_interaction.wheel)
            .scroll_boundary_policy(self.scroll_interaction.boundary)
            .wheel_focus_scope(self.scroll_interaction.focus_scope)
            .full_width(self.full_width)
            .size(self.size.control_size())
            .rows(self.rows)
            .clean_on_escape(self.clean_on_escape)
            .select_all_on_tab_focus(self.select_all_on_tab_focus)
            .max_clipboard_paste_bytes(self.max_clipboard_paste_bytes);
        if let Some(icon) = self.resize_handle_icon {
            builder = builder.resize_handle_icon(icon);
        }
        if let Some(validator) = self.validator {
            builder = builder.validator(validator);
        }
        if let Some(look_override) = self.look_override {
            builder = builder.look_override(move |look| (look_override)(look));
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_keeps_shadcn_axes() {
        let area = TextArea::new("bio").surface().size(ShadcnSize::Lg).rows(6).placeholder("Bio").value("Hello");
        assert!(matches!(area.style, ShadcnTextFieldStyle::Surface));
        assert_eq!(area.size, ShadcnSize::Lg);
        assert_eq!(area.rows, 6);
        assert_eq!(area.placeholder.as_ref(), "Bio");
        assert_eq!(area.value.as_ref(), "Hello");
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = TextArea::new("ok")
            .look(&look)
            .primary()
            .placeholder("Notes")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn forwards_independent_wheel_policies_after_look_synthesis() {
    use xgpui_luma::interaction::{WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
    let mut app = gpui::TestAppContext::single();
    let control = TextArea::new("policy")
        .wheel_scroll_policy(WheelScrollPolicy::PassThrough)
        .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
        .wheel_focus_scope(WheelFocusScope::Owner)
        .into_sdk_builder(ShadcnLook::built_in())
        .spawn(&mut app);
    control.read_with(&app, |view, _| {
        let policy = view.scroll_interaction();
        assert_eq!(policy.wheel, WheelScrollPolicy::PassThrough);
        assert_eq!(policy.boundary, ScrollBoundaryPolicy::Chain);
        assert_eq!(policy.focus_scope, WheelFocusScope::Owner);
    });
}
