use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, IntoElement, SharedString, Stateful, Window};

use crate::controls::control_group::{
    self as control_group, ControlGroupBuilder as InnerControlGroupBuilder, ControlGroupChromeModel,
    ControlGroupFocusStrategy, ControlGroupFocusTarget, ControlGroupFocusTargetProvider, ControlGroupItem,
    ControlGroupItemElementTemplate, ControlGroupItemElements, ControlGroupItemLike, ControlGroupItemRenderModel,
    ControlGroupItemTemplate, ControlGroupItemVisualContext, ControlGroupLayout, ControlGroupRenderModel,
    ControlGroupTemplate, ControlGroupTemplateHandlers, ControlGroupTheme, ControlSelectionMode,
};

use super::control::RadioGroupControl;
use super::template::radio_group_container_template;

pub type RadioGroup<T> = Entity<RadioGroupControl<T>>;
pub type RadioGroupItem = ControlGroupItem;
pub type RadioGroupRenderModel<'a, T> = ControlGroupRenderModel<'a, T>;
pub type RadioGroupTemplate<T> = ControlGroupTemplate<T>;
pub type RadioGroupTemplateHandlers = ControlGroupTemplateHandlers;
pub type SelectionMode = ControlSelectionMode;
pub use crate::controls::control_group::ControlGroupItemLike as RadioGroupItemLike;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RadioGroupEvent {
    Change { value: Option<SharedString> },
    Activate { value: SharedString },
    FocusChanged { focused: bool },
    ItemFocused { item_id: SharedString },
}

pub struct RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::radio_group::RadioGroupEvent>,

    pub(super) inner: InnerControlGroupBuilder<T>,
}

