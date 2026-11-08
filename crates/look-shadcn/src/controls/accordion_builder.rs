//! Look-owned accordion builder. Spawn synthesizes the SDK [`xgpui_luma::controls::accordion::Accordion`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::accordion::{
    AccordionBuilder, AccordionControl, AccordionItem, AccordionSelectionMode, AccordionTemplate,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of an accordion: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Accordion {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::accordion::AccordionEvent>,

    look: Option<ShadcnLook>,
    builder: AccordionBuilder,
    custom_template: bool,
    size: ShadcnSize,
}

impl Accordion {
    xgpui_luma::forward_motion!(xgpui_luma::controls::accordion::AccordionMotion, builder builder);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::accordion::AccordionEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::accordion::AccordionDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }
    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } =
                event
            {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } =
                event
            {
                callback(owner, *source, *target, keys, before, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            xgpui_luma::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ExpandedChanged` through the existing event stream.
    pub fn on_expanded_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::ExpandedChanged { item_id, expanded } = event {
                callback(owner, item_id, *expanded, cx);
            }
        })
    }

    /// Handle `AccordionEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ItemHoverChanged` through the existing event stream.
    pub fn on_item_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::ItemHoverChanged { item_id, hovered } = event {
                callback(owner, item_id, *hovered, cx);
            }
        })
    }

    /// Handle `AccordionEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::accordion::AccordionEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            builder: xgpui_luma::controls::accordion::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.builder = self.builder.mode(mode);
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }

    /// Enable native same-Accordion reordering through header drag handles.
    pub fn reordering(mut self, enabled: bool) -> Self {
        self.builder = self.builder.reordering(enabled);
        self
    }

    pub fn single(mut self) -> Self {
        self.builder = self.builder.single();
        self
    }

    pub fn multiple(mut self) -> Self {
        self.builder = self.builder.multiple();
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.builder = self.builder.collapsible(collapsible);
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn item_dividers(mut self, item_dividers: bool) -> Self {
        self.builder = self.builder.item_dividers(item_dividers);
        self
    }

    pub fn content_padding_y(mut self, padding_y: f32) -> Self {
        self.builder = self.builder.content_padding_y(padding_y);
        self
    }

    pub fn content_padding_top(mut self, padding_top: f32) -> Self {
        self.builder = self.builder.content_padding_top(padding_top);
        self
    }

    pub fn content_padding_bottom(mut self, padding_bottom: f32) -> Self {
        self.builder = self.builder.content_padding_bottom(padding_bottom);
        self
    }

    pub fn trigger_min_height(mut self, height: f32) -> Self {
        self.builder = self.builder.trigger_min_height(height);
        self
    }

    pub fn trigger_padding_y(mut self, padding_y: f32) -> Self {
        self.builder = self.builder.trigger_padding_y(padding_y);
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.builder = self.builder.disclosure_icons(icons);
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<AccordionControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> AccordionBuilder {
        let self_builder =
            self.builder.stylesheet(&look.stylesheet().common).motion_defaults(look.motion_defaults().accordion);
        let builder = self_builder.size(self.size.control_size());
        if self.custom_template {
            builder
        } else {
            builder.template(look.accordion_template())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::accordion::{AccordionContent, AccordionTrigger};

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Accordion::new("ok")
            .look(&look)
            .single()
            .item(AccordionItem::new("one", AccordionTrigger::new("One"), AccordionContent::new("Body")))
            .into_sdk_builder(look);
    }
}
