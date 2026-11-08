//! Look-owned icon-group builder. Spawn synthesizes the SDK icon-group control-group.

use gpui::{App, Context, Entity, IntoElement, SharedString};
use xgpui_luma::controls::control_group::{
    ControlGroupBuilder, ControlGroupControl, ControlGroupItemLike, ControlGroupItemTemplate, ControlGroupRenderModel,
    ControlGroupTemplate,
};
use xgpui_luma::controls::icon_group::{self, IconGroupItem};

use crate::look::{ShadcnLook, resolve_look_from};

enum IconGroupKind {
    Group,
    Toolbar,
}

/// Builder in the guise of an icon group: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct IconGroup<T = IconGroupItem>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::control_group::ControlGroupEvent>,

    look: Option<ShadcnLook>,
    kind: IconGroupKind,
    builder: ControlGroupBuilder<T>,
    custom_template: bool,
}

impl<T> IconGroup<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    xgpui_luma::forward_motion!(xgpui_luma::controls::control_group::ControlGroupMotion, builder builder);

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
            kind: IconGroupKind::Group,
            builder: icon_group::new(id),
            custom_template: false,
        }
    }

    pub fn toolbar(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            kind: IconGroupKind::Toolbar,
            builder: icon_group::new(id).horizontal(),
            custom_template: false,
        }
    }

    pub fn toolbar_multiple(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            kind: IconGroupKind::Toolbar,
            builder: icon_group::new(id).horizontal().multiple(),
            custom_template: false,
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

    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    pub fn animated_selection(mut self, animated_selection: bool) -> Self {
        self.builder = self.builder.animated_selection(animated_selection);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.builder = self.builder.vertical();
        self
    }

    pub fn item_template(mut self, template: ControlGroupItemTemplate<T>) -> Self {
        self.builder = self.builder.item_template(template);
        self
    }

    pub fn template(mut self, template: ControlGroupTemplate<T>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn with_item_layout<F, E>(mut self, layout: F) -> Self
    where
        F: for<'a> Fn(
                xgpui_luma::controls::control_group::ControlGroupItemElements,
                &ControlGroupRenderModel<'a, T>,
                &mut gpui::Window,
                &mut App,
            ) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.custom_template = true;
        self.builder = self.builder.with_item_layout(layout);
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
        if self.custom_template {
            return self_builder;
        }
        match self.kind {
            IconGroupKind::Group => self_builder.template(look.control_group_template()),
            IconGroupKind::Toolbar => {
                self_builder.template(icon_group::icon_toolbar_template(look.control_group_theme()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = IconGroup::new("ok")
            .look(&look)
            .items([IconGroupItem::new("a").label("A")])
            .selected("a")
            .into_sdk_builder(look);
    }

    #[test]
    fn toolbar_into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = IconGroup::toolbar("ok")
            .look(&look)
            .items([IconGroupItem::new("a").label("A")])
            .selected("a")
            .into_sdk_builder(look.clone());
        let _multiple = IconGroup::toolbar_multiple("ok")
            .look(&look)
            .items([IconGroupItem::new("a").label("A")])
            .into_sdk_builder(look);
    }
}
