crate::motion_profile!(ComboBoxMotion => combobox {
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled};

use super::behavior::SelectionItem;
use super::control::ComboBoxControl;
use crate::controls::autocomplete::{AutocompleteTheme, default_autocomplete_theme};
use super::item_template::{
    ComboBoxItemRenderModel, ComboBoxItemTemplate, item_template_with_modifier, make_combobox_item_template,
};
use super::panel_template::{ComboBoxPanelTemplate, default_combobox_panel_template, panel_template_with_modifier};
use super::template::{
    ComboBoxItemsTemplate, ComboBoxTemplate, default_combobox_items_template, default_combobox_template,
    template_with_modifier,
};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum TypingPolicy {
    #[default]
    Flexible,
    Strict,
}

#[derive(Clone)]
pub struct ComboBoxModel {
    pub(crate) icons: crate::theme::stylesheet::IconsStylesheet,
    pub(crate) motion: crate::motion::MotionConfig<ComboBoxMotion>,
    pub(crate) geometry: crate::theme::stylesheet::ComboBoxGeometry,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) enabled: bool,
    /// Build the diagnostic status line (search status, selection, last event and key); off by default.
    pub(crate) debug_status: bool,
    pub(crate) scroll_interaction: crate::interaction::ScrollInteraction,
    pub(crate) invalid: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) typing_policy: TypingPolicy,
    pub(crate) show_down_arrow: bool,
    pub(crate) show_clear_button: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows_explicit: bool,
    pub(crate) max_visible_rows_explicit: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn ComboBoxTemplate>,
    pub(crate) items_template: Arc<dyn ComboBoxItemsTemplate>,
    pub(crate) panel_template: Arc<dyn ComboBoxPanelTemplate>,
    pub(crate) item_template: Option<ComboBoxItemTemplate<SelectionItem>>,
    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTheme>,
    pub(crate) popup_menu: crate::controls::popup_menu::MenuSource,
}

pub struct ComboBoxBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::combobox::ComboBoxEvent>,

    pub(crate) model: ComboBoxModel,
}

