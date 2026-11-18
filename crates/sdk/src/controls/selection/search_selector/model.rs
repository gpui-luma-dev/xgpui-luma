crate::motion_profile!(SearchSelectorMotion => search_selector {
    disclosure: crate::motion::Transition,
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled};

use super::behavior::SelectionItem;
use super::control::SearchSelectorControl;
use crate::controls::autocomplete::{AutocompleteTheme, default_autocomplete_theme};
use super::item_template::{
    SearchSelectorItemRenderModel, SearchSelectorItemTemplate, item_template_with_modifier,
    make_search_selector_item_template,
};
use super::panel_template::{
    SearchSelectorPanelTemplate, default_search_selector_panel_template, panel_template_with_modifier,
};
use super::template::{
    SearchSelectorItemsTemplate, SearchSelectorTemplate, default_search_selector_items_template,
    default_search_selector_template, template_with_modifier,
};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::selector::{SelectorTheme, SelectorTriggerStyle, default_selector_theme};
use crate::controls::selector_list::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::ThemeTokens;

pub type SearchSelectorPopupLookProvider = Arc<dyn Fn() -> SelectorItemsPanelLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SearchSelectorModel {
    pub(crate) icons: crate::theme::stylesheet::IconsStylesheet,
    pub(crate) motion: crate::motion::MotionConfig<SearchSelectorMotion>,
    pub(crate) stylesheet: crate::theme::stylesheet::SearchSelectorStylesheet,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) search_placeholder: SharedString,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: crate::interaction::ScrollInteraction,
    pub(crate) invalid: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows_explicit: bool,
    pub(crate) max_visible_rows_explicit: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) fill_popup_viewport: bool,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) selector_theme: Arc<dyn SelectorTheme>,
    pub(crate) trigger_style: SelectorTriggerStyle,
    pub(crate) without_elevation: bool,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn SearchSelectorTemplate>,
    pub(crate) items_template: Arc<dyn SearchSelectorItemsTemplate>,
    pub(crate) panel_template: Arc<dyn SearchSelectorPanelTemplate>,
    pub(crate) item_template: Option<SearchSelectorItemTemplate<SelectionItem>>,
    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTheme>,
    pub(crate) popup_look_provider: SearchSelectorPopupLookProvider,
}

pub struct SearchSelectorBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::search_selector::SearchSelectorEvent>,

    pub(crate) model: SearchSelectorModel,
    pub(crate) initial_selected_id: Option<SharedString>,
}

impl SearchSelectorBuilder {
    crate::motion_methods!(SearchSelectorMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::search_selector::SearchSelectorEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SearchSelectorEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::Change { query } = event {
                callback(owner, query, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::Complete` through the existing event stream.
    pub fn on_complete<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::Complete { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::Clear` through the existing event stream.
    pub fn on_clear<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::Clear = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SearchSelectorEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::search_selector::SearchSelectorEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            callbacks: Default::default(),

            initial_selected_id: None,
            model: SearchSelectorModel {
                motion: Default::default(),
                icons: crate::theme::stylesheet::CommonStylesheet::sdk_default().icons.clone(),
                stylesheet: crate::theme::stylesheet::CommonStylesheet::sdk_default().search_selector.clone(),
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Select…"),
                search_placeholder: SharedString::from("Selection search"),
                enabled: true,
                scroll_interaction: crate::interaction::ScrollInteraction::VIEWPORT,
                invalid: false,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                min_visible_rows_explicit: false,
                max_visible_rows_explicit: false,
                min_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .row_limits
                    .resolve("search_selector")
                    .0,
                max_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .row_limits
                    .resolve("search_selector")
                    .1,
                fill_popup_viewport: false,
                textfield_template: default_textfield_template(),
                selector_theme: default_selector_theme(),
                trigger_style: SelectorTriggerStyle::default(),
                without_elevation: false,
                scrollbar_template: default_scrollbar_template(),
                template: default_search_selector_template(),
                items_template: default_search_selector_items_template(),
                panel_template: default_search_selector_panel_template(),
                item_template: None,
                autocomplete_theme: default_autocomplete_theme(),
                popup_look_provider: Arc::new(|| default_selector_items_panel_look(&ThemeTokens::default())),
            },
        }
    }

    /// Capture shared affordance geometry; per-size values resolve at render time.
    /// Text-field/popup providers and consumer template modifiers retain their own precedence.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let (minimum, maximum) = stylesheet.row_limits.resolve("search_selector");
        if !self.model.min_visible_rows_explicit {
            self.model.min_visible_rows = minimum;
        }
        if !self.model.max_visible_rows_explicit {
            self.model.max_visible_rows = maximum;
        }
        self.model.icons = stylesheet.with_sdk_defaults().icons;
        self.model.stylesheet = stylesheet.with_sdk_defaults().search_selector;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder = placeholder.into();
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.search_placeholder = placeholder.into();
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

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.initial_selected_id = Some(selected_id.into());
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

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.model.min_visible_rows_explicit = true;
        self.model.min_visible_rows = min_visible_rows.max(1);
        if self.model.max_visible_rows < self.model.min_visible_rows {
            self.model.max_visible_rows = self.model.min_visible_rows;
        }
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.model.max_visible_rows_explicit = true;
        self.model.max_visible_rows = max_visible_rows.max(1);
        if self.model.min_visible_rows > self.model.max_visible_rows {
            self.model.min_visible_rows = self.model.max_visible_rows;
        }
        self
    }

