//! Look-owned search-selector builder. Spawn synthesizes the SDK [`xgpui_luma::controls::search_selector::SearchSelector`].

use std::sync::Arc;

use gpui::{App, Context, Entity, IntoElement, SharedString};
use xgpui_luma::controls::search_selector::{
    SearchSelectorBuilder, SearchSelectorControl, SearchSelectorItemRenderModel, SearchSelectorPanelTemplate,
    SearchSelectorTemplate, SelectionItem,
};
use xgpui_luma::controls::selector::SelectorTriggerStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a search selector: Shadcn templates plus SDK options, until `.spawn(cx)`.
pub struct SearchSelector {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::search_selector::SearchSelectorEvent>,

    look: Option<ShadcnLook>,
    builder: SearchSelectorBuilder,
    size: ShadcnSize,
}

impl SearchSelector {
    xgpui_luma::forward_motion!(xgpui_luma::controls::search_selector::SearchSelectorMotion, builder builder);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::search_selector::SearchSelectorEvent, &mut gpui::Context<M>)
        + 'static,
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::Change { query } = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::Select { item_id, label } = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::Complete { item_id, label } = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::Clear = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::FocusChanged { focused } = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::OpenChanged { open } = event {
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
            if let xgpui_luma::controls::search_selector::SearchSelectorEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            callbacks: Default::default(),
            look: None,
            builder: xgpui_luma::controls::search_selector::new(id, items),
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.builder = self.builder.placeholder(placeholder);
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.builder = self.builder.search_placeholder(placeholder);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.builder = self.builder.wheel_scroll_policy(policy);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.builder = self.builder.scroll_boundary_policy(policy);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.builder = self.builder.wheel_focus_scope(policy);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.builder = self.builder.invalid(invalid);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected_id(selected_id);
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.builder = self.builder.full_width(full_width);
        self
    }

    pub fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.builder = self.builder.clean_on_escape(clean_on_escape);
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.builder = self.builder.scrolling(scrolling);
        self
    }

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.builder = self.builder.min_visible_rows(min_visible_rows);
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.builder = self.builder.max_visible_rows(max_visible_rows);
        self
    }

    pub fn fill_popup_viewport(mut self, fill_popup_viewport: bool) -> Self {
        self.builder = self.builder.fill_popup_viewport(fill_popup_viewport);
        self
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.builder = self.builder.trigger_style(style);
        self
    }

    pub fn ghost(mut self) -> Self {
        self.builder = self.builder.ghost();
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.builder = self.builder.without_elevation();
        self
    }

    pub fn template(mut self, template: Arc<dyn SearchSelectorTemplate>) -> Self {
        self.builder = self.builder.template(template);
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SearchSelectorPanelTemplate>) -> Self {
        self.builder = self.builder.panel_template(template);
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SearchSelectorItemRenderModel<'a, SelectionItem>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.builder = self.builder.with_item_template(template);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<SearchSelectorControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            use xgpui_luma::infra::attachments::TooltipEntityExt;
            let theme = crate::tooltip_theme(&look);
            self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SearchSelectorBuilder {
        let self_builder = self
            .builder
            .stylesheet(&look.mode_tokens().stylesheet().common)
            .motion_defaults(look.motion_defaults().search_selector);
        let theme = look.clone();
        self_builder
            .size(self.size.control_size())
            .textfield_template(theme.input_textfield_template())
            .autocomplete_theme(theme.autocomplete_theme())
            .selector_theme(theme.selector_theme())
            .scrollbar_template(theme.scrollbar_template())
            .popup_look_provider(Arc::new(move |size| {
                theme.selector_items_panel_look(ShadcnSize::from_control_size(size))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = SearchSelector::new("ok", [SelectionItem::new("a", "A")])
            .look(&look)
            .placeholder("Choose…")
            .into_sdk_builder(look);
    }
}
