use crate::interaction::{ScrollInteraction, WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::template::template_with_modifier;
use super::{TextArea, TextAreaState, TextAreaTemplate, default_textarea_template};
use crate::controls::textarea::{TextAreaTheme, default_textarea_theme};
use crate::infra::icon::IconSource;

pub type Validator = Arc<dyn Fn(&str) -> bool + Send + Sync>;
#[derive(Clone)]
pub struct TextAreaModel {
    pub(crate) id: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) value: SharedString,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: ScrollInteraction,
    pub(crate) pointer_focus: crate::interaction::PointerFocusPolicy,
    pub(crate) full_width: bool,
    pub(crate) rows: usize,
    pub(crate) clean_on_escape: bool,
    pub(crate) select_all_on_tab_focus: bool,
    pub(crate) max_clipboard_paste_bytes: Option<usize>,
    pub(crate) resize_handle_icon_explicit: bool,
    pub(crate) resize_handle_icon: IconSource,
    pub(crate) validator: Option<Validator>,
    /// Style overrides set on this area, over its theme's style (#433).
    pub(crate) overrides: crate::style::lss::Overrides,
    pub(crate) template: Arc<dyn TextAreaTemplate>,
    pub(crate) theme: Arc<dyn TextAreaTheme>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextAreaLineMetric {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub y: f32,
    pub height: f32,
    pub character_offsets: Vec<f32>,
}

pub struct TextAreaRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub enabled: bool,
    pub full_width: bool,
    pub rows: usize,
    pub state: TextAreaState,
    pub caret_visible: bool,
    pub vertical_scroll: f32,
    pub line_metrics: Vec<TextAreaLineMetric>,
}

pub struct TextAreaBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::textarea::TextAreaEvent>,

    pub(crate) model: TextAreaModel,
}

impl TextAreaBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::textarea::TextAreaEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::textarea::TextAreaEvent::Change { value } = event {
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
            if let crate::controls::textarea::TextAreaEvent::FocusChanged { focused } = event {
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
            if let crate::controls::textarea::TextAreaEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: TextAreaModel {
                id: id.into(),
                placeholder: SharedString::default(),
                value: SharedString::default(),
                enabled: true,
                scroll_interaction: ScrollInteraction::DOCUMENT,
                pointer_focus: Default::default(),
                full_width: false,
                rows: 4,
                clean_on_escape: false,
                select_all_on_tab_focus: false,
                max_clipboard_paste_bytes: Some(crate::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES),
                resize_handle_icon_explicit: false,
                resize_handle_icon: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .icons
                    .resolve(crate::theme::stylesheet::IconRole::TextareaResize),
                validator: None,
                overrides: crate::style::lss::Overrides::new(),
                template: default_textarea_template(),
                theme: default_textarea_theme(),
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

    /// Configure pointer_focus independently of wheel and keyboard ownership.
    pub fn pointer_focus_policy(mut self, policy: crate::interaction::PointerFocusPolicy) -> Self {
        self.model.pointer_focus = policy;
        self
    }

    /// Override wheel without changing other interaction settings.
    pub fn wheel_scroll_policy(mut self, policy: WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Override boundary without changing other interaction settings.
    pub fn scroll_boundary_policy(mut self, policy: ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Override focus_scope without changing other interaction settings.
    pub fn wheel_focus_scope(mut self, policy: WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.model.rows = rows.max(1);
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

    /// Limit clipboard paste to `max_bytes` UTF-8 bytes (`None` keeps the full clipboard).
    ///
    /// Default is [`crate::controls::text::DEFAULT_MAX_CLIPBOARD_PASTE_BYTES`]. This is a desktop
    /// memory budget; raise or disable it for large-document editors.
    pub fn max_clipboard_paste_bytes(mut self, max_bytes: Option<usize>) -> Self {
        self.model.max_clipboard_paste_bytes = max_bytes;
        self
    }

    /// Set the icon displayed by the draggable resize handle.
    /// Capture shared resize glyphs while preserving explicit consumer glyphs.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.resize_handle_icon_explicit {
            self.model.resize_handle_icon =
                stylesheet.icons.resolve(crate::theme::stylesheet::IconRole::TextareaResize);
        }
        self
    }

    pub fn resize_handle_icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.model.resize_handle_icon_explicit = true;
        self.model.resize_handle_icon = icon.into();
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.model.validator = Some(validator);
        self
    }

    /// Style overrides for this area (#433): values set over its style, such as `padding_x` or `font_size`.
    pub fn overrides(mut self, overrides: crate::style::lss::Overrides) -> Self {
        self.model.overrides = overrides;
        self
    }

    pub fn template(mut self, template: Arc<dyn TextAreaTemplate>) -> Self {
        if let Some(theme) = template.theme() {
            self.model.theme = theme;
        }
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TextAreaRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn TextAreaTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<TextArea> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| TextArea::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_textarea_template();
        let builder = TextAreaBuilder::new("textarea-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
