crate::motion_profile!(AccordionMotion => accordion {
    expansion: crate::motion::Transition,
    drop_layout: crate::motion::Transition,
});

use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, IntoElement, SharedString, Stateful, Window};
use lucide_svg_static::Icon as LucideIcon;

use super::{AccordionControl, AccordionTemplate, default_accordion_template};
use super::template::modified_accordion_template;
use crate::infra::icon::DisclosureIcons;

pub type AccordionElementRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccordionSelectionMode {
    /// Only one item can be expanded at a time. Expanding another collapses the active one.
    #[default]
    Single,
    /// Multiple items can be expanded simultaneously.
    Multiple,
}

/// Represents the header/trigger area of an accordion item.
#[derive(Clone)]
pub struct AccordionTrigger {
    pub(crate) label: Option<SharedString>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) custom_element: Option<AccordionElementRenderer>,
}

impl AccordionTrigger {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: Some(label.into()), icon: None, custom_element: None }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn custom(custom: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        Self { label: None, icon: None, custom_element: Some(Arc::new(custom)) }
    }
}

/// Represents the collapsible panel content.
#[derive(Clone)]
pub struct AccordionContent {
    pub(crate) element: Option<AccordionElementRenderer>,
}

impl AccordionContent {
    pub fn new(element: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        Self { element: Some(Arc::new(move |_, _| element.clone().into_any_element())) }
    }

    pub fn custom(custom: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        Self { element: Some(Arc::new(custom)) }
    }
}

/// Represents an item configuration in the accordion.
#[derive(Clone)]
pub struct AccordionItem {
    pub(crate) id: SharedString,
    pub(crate) trigger: AccordionTrigger,
    pub(crate) content: AccordionContent,
    pub(crate) enabled: bool,
    pub(crate) initially_expanded: bool,
}

impl AccordionItem {
    pub fn new(id: impl Into<SharedString>, trigger: AccordionTrigger, content: AccordionContent) -> Self {
        Self { id: id.into(), trigger, content, enabled: true, initially_expanded: false }
    }

    /// Stable identity retained through reordering.
    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }
}

#[derive(Clone)]
pub struct AccordionModel {
    pub(crate) motion: crate::motion::MotionConfig<AccordionMotion>,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<AccordionItem>,
    pub(crate) selection_mode: AccordionSelectionMode,
    pub(crate) collapsible: bool,
    pub(crate) enabled: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) reordering: bool,
    pub(crate) item_dividers: bool,
    /// When set, overrides themed vertical padding inside expanded item panels (both edges).
    pub(crate) content_padding_y: Option<f32>,
    /// When set, overrides top padding inside expanded item panels.
    pub(crate) content_padding_top: Option<f32>,
    /// When set, overrides bottom padding inside expanded item panels.
    pub(crate) content_padding_bottom: Option<f32>,
    /// When set, overrides themed minimum height of item triggers.
    pub(crate) trigger_min_height: Option<f32>,
    /// When set, overrides themed vertical padding on item triggers.
    pub(crate) trigger_padding_y: Option<f32>,
    pub(crate) disclosure_icons_explicit: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn AccordionTemplate>,
}

pub struct AccordionItemRenderModel<'a> {
    pub id: &'a SharedString,
    pub trigger: &'a AccordionTrigger,
    pub content: &'a AccordionContent,
    pub expanded: bool,
    /// Expand/collapse transition progress in range `0.0`..`1.0`.
    pub progress: f32,
    /// Cached natural content height in pixels (0.0 until measured).
    pub content_height_px: f32,
    pub enabled: bool,
    pub state: crate::infra::state::CompositeItemState,
}

pub struct AccordionRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<AccordionItemRenderModel<'a>>,
    pub selection_mode: AccordionSelectionMode,
    pub collapsible: bool,
    pub enabled: bool,
    pub item_dividers: bool,
    pub content_padding_y: Option<f32>,
    pub content_padding_top: Option<f32>,
    pub content_padding_bottom: Option<f32>,
    pub trigger_min_height: Option<f32>,
    pub trigger_padding_y: Option<f32>,
    pub disclosure_icons: &'a DisclosureIcons,
    pub focus: crate::infra::state::ControlFocusState,
}

pub struct AccordionBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::accordion::AccordionEvent>,

    pub(crate) model: AccordionModel,
}

impl AccordionBuilder {
    crate::motion_methods!(AccordionMotion, model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::accordion::AccordionEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::accordion::AccordionDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::DragDrop(event) = event {
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
            if let crate::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event {
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
            if let crate::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } = event
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
            crate::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
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
            if let crate::controls::accordion::AccordionEvent::ExpandedChanged { item_id, expanded } = event {
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
            if let crate::controls::accordion::AccordionEvent::FocusChanged { focused } = event {
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
            if let crate::controls::accordion::AccordionEvent::ItemFocused { item_id } = event {
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
            if let crate::controls::accordion::AccordionEvent::ItemHoverChanged { item_id, hovered } = event {
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
            if let crate::controls::accordion::AccordionEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: AccordionModel {
                motion: Default::default(),
                id: id.into(),
                items: Vec::new(),
                selection_mode: AccordionSelectionMode::Single,
                collapsible: true,
                enabled: true,
                motion_enabled: true,
                reordering: false,
                item_dividers: true,
                content_padding_y: None,
                content_padding_top: None,
                content_padding_bottom: None,
                trigger_min_height: None,
                trigger_padding_y: None,
                disclosure_icons_explicit: false,
                disclosure_icons: DisclosureIcons::default(),
                template: default_accordion_template(),
            },
        }
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    /// Enables or disables expand/collapse animation (default `true`).
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    /// Enable native same-Accordion reordering through header drag handles.
    pub fn reordering(mut self, enabled: bool) -> Self {
        self.model.reordering = enabled;
        self
    }

    pub fn single(self) -> Self {
        self.mode(AccordionSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.mode(AccordionSelectionMode::Multiple)
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.model.collapsible = collapsible;
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn item_dividers(mut self, item_dividers: bool) -> Self {
        self.model.item_dividers = item_dividers;
        self
    }

    pub fn content_padding_y(mut self, padding_y: f32) -> Self {
        self.model.content_padding_y = Some(padding_y);
        self
    }

    pub fn content_padding_top(mut self, padding_top: f32) -> Self {
        self.model.content_padding_top = Some(padding_top);
        self
    }

    pub fn content_padding_bottom(mut self, padding_bottom: f32) -> Self {
        self.model.content_padding_bottom = Some(padding_bottom);
        self
    }

    pub fn trigger_min_height(mut self, height: f32) -> Self {
        self.model.trigger_min_height = Some(height);
        self
    }

    pub fn trigger_padding_y(mut self, padding_y: f32) -> Self {
        self.model.trigger_padding_y = Some(padding_y);
        self
    }

    /// Sets the expanded and collapsed disclosure icons used by item triggers.
    /// Capture shared disclosure defaults; explicit glyphs win in either method order.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.disclosure_icons_explicit {
            self.model.disclosure_icons = DisclosureIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons_explicit = true;
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = modified_accordion_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<AccordionControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| AccordionControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::AccordionBuilder;

    #[test]
    fn test_accordion_builder_animated_option() {
        let builder_default = AccordionBuilder::new("accordion");
        assert!(builder_default.model.motion_enabled);

        let builder_opt_out = AccordionBuilder::new("accordion").motion_enabled(false);
        assert!(!builder_opt_out.model.motion_enabled);
    }
}
