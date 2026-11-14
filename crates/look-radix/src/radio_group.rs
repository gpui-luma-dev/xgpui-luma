//! Standard Radix radio groups backed by SDK selection and keyboard navigation.

use std::sync::Arc;
use gpui::{Context, SharedString, div, px, prelude::*};
use xgpui_luma::controls::button::ButtonRenderModel;
use xgpui_luma::controls::radio_button::RadioButtonData;
use xgpui_luma::controls::control_group::{ControlGroupItem, ControlGroupItemLike, ControlGroupItemHandlerExt};
use xgpui_luma::controls::radio_group::{
    RadioGroupBuilder, RadioGroupLayout, RadioGroupTemplate, radio_group_button_item_element_template,
};
use crate::{Look, Paint, RadioSize, RadioVariant, radio_template_for};
use crate::look::resolve_look;

/// Standard (non-segmented) radio template.
pub fn radio_group_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
) -> RadioGroupTemplate<T> {
    let template = radio_template_for(look, variant, paint, size);
    Arc::new(move |model, handlers, window, cx| {
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let label = item.item.label().clone();
                let button = ButtonRenderModel {
                    id: format!("{}-{}", item.group_id, item.item.id()).into(),
                    data: RadioButtonData::new(item.selected),
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
        let key = match layout {
            RadioGroupLayout::Vertical => "vertical",
            RadioGroupLayout::Horizontal => "horizontal",
        };
        let recipe = model.stylesheet.radio_group.authored_geometry(key);
        div()
            .id(model.id.clone())
            .flex()
            .gap(px(recipe.gap.unwrap_or_default()))
            .items_start()
            .when(layout == RadioGroupLayout::Vertical, |group| group.flex_col())
            .children(rows)
    })
}

/// Look-owned single-selection group. Events and state are owned by the SDK.
pub struct RadioGroup<T: ControlGroupItemLike + 'static = ControlGroupItem> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::radio_group::RadioGroupEvent>,

    look: Option<Look>,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
    builder: RadioGroupBuilder<T>,
}

impl<T: ControlGroupItemLike + 'static> RadioGroup<T> {
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
        callback: impl Fn(&mut M, &xgpui_luma::controls::radio_group::RadioGroupEvent, &mut gpui::Context<M>) + 'static,
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
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::Change { value } = event {
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
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::Activate { value } = event {
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
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::FocusChanged { focused } = event {
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
            if let xgpui_luma::controls::radio_group::RadioGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            variant: RadioVariant::default(),
            paint: Paint::accent(),
            size: RadioSize::default(),
            layout: RadioGroupLayout::Vertical,
            builder: xgpui_luma::controls::radio_group::new(id).selection_follows_active(true),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: RadioVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.layout = RadioGroupLayout::Horizontal;
        self.builder = self.builder.horizontal();
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(id);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::radio_group::RadioGroup<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            let item = radio_group_button_item_element_template(radio_template_for(
                &look,
                self.variant,
                self.paint,
                self.size,
            ));
            self.builder
                .stylesheet(&look.common_stylesheet())
                .motion_defaults(look.motion_defaults().control_group)
                .item_element_template(item)
                .template(radio_group_template(&look, self.variant, self.paint, self.size, self.layout))
                .spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_radio_group_stylesheet() -> xgpui_luma::theme::stylesheet::RadioGroupStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    RadioGroupStylesheet {
        sizes: xgpui_luma::theme::ControlSizes::from([(
            "horizontal".into(),
            RadioGroupGeometryStylesheet { gap: Some(8.0), ..Default::default() },
        )]),
        ..Default::default()
    }
}