impl ComboBoxBuilder {
    crate::motion_methods!(ComboBoxMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::combobox::ComboBoxEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ComboBoxEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::Change { query } = event {
                callback(owner, query, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::Complete` through the existing event stream.
    pub fn on_complete<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::Complete { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::Clear` through the existing event stream.
    pub fn on_clear<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::Clear = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `ComboBoxEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::combobox::ComboBoxEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            callbacks: Default::default(),

            model: ComboBoxModel {
                motion: Default::default(),
                icons: crate::theme::stylesheet::CommonStylesheet::sdk_default().icons.clone(),
                geometry: crate::theme::stylesheet::ComboBoxGeometry::sdk(),
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Select…"),
                enabled: true,
                debug_status: false,
                scroll_interaction: crate::interaction::ScrollInteraction::VIEWPORT,
                invalid: false,
                full_width: true,
                clean_on_escape: true,
                typing_policy: TypingPolicy::Flexible,
                show_down_arrow: true,
                show_clear_button: true,
                scrolling: true,
                min_visible_rows_explicit: false,
                max_visible_rows_explicit: false,
                min_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .row_limits
                    .resolve("combobox")
                    .0,
                max_visible_rows: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                    .row_limits
                    .resolve("combobox")
                    .1,
                textfield_template: default_textfield_template(),
                scrollbar_template: default_scrollbar_template(),
                template: default_combobox_template(),
                items_template: default_combobox_items_template(),
                panel_template: default_combobox_panel_template(),
                item_template: None,
                autocomplete_theme: default_autocomplete_theme(),
                popup_menu: Arc::new(|| crate::style::StyleBinding::sdk("floating_menu")),
            },
        }
    }

    /// Capture shared affordance geometry; per-size values resolve at render time.
    /// Text-field/popup providers and consumer template modifiers retain their own precedence.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let (minimum, maximum) = stylesheet.row_limits.resolve("combobox");
        if !self.model.min_visible_rows_explicit {
            self.model.min_visible_rows = minimum;
        }
        if !self.model.max_visible_rows_explicit {
            self.model.max_visible_rows = maximum;
        }
        self.model.icons = stylesheet.with_sdk_defaults().icons;
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

    /// Show a diagnostic status line (search status, selection, match count, last event and key)
    /// in the render model's `status_label` / `status_detail`. Off by default: both are empty.
    pub fn debug_status(mut self, debug_status: bool) -> Self {
        self.model.debug_status = debug_status;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
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

    pub fn typing_policy(mut self, typing_policy: TypingPolicy) -> Self {
        self.model.typing_policy = typing_policy;
        self
    }

    pub fn show_down_arrow(mut self, show_down_arrow: bool) -> Self {
        self.model.show_down_arrow = show_down_arrow;
        self
    }

    pub fn show_clear_button(mut self, show_clear_button: bool) -> Self {
        self.model.show_clear_button = show_clear_button;
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

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn ComboBoxTemplate>) -> Self {
        self.model.template = template;
        self
    }

    /// Transitional row-list template hook.
    ///
    /// Prefer `panel_template(...)` and `with_item_template(...)` for new code.
    pub fn items_template(mut self, template: Arc<dyn ComboBoxItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn ComboBoxPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTheme>) -> Self {
        self.model.autocomplete_theme = theme;
        self
    }

    /// Where the popup panel's style comes from: a `floating_menu` binding, read each time it is drawn (#433).
    pub fn popup_menu(mut self, menu: crate::controls::popup_menu::MenuSource) -> Self {
        self.model.popup_menu = menu;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&ComboBoxItemRenderModel<'a, SelectionItem>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_combobox_item_template(template));
        self
    }

    pub fn with_item_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::AnyElement, &ComboBoxItemRenderModel<'a, SelectionItem>, &mut App) -> gpui::AnyElement
            + Send
            + Sync
            + 'static,
    {
        let base = self.model.item_template.take().unwrap_or_else(|| {
            make_combobox_item_template(|item: &ComboBoxItemRenderModel<'_, SelectionItem>, _cx| {
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
        F: Fn(gpui::AnyElement, &mut App) -> gpui::AnyElement + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<ComboBoxControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| ComboBoxControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> ComboBoxBuilder {
    ComboBoxBuilder::new(id, items)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn with_item_template_modifier_creates_template_from_default_content() {
        let builder = ComboBoxBuilder::new("combobox-test", Vec::<SelectionItem>::new())
            .with_item_template_modifier(|content, _, _| content);

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn with_panel_template_modifier_wraps_panel_template() {
        let template = default_combobox_panel_template();
        let builder = ComboBoxBuilder::new("combobox-test", Vec::<SelectionItem>::new())
            .panel_template(template.clone())
            .with_panel_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.panel_template, &template));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_combobox_template();
        let builder = ComboBoxBuilder::new("combobox-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_item_template_modifier_wraps_existing_item_template() {
        let builder = ComboBoxBuilder::new("combobox-test", Vec::<SelectionItem>::new())
            .with_item_template(|_item: &ComboBoxItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
        let item_template = builder.model.item_template.as_ref().unwrap().clone();
        let builder = builder.with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn template_panel_and_item_modifiers_compose_on_builder() {
        let template = default_combobox_template();
        let panel_template = default_combobox_panel_template();
        let builder = ComboBoxBuilder::new("combobox-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .panel_template(panel_template.clone())
            .with_item_template(|_item: &ComboBoxItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
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

    /// The affordance geometry is `combobox.lss`'s (#287).
    #[test]
    fn combobox_affordances_read_combobox_lss() {
        let builder = ComboBoxBuilder::new("geometry", []);
        let sdk = |name| crate::style::sdk_number("combobox", name);
        let geometry = builder.model.geometry;
        assert_eq!(
            (geometry.affordance_inset, geometry.affordance_width, geometry.affordance_icon_size),
            (sdk("affordance_inset"), sdk("affordance_width"), sdk("affordance_icon_size"))
        );
    }
}