    /// Expand the popup list to use available viewport space below the trigger,
    /// similar to `Selector` smart placement, instead of capping at `max_visible_rows`.
    pub fn fill_popup_viewport(mut self, fill_popup_viewport: bool) -> Self {
        self.model.fill_popup_viewport = fill_popup_viewport;
        self
    }

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    /// Theme for the read-only display trigger (Outline/Ghost command-button chrome).
    pub fn selector_theme(mut self, theme: Arc<dyn SelectorTheme>) -> Self {
        self.model.selector_theme = theme;
        self
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.model.trigger_style = SelectorTriggerStyle::Ghost;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn SearchSelectorTemplate>) -> Self {
        self.model.template = template;
        self
    }

    /// Transitional row-list template hook.
    ///
    /// Prefer `panel_template(...)` and `with_item_template(...)` for new code.
    #[deprecated(note = "Prefer panel_template(...) + with_item_template(...)")]
    pub fn items_template(mut self, template: Arc<dyn SearchSelectorItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SearchSelectorPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTheme>) -> Self {
        self.model.autocomplete_theme = theme;
        self
    }

    pub fn popup_look_provider(mut self, provider: SearchSelectorPopupLookProvider) -> Self {
        self.model.popup_look_provider = provider;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SearchSelectorItemRenderModel<'a, SelectionItem>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_search_selector_item_template(template));
        self
    }

    pub fn with_item_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::AnyElement,
                &SearchSelectorItemRenderModel<'a, SelectionItem>,
                &mut App,
            ) -> gpui::AnyElement
            + Send
            + Sync
            + 'static,
    {
        let base = self.model.item_template.take().unwrap_or_else(|| {
            make_search_selector_item_template(|item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| {
                gpui::div().flex_1().child(item.item.label.clone())
            })
        });
        self.model.item_template = Some(item_template_with_modifier(base, modifier));
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::AnyElement, &mut App) -> gpui::AnyElement + Send + Sync + 'static,
    {
        self.model.panel_template = panel_template_with_modifier(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &mut App) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SearchSelectorControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| SearchSelectorControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> SearchSelectorBuilder {
    SearchSelectorBuilder::new(id, items)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn with_item_template_modifier_creates_template_from_default_content() {
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .with_item_template_modifier(|content, _, _| content);

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn with_panel_template_modifier_wraps_panel_template() {
        let template = default_search_selector_panel_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .panel_template(template.clone())
            .with_panel_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.panel_template, &template));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_search_selector_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_item_template_modifier_wraps_existing_item_template() {
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .with_item_template(|_item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
        let item_template = builder.model.item_template.as_ref().unwrap().clone();
        let builder = builder.with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn template_panel_and_item_modifiers_compose_on_builder() {
        let template = default_search_selector_template();
        let panel_template = default_search_selector_panel_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .panel_template(panel_template.clone())
            .with_item_template(|_item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
        let item_template = builder.model.item_template.as_ref().unwrap().clone();
        let builder = builder
            .with_template_modifier(|element, _| element)
            .with_panel_template_modifier(|element, _| element)
            .with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(!Arc::ptr_eq(&builder.model.panel_template, &panel_template));
        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;

    #[test]
    fn search_affordance_snapshot_inherits_and_preserves_zero() {
        let sheet = crate::theme::stylesheet::CommonStylesheet::parse("[common.search_selector.geometry]\nsearch_padding=0\nsearch_separator_height=0\nsearch_icon_size=21\nsearch_chrome_height=0").unwrap();
        let builder = SearchSelectorBuilder::new("geometry", []).stylesheet(&sheet);
        let geometry = builder.model.stylesheet.resolve_geometry(Default::default());
        assert_eq!(
            (
                geometry.search_padding,
                geometry.search_separator_height,
                geometry.search_icon_size,
                geometry.search_chrome_height
            ),
            (0.0, 0.0, 21.0, 0.0)
        );
        assert_eq!(geometry.search_icon_width, 18.0);
    }
}
