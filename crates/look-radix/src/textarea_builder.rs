//! Look-owned text-area builder. Spawn synthesizes the SDK [`xgpui_luma::controls::textarea::TextArea`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::textarea::{TextAreaBuilder, TextAreaRenderModel, Validator};
use xgpui_luma::infra::icon::IconSource;

use crate::look::{Look, resolve_look};
use crate::textarea::{TextAreaSize, textarea_template};
use crate::textfield::TextFieldVariant;

type TextAreaModifier = Box<dyn Fn(Stateful<Div>, &TextAreaRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a text area: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct TextArea {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::textarea::TextAreaEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: TextFieldVariant,
    size: TextAreaSize,
    placeholder: SharedString,
    value: SharedString,
    enabled: bool,
    full_width: bool,
    rows: usize,
    clean_on_escape: bool,
    select_all_on_tab_focus: bool,
    max_clipboard_paste_bytes: Option<usize>,
    resize_handle_icon: Option<IconSource>,
    validator: Option<Validator>,
    overrides: xgpui_luma::style::lss::Overrides,
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
            variant: TextFieldVariant::default(),
            size: TextAreaSize::default(),
            placeholder: SharedString::default(),
            value: SharedString::default(),
            enabled: true,
            full_width: false,
            rows: 4,
            clean_on_escape: false,
            select_all_on_tab_focus: false,
            max_clipboard_paste_bytes: Some(xgpui_luma::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES),
            resize_handle_icon: None,
            validator: None,
            overrides: xgpui_luma::style::lss::Overrides::new(),
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
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

    pub fn size(mut self, size: TextAreaSize) -> Self {
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

    /// Style overrides for this control (#433): values set over its style, such as `padding_x` or `font_size`.
    pub fn overrides(mut self, overrides: xgpui_luma::style::lss::Overrides) -> Self {
        self.overrides = overrides;
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

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> TextAreaBuilder {
        let look = look.with_size(self.size.control_size());
        let template = textarea_template(&look, self.variant);
        let mut builder = xgpui_luma::controls::textarea::TextArea::new(self.id)
            .stylesheet(&look.stylesheet_snapshot())
            .template(template)
            .placeholder(self.placeholder)
            .value(self.value)
            .enabled(self.enabled)
            .full_width(self.full_width)
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
        builder = builder.overrides(self.overrides);
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RadixSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(TextArea::new("s").size(TextAreaSize::One).size.control_size(), RadixSize::One);
        assert_eq!(TextArea::new("s").size(TextAreaSize::Two).size.control_size(), RadixSize::Two);
        assert_eq!(TextArea::new("s").size(TextAreaSize::Three).size.control_size(), RadixSize::Three);
    }

    #[test]
    fn value_keeps_radix_axes() {
        let area = TextArea::new("bio").soft().size(TextAreaSize::Three).rows(6).placeholder("Bio").value("Hello");
        assert!(matches!(area.variant, TextFieldVariant::Soft));
        assert_eq!(area.size, TextAreaSize::Three);
        assert_eq!(area.rows, 6);
        assert_eq!(area.placeholder.as_ref(), "Bio");
        assert_eq!(area.value.as_ref(), "Hello");
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = TextArea::new("ok")
            .look(&look)
            .classic()
            .placeholder("Notes")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
