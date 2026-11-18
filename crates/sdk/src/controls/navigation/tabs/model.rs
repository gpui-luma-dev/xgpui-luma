use std::sync::Arc;
use std::collections::HashMap;

use gpui::{App, AppContext, Entity, IntoElement, Render, SharedString, Window};

use super::indicator::{TabsIndicatorMotion, TabsIndicatorPaint};
use super::template::{tabs_control_group_template, template_with_modifier};
use super::{TabsContentLayout, TabsContentTransition, Tabs, TabsContent, TabsTemplate, default_tabs_template};
use crate::controls::control_group::ControlGroupItemLike;
use crate::infra::icon::IconSource;
use crate::infra::icon::DisclosureIcons;
use crate::controls::tabs::{ControlFocusState, TabsItemState};

crate::motion_profile!(TabsMotion => tabs {
    drop_layout: crate::motion::Transition,
    indicator: crate::motion::Transition,
    content: crate::motion::Transition,
    disclosure: crate::motion::Transition,
});

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsWidthMode {
    #[default]
    Intrinsic,
    Uniform,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsTriggerKind {
    #[default]
    Select,
    Dropdown,
}

#[derive(Clone, Debug)]
pub enum TabsItemAccessory {
    Icon(IconSource),
    Disclosure { open: bool },
}

impl TabsItemAccessory {
    pub fn icon(icon: impl Into<IconSource>) -> Self {
        Self::Icon(icon.into())
    }

    pub fn disclosure(open: bool) -> Self {
        Self::Disclosure { open }
    }

    pub fn is_disclosure_open(&self) -> Option<bool> {
        match self {
            Self::Disclosure { open } => Some(*open),
            Self::Icon(_) => None,
        }
    }

    pub(crate) fn set_disclosure_open(&mut self, open: bool) -> bool {
        let Self::Disclosure { open: current } = self else {
            return false;
        };
        if *current == open {
            return false;
        }
        *current = open;
        true
    }
}

#[derive(Clone, Debug)]
pub struct TabsItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
    pub(crate) trigger_kind: TabsTriggerKind,
    pub(crate) leading_accessory: Option<TabsItemAccessory>,
    pub(crate) trailing_accessory: Option<TabsItemAccessory>,
    pub(crate) disclosure_progress: f32,
}

impl TabsItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            label: id.clone(),
            id,
            enabled: true,
            trigger_kind: TabsTriggerKind::Select,
            leading_accessory: None,
            trailing_accessory: None,
            disclosure_progress: 0.0,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn trigger_kind(mut self, trigger_kind: TabsTriggerKind) -> Self {
        self.trigger_kind = trigger_kind;
        self
    }

    pub fn dropdown_trigger(mut self) -> Self {
        self.trigger_kind = TabsTriggerKind::Dropdown;
        if self.trailing_accessory.is_none() {
            self.trailing_accessory = Some(TabsItemAccessory::disclosure(false));
        }
        self.disclosure_progress = 0.0;
        self
    }

    pub fn leading_accessory(mut self, accessory: TabsItemAccessory) -> Self {
        self.leading_accessory = Some(accessory);
        self
    }

    pub fn trailing_accessory(mut self, accessory: TabsItemAccessory) -> Self {
        self.trailing_accessory = Some(accessory);
        self
    }

    pub fn leading_icon(self, icon: impl Into<IconSource>) -> Self {
        self.leading_accessory(TabsItemAccessory::icon(icon))
    }

    pub fn trailing_icon(self, icon: impl Into<IconSource>) -> Self {
        self.trailing_accessory(TabsItemAccessory::icon(icon))
    }

    pub fn trailing_disclosure(self, open: bool) -> Self {
        let mut item = self;
        item.trailing_accessory = Some(TabsItemAccessory::disclosure(open));
        item.disclosure_progress = if open { 1.0 } else { 0.0 };
        item
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn trigger_kind_value(&self) -> TabsTriggerKind {
        self.trigger_kind
    }

    pub fn is_dropdown_trigger(&self) -> bool {
        self.trigger_kind == TabsTriggerKind::Dropdown
    }

    pub fn leading_accessory_ref(&self) -> Option<&TabsItemAccessory> {
        self.leading_accessory.as_ref()
    }

    pub fn trailing_accessory_ref(&self) -> Option<&TabsItemAccessory> {
        self.trailing_accessory.as_ref()
    }

    pub fn disclosure_progress(&self) -> f32 {
        self.disclosure_progress
    }
}

