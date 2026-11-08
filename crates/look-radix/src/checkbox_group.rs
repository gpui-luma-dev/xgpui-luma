//! Multiple-selection Radix checkbox groups using SDK group state and input.

use std::sync::Arc;
use gpui::{Context, SharedString, div, prelude::*};
use xgpui_luma::controls::button::ButtonRenderModel;
use xgpui_luma::controls::checkbox::CheckboxData;
use xgpui_luma::controls::control_group::{
    self, ControlGroupBuilder, ControlGroupItem, ControlGroupItemLike, ControlGroupItemHandlerExt, ControlGroupLayout,
    ControlGroupTemplate,
};
use crate::{Look, Paint, CheckboxSize, CheckboxVariant, Radius, checkbox_template_for};
use crate::look::resolve_look;

/// Checkbox rows with SDK-owned focus, disabled state and multiple selection.
pub fn checkbox_group_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
) -> ControlGroupTemplate<T> {
    let template = checkbox_template_for(look, variant, paint, size, radius);
    Arc::new(move |model, handlers, window, cx| {
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let label = item.item.label().clone();
                let button = ButtonRenderModel {
                    id: format!("{}-{}", item.group_id, item.item.id()).into(),
                    data: CheckboxData::new(item.selected),
                    content: Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
                    state: item.state.interaction_state(),
                    size: size.control_size(),
                    ..Default::default()
                };
                let selector = button.id.to_string();
                template
                    .render(&button, window, cx)
                    .debug_selector(move || selector.clone())
                    .control_group_item_handlers(handlers)
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        div()
            .id(model.id.clone())
            .flex()
            .gap_2()
            .items_start()
            .when(model.layout == ControlGroupLayout::Vertical, |group| group.flex_col())
            .children(rows)
    })
}

/// Look-owned checkbox group. Subscribe to SDK `ControlGroupEvent` for changes.
pub struct CheckboxGroup<T: ControlGroupItemLike + 'static = ControlGroupItem> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::control_group::ControlGroupEvent>,

    look: Option<Look>,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
    builder: ControlGroupBuilder<T>,
}
impl<T: ControlGroupItemLike + 'static> CheckboxGroup<T> {
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
            variant: CheckboxVariant::default(),
            paint: Paint::accent(),
            size: CheckboxSize::default(),
            radius: Radius::default(),
            builder: control_group::new(id).multiple().independent_toggles(true),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: CheckboxVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }
    pub fn selected_ids<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.builder = self.builder.selected_ids(ids);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> control_group::ControlGroup<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            self.builder
                .stylesheet(&look.common_stylesheet())
                .motion_defaults(look.motion_defaults().control_group)
                .template(checkbox_group_template(&look, self.variant, self.paint, self.size, self.radius))
                .spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}
