#![allow(deprecated)] // The look-closure bridge reads `TextFieldLook` until it is removed (#433).
use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::template::template_with_modifier;
use super::{TextFieldLook, TextFieldState, TextFieldTemplate, default_textfield_template};
use super::control::TextFieldControl;
use crate::controls::button::ControlIcon;

pub type Validator = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type TextFieldLookOverride = Arc<dyn Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct TextFieldModel {
    pub(crate) id: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) value: SharedString,
    pub(crate) prefix_icon: Option<ControlIcon>,
    pub(crate) enabled: bool,
    pub(crate) tab_stop: bool,
    pub(crate) compact: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) select_all_on_tab_focus: bool,
    pub(crate) propagate_home_end_to_parent: bool,
    pub(crate) max_clipboard_paste_bytes: Option<usize>,
    pub(crate) validator: Option<Validator>,
    /// Style overrides set on this field, over its template's style (#433).
    pub(crate) overrides: crate::style::lss::Overrides,
    pub(crate) look_override: Option<TextFieldLookOverride>,
    pub(crate) template: Arc<dyn TextFieldTemplate>,
}

pub struct TextFieldRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub prefix_icon: Option<&'a ControlIcon>,
    pub enabled: bool,
    pub full_width: bool,
    pub state: TextFieldState,
    /// Active IME composition range, measured in Unicode characters.
    pub marked_range: Option<std::ops::Range<usize>>,
    pub caret_visible: bool,
    pub horizontal_scroll: f32,
    pub character_offsets: Vec<f32>,
    pub style: std::sync::Arc<xgpui_luma_style::Resolved>,
}

pub struct TextFieldBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::textfield::TextFieldEvent>,

    pub(crate) model: TextFieldModel,
}

impl TextFieldBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::textfield::TextFieldEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::textfield::TextFieldEvent::Change { value } = event {
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
            if let crate::controls::textfield::TextFieldEvent::Submit { value } = event {
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
            if let crate::controls::textfield::TextFieldEvent::FocusChanged { focused } = event {
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
            if let crate::controls::textfield::TextFieldEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: TextFieldModel {
                id: id.into(),
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
                max_clipboard_paste_bytes: Some(crate::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES),
                validator: None,
                overrides: crate::style::lss::Overrides::new(),
                look_override: None,
                template: default_textfield_template(),
            },
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.model.value = value.into();
        self
    }

    pub fn prefix_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.model.prefix_icon = Some(icon.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    /// Compact layout: applies tighter vertical padding.
    pub fn compact(mut self) -> Self {
        self.model.compact = true;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean: bool) -> Self {
        self.model.clean_on_escape = clean;
        self
    }

    pub fn select_all_on_tab_focus(mut self, select_all: bool) -> Self {
        self.model.select_all_on_tab_focus = select_all;
        self
    }

    pub fn propagate_home_end_to_parent(mut self, propagate: bool) -> Self {
        self.model.propagate_home_end_to_parent = propagate;
        self
    }

    /// Limit clipboard paste to `max_bytes` UTF-8 bytes (`None` keeps the full clipboard).
    ///
    /// Default is [`crate::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES`]. This is a desktop
    /// memory budget; raise or disable it for large-document editors.
    pub fn max_clipboard_paste_bytes(mut self, max_bytes: Option<usize>) -> Self {
        self.model.max_clipboard_paste_bytes = max_bytes;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.model.validator = Some(validator);
        self
    }

    /// Style overrides for this field (#433): values set over its style, such as `padding_x` or `font_size`.
    pub fn overrides(mut self, overrides: crate::style::lss::Overrides) -> Self {
        self.model.overrides = overrides;
        self
    }

    #[deprecated(note = "set style overrides instead; the closure is bridged to them (#433)")]
    pub fn look_override<F>(mut self, look_override: F) -> Self
    where
        F: Fn(TextFieldLook) -> TextFieldLook + Send + Sync + 'static,
    {
        self.model.look_override = Some(Arc::new(look_override));
        self
    }

    pub fn template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TextFieldRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<TextFieldControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| TextFieldControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::textfield::{
        TextFieldPalette, TextFieldState, TextFieldTheme, ThemedTextFieldTemplate, default_textfield_theme,
    };
    use crate::theme::{MetricTokens, ThemeTokens};
    use gpui::hsla;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_textfield_template();
        let builder = TextFieldBuilder::new("textfield-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_template_modifier_forwards_resolve_look() {
        struct MarkerTheme;

        impl TextFieldTheme for MarkerTheme {
            fn resolve(&self, _state: TextFieldState, _enabled: bool) -> TextFieldPalette {
                let mut palette = default_textfield_theme().resolve(TextFieldState::default(), true);
                palette.background = hsla(0.12, 1.0, 0.55, 1.0);
                palette
            }

            fn metrics(&self) -> MetricTokens {
                ThemeTokens::default().metrics
            }
        }

        let base = Arc::new(ThemedTextFieldTemplate::new(Arc::new(MarkerTheme)));
        let expected = base.resolve_look(TextFieldState::default(), true);
        let modified = TextFieldBuilder::new("textfield-test")
            .template(base)
            .with_template_modifier(|element, _| element)
            .model
            .template;
        let look = modified.resolve_look(TextFieldState::default(), true);

        assert_eq!(look.background, expected.background);
        assert_eq!(look.background, hsla(0.12, 1.0, 0.55, 1.0));
    }
}