impl ControlGroupItemLike for TabsItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct TabsModel {
    pub(crate) reordering: bool,
    pub(crate) id: SharedString,
    pub(crate) width_mode: TabsWidthMode,
    pub(crate) items: Vec<TabsItem>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) content_transition: Option<TabsContentTransition>,
    pub(crate) content_layout: TabsContentLayout,
    pub(crate) contents: HashMap<SharedString, TabsContent>,
    pub(crate) motion: crate::motion::MotionConfig<TabsMotion>,
    pub(crate) disclosure_icons_explicit: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn TabsTemplate>,
}

pub struct TabsRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub trigger_kind: TabsTriggerKind,
    pub leading_accessory: Option<&'a TabsItemAccessory>,
    pub trailing_accessory: Option<&'a TabsItemAccessory>,
    pub active: bool,
    pub enabled: bool,
    pub state: TabsItemState,
    pub disclosure_progress: f32,
}

pub struct TabsRenderModel<'a> {
    /// Native SDK binding used by themed/custom tab templates.
    pub drag_drop: Option<gpui::WeakEntity<super::Tabs>>,
    pub id: &'a SharedString,
    pub width_mode: TabsWidthMode,
    pub items: Vec<TabsRenderItem<'a>>,
    pub active_id: Option<&'a SharedString>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub indicator: Option<TabsIndicatorPaint>,
    pub indicator_motion: Option<&'a TabsIndicatorMotion>,
    pub disclosure_icons: &'a DisclosureIcons,
}

pub struct TabsBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::tabs::TabsEvent>,

    pub(crate) model: TabsModel,
}

