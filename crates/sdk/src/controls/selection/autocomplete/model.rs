crate::motion_profile!(AutocompleteMotion => autocomplete {
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::behavior::SelectionItem;
use super::control::AutocompleteControl;
use super::theme::{AutocompleteTheme, default_autocomplete_theme};
use super::template::{
    AutocompleteItemsTemplate, AutocompleteTemplate, default_autocomplete_items_template,
    default_autocomplete_template, modified_autocomplete_items_template, modified_autocomplete_template,
};
use crate::controls::selector_list::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::{ControlSize, ThemeTokens};

pub type AutocompletePopupLookProvider = Arc<dyn Fn(ControlSize) -> SelectorItemsPanelLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AutocompleteModel {
    pub(crate) row_limits: (usize, usize),
    pub(crate) icons: crate::theme::stylesheet::IconsStylesheet,
    pub(crate) motion: crate::motion::MotionConfig<AutocompleteMotion>,
    pub(crate) stylesheet: crate::theme::stylesheet::AutocompleteStylesheet,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) placeholder_explicit: bool,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: crate::interaction::ScrollInteraction,
    pub(crate) invalid: bool,
    pub(crate) size: ControlSize,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn AutocompleteTemplate>,
    pub(crate) items_template: Arc<dyn AutocompleteItemsTemplate>,
    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTheme>,
    pub(crate) popup_look_provider: AutocompletePopupLookProvider,
}

pub struct AutocompleteBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::autocomplete::AutocompleteEvent>,

    pub(crate) model: AutocompleteModel,
}

impl AutocompleteBuilder {
    crate::motion_methods!(AutocompleteMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::autocomplete::AutocompleteEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `AutocompleteEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::Change { query } = event {
                callback(owner, query, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::Complete` through the existing event stream.
    pub fn on_complete<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::Complete { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::Clear` through the existing event stream.
    pub fn on_clear<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::Clear = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `AutocompleteEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::autocomplete::AutocompleteEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            callbacks: Default::default(),

            model: AutocompleteModel {
                motion: Default::default(),
                row_limits: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .row_limits
                    .resolve("autocomplete"),
                icons: crate::theme::stylesheet::CommonStylesheet::sdk_default().icons.clone(),
                stylesheet: crate::theme::stylesheet::CommonStylesheet::sdk_default().autocomplete.clone(),
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: crate::infra::locale::SdkStrings::default().search_placeholder,
                placeholder_explicit: false,
                enabled: true,
                scroll_interaction: crate::interaction::ScrollInteraction::VIEWPORT,
                invalid: false,
                size: ControlSize::Md,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                textfield_template: default_textfield_template(),
                scrollbar_template: default_scrollbar_template(),
                template: default_autocomplete_template(),
                items_template: default_autocomplete_items_template(),
                autocomplete_theme: default_autocomplete_theme(),
                popup_look_provider: Arc::new(|size| default_selector_items_panel_look(&ThemeTokens::default(), size)),
            },
        }
    }

    /// Capture shared affordance geometry; per-size values resolve at render time.
    /// Text-field/popup providers and consumer template modifiers retain their own precedence.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        self.model.row_limits = stylesheet.row_limits.resolve("autocomplete");
        self.model.icons = stylesheet.with_sdk_defaults().icons;
        self.model.stylesheet = stylesheet.with_sdk_defaults().autocomplete;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder_explicit = true;
        self.model.placeholder = placeholder.into();
        self
    }

    /// Configure popup wheel independently; wheel input never focuses a row.
    pub fn wheel_scroll_policy(mut self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Configure popup boundary independently; wheel input never focuses a row.
    pub fn scroll_boundary_policy(mut self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Configure popup focus_scope independently; wheel input never focuses a row.
    pub fn wheel_focus_scope(mut self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.model.clean_on_escape = clean_on_escape;
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.model.scrolling = scrolling;
        self
    }

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn AutocompleteTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &mut gpui::App) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_autocomplete_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn items_template(mut self, template: Arc<dyn AutocompleteItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn with_items_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::Stateful<gpui::Div>,
                &super::template::AutocompleteItemsRenderModel<'a>,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.items_template =
            modified_autocomplete_items_template(Arc::clone(&self.model.items_template), modifier);
        self
    }

    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTheme>) -> Self {
        self.model.autocomplete_theme = theme;
        self
    }

    pub fn popup_look_provider(mut self, provider: AutocompletePopupLookProvider) -> Self {
        self.model.popup_look_provider = provider;
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<AutocompleteControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| AutocompleteControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> AutocompleteBuilder {
    AutocompleteBuilder::new(id, items)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_autocomplete_template();
        let builder = AutocompleteBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn template_modifier_composes_with_items_template() {
        let template = default_autocomplete_template();
        let items_template = default_autocomplete_items_template();
        let builder = AutocompleteBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .items_template(items_template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(Arc::ptr_eq(&builder.model.items_template, &items_template));
    }

    #[test]
    fn with_items_template_modifier_wraps_items_template() {
        let items_template = default_autocomplete_items_template();
        let builder = AutocompleteBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .items_template(items_template.clone())
            .with_items_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.items_template, &items_template));
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;

    #[test]
    fn affordance_snapshot_keeps_size_overrides_and_preserves_custom_theme_provider() {
        let mut sheet = crate::theme::stylesheet::CommonStylesheet::parse(
            "[common.autocomplete.geometry]\nclear_width=27\n[common.autocomplete.sizes.sm]\nclear_width=0",
        )
        .unwrap();
        let original = AutocompleteBuilder::new("snapshot", []);
        let theme = original.model.autocomplete_theme.clone();
        let builder = original.stylesheet(&sheet).size(ControlSize::Sm);
        sheet.autocomplete.geometry.clear_width = Some(41.0);
        assert!(Arc::ptr_eq(&theme, &builder.model.autocomplete_theme));
        assert_eq!(
            builder
                .model
                .stylesheet
                .resolve_geometry(builder.model.size.stylesheet_key(), Default::default())
                .clear_width,
            0.0
        );
        assert_eq!(builder.model.stylesheet.resolve_geometry("md", Default::default()).clear_width, 27.0);
        assert_eq!(builder.model.stylesheet.resolve_geometry("md", Default::default()).clear_icon_size, 12.0);
    }
}