impl<T> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::radio_group::RadioGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `RadioGroupEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::Activate { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    crate::forward_motion!(crate::controls::control_group::ControlGroupMotion, builder inner);
    pub fn motion_defaults(mut self, profile: crate::controls::control_group::ControlGroupMotion) -> Self {
        self.inner = self.inner.motion_defaults(profile);
        self
    }
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.motion_enabled(enabled);
        self
    }

    fn from_inner(inner: InnerControlGroupBuilder<T>) -> Self {
        Self { callbacks: Default::default(), inner }
    }

    /// Capture visual defaults for the group and its radio item templates.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        self.inner = self.inner.stylesheet(stylesheet);
        self
    }

    pub fn layout(self, layout: ControlGroupLayout) -> Self {
        Self { inner: self.inner.layout(layout), ..self }
    }

    pub fn horizontal(self) -> Self {
        Self { inner: self.inner.horizontal(), ..self }
    }

    pub fn vertical(self) -> Self {
        Self { inner: self.inner.vertical(), ..self }
    }

    pub fn item(self, item: T) -> Self {
        Self { inner: self.inner.item(item), ..self }
    }

    pub fn items(self, items: impl IntoIterator<Item = T>) -> Self {
        Self { inner: self.inner.items(items), ..self }
    }

    pub fn mode(self, selection_mode: ControlSelectionMode) -> Self {
        Self { inner: self.inner.mode(selection_mode), ..self }
    }

    pub fn selection_mode(self, selection_mode: ControlSelectionMode) -> Self {
        Self { inner: self.inner.selection_mode(selection_mode), ..self }
    }

    pub fn selection_follows_active(self, selection_follows_active: bool) -> Self {
        Self { inner: self.inner.selection_follows_active(selection_follows_active), ..self }
    }

    pub fn focus_strategy(self, focus_strategy: ControlGroupFocusStrategy) -> Self {
        Self { inner: self.inner.focus_strategy(focus_strategy), ..self }
    }

    pub fn active_descendant(self) -> Self {
        Self { inner: self.inner.active_descendant(), ..self }
    }

    pub fn roving_item_focus(self) -> Self {
        Self { inner: self.inner.roving_item_focus(), ..self }
    }

    pub fn focus_target_provider(self, provider: ControlGroupFocusTargetProvider<T>) -> Self {
        Self { inner: self.inner.focus_target_provider(provider), ..self }
    }

    pub fn with_focus_target_provider<F>(self, provider: F) -> Self
    where
        F: for<'a> Fn(&'a T, &mut Window, &mut App) -> Option<ControlGroupFocusTarget> + Send + Sync + 'static,
    {
        Self { inner: self.inner.with_focus_target_provider(provider), ..self }
    }

    pub fn clear_focus_target_provider(self) -> Self {
        Self { inner: self.inner.clear_focus_target_provider(), ..self }
    }

    pub fn single_required(self) -> Self {
        Self { inner: self.inner.single_required(), ..self }
    }

    pub fn single_allow_none(self) -> Self {
        Self { inner: self.inner.single_allow_none(), ..self }
    }

    pub fn selected(self, selected_id: impl Into<SharedString>) -> Self {
        Self { inner: self.inner.selected(selected_id), ..self }
    }

    pub fn selected_ids<I, S>(self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        Self { inner: self.inner.selected_ids(selected_ids), ..self }
    }

    pub fn managed(self) -> Self {
        Self { inner: self.inner.managed(), ..self }
    }

    pub fn unmanaged(self) -> Self {
        Self { inner: self.inner.unmanaged(), ..self }
    }

    pub fn managed_selected(self, selected_id: impl Into<SharedString>) -> Self {
        Self { inner: self.inner.managed_selected(selected_id), ..self }
    }

    pub fn managed_selected_ids<I, S>(self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        Self { inner: self.inner.managed_selected_ids(selected_ids), ..self }
    }

    pub fn active(self, active_id: impl Into<SharedString>) -> Self {
        Self { inner: self.inner.active(active_id), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { inner: self.inner.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { inner: self.inner.tab_stop(tab_stop), ..self }
    }

    pub fn item_template(self, template: ControlGroupItemTemplate<T>) -> Self {
        Self { inner: self.inner.item_template(template), ..self }
    }

    pub fn item_element_template(self, template: ControlGroupItemElementTemplate<T>) -> Self {
        Self { inner: self.inner.item_element_template(template), ..self }
    }

    pub fn with_item_template<F, E>(self, template: F) -> Self
    where
        F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        Self { inner: self.inner.with_item_template(template), ..self }
    }

    pub fn with_item_template_modifier<F>(self, modifier: F) -> Self
    where
        F: for<'a> Fn(AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        Self { inner: self.inner.with_item_template_modifier(modifier), ..self }
    }

    pub fn clear_item_template(self) -> Self {
        Self { inner: self.inner.clear_item_template(), ..self }
    }

    pub fn clear_item_element_template(self) -> Self {
        Self { inner: self.inner.clear_item_element_template(), ..self }
    }

    pub fn template(self, template: ControlGroupTemplate<T>) -> Self {
        Self { inner: self.inner.template(template), ..self }
    }

    pub fn with_template_modifier<F>(self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
    {
        Self { inner: self.inner.with_template_modifier(modifier), ..self }
    }

    pub fn with_template<F>(self, template: F) -> Self
    where
        F: for<'a> Fn(
                &ControlGroupRenderModel<'a, T>,
                ControlGroupTemplateHandlers,
                &mut Window,
                &mut App,
            ) -> Stateful<Div>
            + Send
            + Sync
            + 'static,
    {
        Self { inner: self.inner.with_template(template), ..self }
    }

    pub fn with_item_layout<F, E>(self, layout: F) -> Self
    where
        F: for<'a> Fn(ControlGroupItemElements, &ControlGroupRenderModel<'a, T>, &mut Window, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        Self { inner: self.inner.with_item_layout(layout), ..self }
    }

    pub fn with_menu_row_item_content<F>(self, theme: Arc<dyn ControlGroupTheme>, content: F) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        {
            let geometry = crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry();
            self.with_menu_row_item_content_sized(theme, geometry.menu_row_height, geometry.menu_row_gap, content)
        }
    }

    pub fn with_menu_row_item_content_sized<F>(
        self,
        theme: Arc<dyn ControlGroupTheme>,
        row_height: f32,
        row_radius: f32,
        content: F,
    ) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        Self { inner: self.inner.with_menu_row_item_content_sized(theme, row_height, row_radius, content), ..self }
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> RadioGroup<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| RadioGroupControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub fn new<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    RadioGroupBuilder::from_inner(control_group::new(id).single_required().template(radio_group_container_template()))
}

pub fn horizontal<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    new(id).horizontal()
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;

    #[test]
    fn radio_builder_forwards_snapshot_without_changing_selection_mode() {
        let mut sheet = crate::theme::stylesheet::CommonStylesheet::resolve_layers(&[
            "[common.radio_group.sizes.horizontal]\ngap=17",
        ])
        .unwrap();
        let group = new::<ControlGroupItem>("radio").horizontal().stylesheet(&sheet);
        sheet.radio_group.sizes.get_mut("horizontal").unwrap().gap = Some(23.0);
        assert_eq!(group.inner.model.stylesheet.radio_group.authored_geometry_for("horizontal").gap, Some(17.0));
        assert_eq!(group.inner.model.layout, ControlGroupLayout::Horizontal);
        assert_eq!(group.inner.model.selection_mode, ControlSelectionMode::SingleRequired);
    }
}