impl TabsBuilder {
    crate::motion_methods!(TabsMotion, model.motion);

    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tabs::TabsDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::DragDrop(event) = event {
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

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tabs::TabsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `TabsEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Change { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Activate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Reactivate` through the existing event stream.
    pub fn on_reactivate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Reactivate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::DropdownRequested` through the existing event stream.
    pub fn on_dropdown_requested<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            &gpui::SharedString,
            &gpui::SharedString,
            &Option<gpui::Bounds<gpui::Pixels>>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::DropdownRequested { tab_id, label, bounds } = event {
                callback(owner, tab_id, label, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemBoundsChanged` through the existing event stream.
    pub fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::ItemBoundsChanged { tab_id, bounds } = event {
                callback(owner, tab_id, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::ItemFocused { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: TabsModel {
                reordering: false,
                id: id.into(),
                width_mode: TabsWidthMode::Intrinsic,
                items: Vec::new(),
                active_id: None,
                enabled: true,
                motion_enabled: true,
                content_transition: None,
                content_layout: TabsContentLayout::Auto,
                contents: HashMap::new(),
                motion: Default::default(),
                disclosure_icons_explicit: false,
                disclosure_icons: DisclosureIcons::menu_from_stylesheet(
                    crate::theme::stylesheet::CommonStylesheet::sdk_default(),
                ),
                template: default_tabs_template(),
            },
        }
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self.model.contents.retain(|id, _| self.model.items.iter().any(|item| item.id() == id));
        self
    }

    /// Add a tab backed by an existing view. Its entity survives selection changes.
    pub fn tab<V: Render + 'static>(
        self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        view: Entity<V>,
    ) -> Self {
        self.tab_with(id, label, move |_, _| view.clone())
    }

    /// Add a tab with a content presenter. The callback runs only for the selected panel.
    pub fn tab_with<F, E>(self, id: impl Into<SharedString>, label: impl Into<SharedString>, presenter: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        self.tab_content(TabsItem::new(id).label(label), TabsContent::new(presenter))
    }

    /// Add an item and its content, preserving item options such as disabled state.
    pub fn tab_content(mut self, item: TabsItem, content: TabsContent) -> Self {
        self.model.contents.insert(item.id().clone(), content);
        self.model.items.push(item);
        self
    }

    /// Explicit content sizing, independent of the selected effect.
    pub fn content_layout(mut self, layout: TabsContentLayout) -> Self {
        self.model.content_layout = layout;
        self
    }

    /// Choose the content transition; timing comes from `TabsMotion::content`.
    pub fn content_transition(mut self, transition: TabsContentTransition) -> Self {
        self.model.content_transition = Some(transition);
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.model.active_id = Some(active_id.into());
        self
    }

    /// Opt into same-strip keyed reordering; content and active identity survive moves.
    pub fn reordering(mut self, enabled: bool) -> Self {
        self.model.reordering = enabled;
        self
    }

    pub fn width_mode(mut self, width_mode: TabsWidthMode) -> Self {
        self.model.width_mode = width_mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    /// Capture shared icon defaults without replacing explicit control icons.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.disclosure_icons_explicit {
            self.model.disclosure_icons = DisclosureIcons::menu_from_stylesheet(stylesheet);
        }
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons_explicit = true;
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn TabsTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TabsRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<Tabs> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let tabs = cx.new(|cx| Tabs::from_builder(self, cx));
            tabs.update(cx, |tabs, cx| tabs.initialize_slots(cx));
            tabs
        };
        callbacks.bind(&control, cx);
        control
    }

    pub(crate) fn control_group_template(
        &self,
        indicator_motion: TabsIndicatorMotion,
        disclosure_progress: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
        owner: gpui::WeakEntity<super::Tabs>,
    ) -> crate::controls::control_group::ControlGroupTemplate<TabsItem> {
        tabs_control_group_template(
            self.model.width_mode,
            Arc::clone(&self.model.template),
            indicator_motion,
            self.model.disclosure_icons.clone(),
            disclosure_progress,
            owner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn sdk_role_edits_apply_after_defaults_regardless_of_builder_order() {
        use crate::motion::{Easing, MotionState, Transition};
        let defaults = TabsMotion {
            indicator: Transition::timed(Duration::from_secs(1), Easing::Linear),
            content: Transition::timed(Duration::from_millis(500), Easing::Linear),
            ..TabsMotion::default()
        };
        let edit = |m: &mut TabsMotion| m.indicator = Transition::instant();
        for builder in [
            TabsBuilder::new("before").motion(edit).motion_defaults(defaults.clone()),
            TabsBuilder::new("after").motion_defaults(defaults).motion(edit),
        ] {
            let profile = builder.motion_profile();
            assert!(profile.indicator.start(MotionState::default(), 1.0).sample(Duration::ZERO).complete);
            assert_eq!(
                profile.content.start(MotionState::default(), 1.0).sample(Duration::from_millis(100)).state.value,
                0.2
            );
        }
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_tabs_template();
        let builder = TabsBuilder::new("tabs-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn item_accessories_are_configurable() {
        let item = TabsItem::new("controls").leading_icon(lucide_svg_static::Icon::Settings).trailing_disclosure(true);

        assert!(item.leading_accessory_ref().is_some());
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(true));
    }

    #[test]
    fn dropdown_trigger_sets_trigger_kind_and_default_disclosure() {
        let item = TabsItem::new("controls").label("Controls").dropdown_trigger();

        assert!(item.is_dropdown_trigger());
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(false));
    }
}
