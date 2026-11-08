//! Look-owned menu-choice-group builder. Spawn synthesizes an SDK control-group.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, SharedString, Window};
use xgpui_luma::controls::control_group::{
    ControlGroupBuilder, ControlGroupControl, ControlGroupItem, ControlGroupItemLike, ControlGroupItemRenderModel,
    ControlGroupItemVisualContext, MenuChoiceRowContentFn,
};

use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

struct MenuRowConfig<T>
where
    T: ControlGroupItemLike + 'static,
{
    size: ShadcnSize,
    row_height: f32,
    row_radius: f32,
    content: MenuChoiceRowContentFn<T>,
}

/// Builder in the guise of a menu choice group: Shadcn theme plus SDK options, until `.spawn(cx)`.
pub struct MenuChoiceGroup<T = ControlGroupItem>
where
    T: ControlGroupItemLike + 'static,
{
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::control_group::ControlGroupEvent>,

    look: Option<ShadcnLook>,
    builder: ControlGroupBuilder<T>,
    menu_row: Option<MenuRowConfig<T>>,
}

impl<T> MenuChoiceGroup<T>
where
    T: ControlGroupItemLike + 'static,
{
    xgpui_luma::forward_motion!(xgpui_luma::controls::control_group::ControlGroupMotion, builder builder);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::control_group::ControlGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ControlGroupEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::control_group::ControlGroupEvent::Activate { activated_id } = event {
                callback(owner, activated_id, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::control_group::ControlGroupEvent::Change {
                changed_id,
                selected,
                selected_ids,
            } = event
            {
                callback(owner, changed_id, *selected, selected_ids, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemBoundsChanged` through the existing event stream.
    pub fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::control_group::ControlGroupEvent::ItemBoundsChanged { item_id, bounds } = event
            {
                callback(owner, item_id, bounds, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::control_group::ControlGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::control_group::ControlGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),
            look: None,
            builder: xgpui_luma::controls::control_group::new(id).active_descendant(),
            menu_row: None,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(selected_id);
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.builder = self.builder.selected_ids(selected_ids);
        self
    }

    pub fn multiple(mut self) -> Self {
        self.builder = self.builder.multiple();
        self
    }

    pub fn single_required(mut self) -> Self {
        self.builder = self.builder.single_required();
        self
    }

    pub fn single_allow_none(mut self) -> Self {
        self.builder = self.builder.single_allow_none();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn with_menu_row_item_content<F>(self, content: F) -> Self
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
        self.with_menu_row_item_content_sized(ShadcnSize::Sm, 27.0, 4.0, content)
    }

    pub fn with_menu_row_item_content_sized<F>(
        mut self,
        size: ShadcnSize,
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
        self.menu_row = Some(MenuRowConfig { size, row_height, row_radius, content: Arc::new(content) });
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<ControlGroupControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ControlGroupBuilder<T> {
        let self_builder = self
            .builder
            .stylesheet(&look.mode_tokens().stylesheet().common)
            .motion_defaults(look.motion_defaults().control_group);
        let mut builder = self_builder;
        if let Some(row) = self.menu_row {
            let content = row.content;
            builder = builder.with_menu_row_item_content_sized(
                look.control_group_theme(),
                row.size.control_size(),
                row.row_height,
                row.row_radius,
                move |model, visual, window, cx| content(model, visual, window, cx),
            );
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::prelude::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = MenuChoiceGroup::new("ok")
            .look(&look)
            .items([ControlGroupItem::new("a").label("A")])
            .selected("a")
            .with_menu_row_item_content(|model, _visual, _window, _cx| {
                gpui::div().child(model.item.label().clone()).into_any_element()
            })
            .into_sdk_builder(look);
    }
}
