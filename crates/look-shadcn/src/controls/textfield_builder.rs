//! Look-owned text-field builder. Spawn synthesizes the SDK [`xgpui_luma::controls::textfield::TextField`].

use std::sync::Arc;

use gpui::{App, Context, Div, SharedString, Stateful};
use xgpui_luma::controls::button::ControlIcon;
use xgpui_luma::controls::textfield::{
    TextFieldBuilder, TextFieldLook, TextFieldLookOverride, TextFieldRenderModel, ThemedTextFieldTemplate, Validator,
};
use super::textfield::ShadcnTextFieldStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type TextFieldModifier = Box<dyn Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a text field: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct TextField {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::textfield::TextFieldEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnTextFieldStyle,
    size: ShadcnSize,
    placeholder: SharedString,
    value: SharedString,
    prefix_icon: Option<ControlIcon>,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    full_width: bool,
    clean_on_escape: bool,
    select_all_on_tab_focus: bool,
    propagate_home_end_to_parent: bool,
    max_clipboard_paste_bytes: Option<usize>,
    validator: Option<Validator>,
    look_override: Option<TextFieldLookOverride>,
    modifiers: Vec<TextFieldModifier>,
}

impl TextField {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::textfield::TextFieldEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `TextFieldEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &String, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textfield::TextFieldEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `TextFieldEvent::Submit` through the existing event stream.
    pub fn on_submit<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &String, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textfield::TextFieldEvent::Submit { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `TextFieldEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textfield::TextFieldEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TextFieldEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::textfield::TextFieldEvent::EnabledChanged { enabled } = event {
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
            prefix_icon: None,
            enabled: true,
            tab_stop: true,
            compact: false,
            full_width: false,
            clean_on_escape: false,
            select_all_on_tab_focus: false,
            propagate_home_end_to_parent: false,
            max_clipboard_paste_bytes: Some(xgpui_luma::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES),
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

    pub fn prefix_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.prefix_icon = Some(icon.into());
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

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
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

    pub fn propagate_home_end_to_parent(mut self, propagate: bool) -> Self {
        self.propagate_home_end_to_parent = propagate;
        self
    }

    /// Limit clipboard paste to `max_bytes` UTF-8 bytes (`None` keeps the full clipboard).
    pub fn max_clipboard_paste_bytes(mut self, max_bytes: Option<usize>) -> Self {
        self.max_clipboard_paste_bytes = max_bytes;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static,
    {
        self.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::textfield::TextField {
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

    fn into_sdk_builder(self, look: ShadcnLook) -> TextFieldBuilder {
        let template = match self.style {
            ShadcnTextFieldStyle::Outline => look.textfield_template(),
            ShadcnTextFieldStyle::Input => look.input_textfield_template(),
            ShadcnTextFieldStyle::Primary => look.primary_textfield_template(),
            ShadcnTextFieldStyle::Surface => Arc::new(ThemedTextFieldTemplate::new(look.surface_textfield_theme())),
        };
        let mut builder = xgpui_luma::controls::textfield::new(self.id)
            .template(template)
            .placeholder(self.placeholder)
            .value(self.value)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .size(self.size.control_size())
            .full_width(self.full_width)
            .clean_on_escape(self.clean_on_escape)
            .select_all_on_tab_focus(self.select_all_on_tab_focus)
            .propagate_home_end_to_parent(self.propagate_home_end_to_parent)
            .max_clipboard_paste_bytes(self.max_clipboard_paste_bytes);
        if let Some(icon) = self.prefix_icon {
            builder = builder.prefix_icon(icon);
        }
        if self.compact {
            builder = builder.compact();
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
        let field = TextField::new("name").surface().size(ShadcnSize::Lg).placeholder("Name").value("Ada");
        assert!(matches!(field.style, ShadcnTextFieldStyle::Surface));
        assert_eq!(field.size, ShadcnSize::Lg);
        assert_eq!(field.placeholder.as_ref(), "Name");
        assert_eq!(field.value.as_ref(), "Ada");
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = TextField::new("ok")
            .look(&look)
            .primary()
            .placeholder("Search…")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
