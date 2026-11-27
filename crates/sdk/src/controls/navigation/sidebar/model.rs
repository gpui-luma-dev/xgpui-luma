crate::motion_profile!(SidebarMotion => sidebar {
    expansion: crate::motion::Transition,
    drop_layout: crate::motion::Transition,
    popup: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};
use lucide_svg_static::Icon as LucideIcon;

use super::control::SidebarControl;
use super::engine::{NavNode, SidebarPanelTemplate, default_sidebar_panel_template, modified_sidebar_panel_template};
use super::theme::SidebarPresentation;
use crate::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use crate::infra::icon::DisclosureIcons;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};

#[derive(Clone)]
pub struct SidebarMenuItemModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) tooltip: Option<crate::controls::tooltip::Tooltip>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) badge: Option<SharedString>,
    pub(crate) action_label: Option<SharedString>,
    pub(crate) active: bool,
    pub(crate) disabled: bool,
    pub(crate) draggable: bool,
    pub(crate) branch: bool,
    pub(crate) expanded: Option<bool>,
    pub(crate) children: Vec<SidebarMenuItemModel>,
}

#[derive(Clone)]
pub struct SidebarMenuModel {
    #[allow(dead_code)]
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SidebarMenuItemModel>,
}

#[derive(Clone)]
pub struct SidebarGroupModel {
    pub(crate) id: Option<SharedString>,
    pub(crate) label: Option<SharedString>,
    pub(crate) action_label: Option<SharedString>,
    pub(crate) menu: Option<SidebarMenuModel>,
}

#[derive(Clone)]
pub struct SidebarHeaderModel {
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
}

#[derive(Clone)]
pub struct SidebarContentModel {
    pub(crate) groups: Vec<SidebarGroupModel>,
}

#[derive(Clone)]
pub struct SidebarFooterModel {
    pub(crate) items: Vec<SidebarMenuItemModel>,
}

#[derive(Clone)]
pub struct SidebarPanelModel {
    pub(crate) id: SharedString,
    pub(crate) header: Option<SidebarHeaderModel>,
    pub(crate) content: Option<SidebarContentModel>,
    pub(crate) footer: Option<SidebarFooterModel>,
    pub(crate) disclosure_icons_explicit: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
}

#[derive(Clone)]
pub struct SidebarControlModel {
    pub(crate) motion: crate::motion::MotionConfig<SidebarMotion>,
    pub(crate) id: SharedString,
    pub(crate) presentation: SidebarPresentation,
    pub(crate) enabled: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) rail_dismiss: crate::infra::popup_dismiss::PopupDismiss,
    pub(crate) selected_id: Option<SharedString>,
    pub(crate) drag_drop: Option<super::SidebarDragDrop>,
    pub(crate) sidebar: Option<SidebarPanelModel>,
    pub(crate) default_disclosure_icons: DisclosureIcons,
    pub(crate) panel_template: Arc<dyn SidebarPanelTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) scrollbar_placement: ScrollbarPlacement,
    pub(crate) scrollbar_visibility: ScrollbarVisibility,
    pub(crate) scrollbar_auto_hide_activate: ScrollbarAutoHideActivate,
}

impl SidebarControlModel {
    pub(crate) fn resolved_disclosure_icons(&self) -> DisclosureIcons {
        self.sidebar
            .as_ref()
            .filter(|panel| panel.disclosure_icons_explicit)
            .map(|panel| panel.disclosure_icons.clone())
            .unwrap_or_else(|| self.default_disclosure_icons.clone())
    }
}

pub struct SidebarMenuItemBuilder {
    model: SidebarMenuItemModel,
}

impl SidebarMenuItemBuilder {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            model: SidebarMenuItemModel {
                id: id.clone(),
                label: label.into(),
                tooltip: None,
                icon: None,
                badge: None,
                action_label: None,
                active: false,
                disabled: false,
                draggable: true,
                branch: false,
                expanded: None,
                children: Vec::new(),
            },
        }
    }

    /// Attach help to this navigation item in either sidebar presentation.
    pub fn help(self, text: impl Into<SharedString>) -> Self {
        self.tooltip(crate::controls::tooltip::Tooltip::new(text))
    }

    /// Configure per-item help; delay and dismissal use the shared SDK behavior.
    pub fn tooltip(mut self, tooltip: crate::controls::tooltip::Tooltip) -> Self {
        self.model.tooltip = Some(tooltip);
        self
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.model.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.model.badge = Some(badge.into());
        self
    }

    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.model.action_label = Some(label.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.model.active = active;
        self
    }

    /// Keep a navigation item fixed while other items can be dragged.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.model.draggable = draggable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.model.disabled = disabled;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.model.expanded = Some(expanded);
        self
    }

    /// Attach a nested submenu branch (Shadcn `SidebarMenuSub` ergonomics).
    #[allow(clippy::should_implement_trait)]
    pub fn sub(mut self, sub: SidebarMenuSubBuilder) -> Self {
        self.model.branch = true;
        self.model.children = sub.into_items();
        self
    }

    pub(crate) fn build(self) -> SidebarMenuItemModel {
        self.model
    }
}

pub struct SidebarMenuSubBuilder {
    items: Vec<SidebarMenuItemModel>,
}

impl SidebarMenuSubBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.items.push(item.build());
        self
    }

    pub(crate) fn into_items(self) -> Vec<SidebarMenuItemModel> {
        self.items
    }
}

impl Default for SidebarMenuSubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarMenuBuilder {
    model: SidebarMenuModel,
}

impl SidebarMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { model: SidebarMenuModel { id: id.into(), items: Vec::new() } }
    }

    pub fn item(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.model.items.push(item.build());
        self
    }

    pub(crate) fn build(self) -> SidebarMenuModel {
        self.model
    }
}

pub struct SidebarGroupBuilder {
    model: SidebarGroupModel,
}

impl SidebarGroupBuilder {
    /// Stable destination identity. Defaults to the menu ID when omitted.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.model.id = Some(id.into());
        self
    }
    pub fn new() -> Self {
        Self { model: SidebarGroupModel { id: None, label: None, action_label: None, menu: None } }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = Some(label.into());
        self
    }

    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.model.action_label = Some(label.into());
        self
    }

    pub fn menu(mut self, menu: SidebarMenuBuilder) -> Self {
        self.model.menu = Some(menu.build());
        self
    }

    pub(crate) fn build(self) -> SidebarGroupModel {
        self.model
    }
}

impl Default for SidebarGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarHeaderBuilder {
    model: SidebarHeaderModel,
}

impl SidebarHeaderBuilder {
    pub fn new() -> Self {
        Self { model: SidebarHeaderModel { title: None, subtitle: None } }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = Some(title.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.model.subtitle = Some(subtitle.into());
        self
    }

    pub(crate) fn build(self) -> SidebarHeaderModel {
        self.model
    }
}

impl Default for SidebarHeaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarContentBuilder {
    model: SidebarContentModel,
}

impl SidebarContentBuilder {
    pub fn new() -> Self {
        Self { model: SidebarContentModel { groups: Vec::new() } }
    }

    pub fn group(mut self, group: SidebarGroupBuilder) -> Self {
        self.model.groups.push(group.build());
        self
    }

    pub(crate) fn build(self) -> SidebarContentModel {
        self.model
    }
}

impl Default for SidebarContentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarFooterBuilder {
    model: SidebarFooterModel,
}

impl SidebarFooterBuilder {
    pub fn new() -> Self {
        Self { model: SidebarFooterModel { items: Vec::new() } }
    }

    pub fn child(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.model.items.push(item.build());
        self
    }

    pub(crate) fn build(self) -> SidebarFooterModel {
        self.model
    }
}

impl Default for SidebarFooterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarBuilder {
    model: SidebarPanelModel,
}

impl SidebarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SidebarPanelModel {
                id: id.into(),
                header: None,
                content: None,
                footer: None,
                disclosure_icons_explicit: false,
                disclosure_icons: DisclosureIcons::default(),
            },
        }
    }

    pub fn header(mut self, header: SidebarHeaderBuilder) -> Self {
        self.model.header = Some(header.build());
        self
    }

    pub fn content(mut self, content: SidebarContentBuilder) -> Self {
        self.model.content = Some(content.build());
        self
    }

    pub fn footer(mut self, footer: SidebarFooterBuilder) -> Self {
        self.model.footer = Some(footer.build());
        self
    }

    /// Capture a panel-local disclosure pair that takes precedence over host defaults.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.disclosure_icons_explicit {
            self.model.disclosure_icons = DisclosureIcons::from_stylesheet(stylesheet);
            self.model.disclosure_icons_explicit = true;
        }
        self
    }

    /// Set explicit expanded and collapsed glyphs for nested menu rows.
    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons_explicit = true;
        self.model.disclosure_icons = icons;
        self
    }

    pub fn build(self) -> SidebarPanelModel {
        self.model
    }
}

pub struct SidebarControlBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::sidebar::SidebarEvent>,

    pub(crate) model: SidebarControlModel,
}

impl SidebarControlBuilder {
    crate::motion_methods!(SidebarMotion, model.motion);

    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::DragDrop(event) = event {
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
            if let crate::controls::sidebar::SidebarDragEvent::DragStarted { source, keys } = event {
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
            if let crate::controls::sidebar::SidebarDragEvent::ItemsRemoved { source, target, keys } = event {
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
            &crate::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::ItemsAdded { source, target, keys, location } = event {
                callback(owner, *source, *target, keys, location, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::ItemsReordered { target, keys, location } = event {
                callback(owner, *target, keys, location, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::Dropped { source, target, keys, location, changed } =
                event
            {
                callback(owner, *source, *target, keys, location, *changed, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::DropRejected { source, target, keys, location, error } =
                event
            {
                callback(owner, *source, *target, keys, location, error, cx);
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
            if let crate::controls::sidebar::SidebarDragEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SidebarEvent::PresentationChanged` through the existing event stream.
    pub fn on_presentation_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarPresentation, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::PresentationChanged { presentation } = event {
                callback(owner, presentation, cx);
            }
        })
    }

    /// Handle `SidebarEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::Select { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::ItemFocused { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::HoverChanged { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::SubMenuToggle` through the existing event stream.
    pub fn on_sub_menu_toggle<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::SubMenuToggle { id, open } = event {
                callback(owner, id, *open, cx);
            }
        })
    }

    /// Handle `SidebarEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: SidebarControlModel {
                motion: Default::default(),
                id: id.into(),
                presentation: SidebarPresentation::Expanded,
                enabled: true,
                motion_enabled: true,
                rail_dismiss: crate::infra::popup_dismiss::PopupDismiss::CLICK_OUTSIDE,
                selected_id: None,
                drag_drop: None,
                sidebar: None,
                default_disclosure_icons: DisclosureIcons::default(),
                panel_template: default_sidebar_panel_template(),
                scrollbar_template: default_scrollbar_template(),
                scrollbar_placement: ScrollbarPlacement::Inset,
                scrollbar_visibility: ScrollbarVisibility::AlwaysVisible,
                scrollbar_auto_hide_activate: ScrollbarAutoHideActivate::HoverOrMove,
            },
        }
    }

    /// Select expanded rows or an icon rail, independently of host visibility.
    /// Enable validated same-sidebar moves for the configured groups.
    pub fn drag_drop(mut self, config: super::SidebarDragDrop) -> Self {
        self.model.drag_drop = Some(config);
        self
    }

    pub fn presentation(mut self, presentation: SidebarPresentation) -> Self {
        self.model.presentation = presentation;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// What closes the icon rail's flyout menu; an outside click or Escape by default.
    pub fn rail_dismiss(mut self, dismiss: crate::infra::popup_dismiss::PopupDismiss) -> Self {
        self.model.rail_dismiss = dismiss;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_id = Some(selected_id.into());
        self
    }

    /// Capture defaults for current and subsequently supplied sidebar content.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        self.model.default_disclosure_icons = DisclosureIcons::from_stylesheet(stylesheet);
        self
    }

    pub fn sidebar(mut self, sidebar: SidebarBuilder) -> Self {
        self.model.sidebar = Some(sidebar.build());
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SidebarPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.panel_template = modified_sidebar_panel_template(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn scrollbar_placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.model.scrollbar_placement = placement;
        self
    }

    pub fn overlay_scrollbar(mut self, overlay: bool) -> Self {
        self.model.scrollbar_placement = if overlay {
            ScrollbarPlacement::Overlay
        } else {
            ScrollbarPlacement::Inset
        };
        self
    }

    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    pub fn auto_hide_scrollbar(mut self, auto_hide: bool) -> Self {
        self.model.scrollbar_visibility = if auto_hide {
            ScrollbarVisibility::AutoHide
        } else {
            ScrollbarVisibility::AlwaysVisible
        };
        self
    }

    pub fn auto_hide_scrollbar_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.model.scrollbar_auto_hide_activate = activate;
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SidebarControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| SidebarControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

pub(crate) fn find_active_id(panel: &SidebarPanelModel) -> Option<SharedString> {
    if let Some(content) = &panel.content {
        for group in &content.groups {
            if let Some(menu) = &group.menu
                && let Some(id) = find_active_in_items(&menu.items)
            {
                return Some(id);
            }
        }
    }
    if let Some(footer) = &panel.footer
        && let Some(id) = find_active_in_items(&footer.items)
    {
        return Some(id);
    }
    None
}

fn find_active_in_items(items: &[SidebarMenuItemModel]) -> Option<SharedString> {
    for item in items {
        if item.active {
            return Some(item.id.clone());
        }
        if let Some(id) = find_active_in_items(&item.children) {
            return Some(id);
        }
    }
    None
}

pub(crate) fn panel_to_nav_nodes(panel: &SidebarPanelModel) -> (Vec<NavNode>, Vec<NavNode>) {
    let mut main_nodes = Vec::new();
    if let Some(content) = &panel.content {
        for (group_index, group) in content.groups.iter().enumerate() {
            if let Some(label) = &group.label {
                let section_id =
                    group.id.clone().unwrap_or_else(|| SharedString::from(format!("{}-group-{group_index}", panel.id)));
                main_nodes.push(NavNode::section(section_id, label.clone()));
            }
            if let Some(menu) = &group.menu {
                main_nodes.extend(menu.items.iter().map(menu_item_to_nav_node));
            }
        }
    }

    let footer_nodes = panel
        .footer
        .as_ref()
        .map(|footer| footer.items.iter().map(menu_item_to_nav_node).collect())
        .unwrap_or_default();

    (main_nodes, footer_nodes)
}

fn menu_item_to_nav_node(item: &SidebarMenuItemModel) -> NavNode {
    let mut node = NavNode::new(item.id.clone()).label(item.label.clone()).enabled(!item.disabled);

    node.branch = item.branch;
    node.tooltip = item.tooltip.clone();
    if let Some(icon) = item.icon {
        node = node.icon(icon);
    }

    if item.branch || !item.children.is_empty() {
        let child_active = find_active_in_items(&item.children).is_some();
        let expanded = item.expanded.unwrap_or(child_active || item.active);
        node = node.expanded(expanded).children(item.children.iter().map(menu_item_to_nav_node));
    }

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::engine::NavNodeKind;

    #[test]
    fn menu_tree_maps_to_nav_nodes_with_sections_and_selection() {
        let panel = SidebarBuilder::new("workbench")
            .content(
                SidebarContentBuilder::new()
                    .group(
                        SidebarGroupBuilder::new().label("Pinned").menu(
                            SidebarMenuBuilder::new("pinned")
                                .item(SidebarMenuItemBuilder::new("summary", "Summary").icon(LucideIcon::Info)),
                        ),
                    )
                    .group(
                        SidebarGroupBuilder::new().label("Properties").menu(
                            SidebarMenuBuilder::new("properties").item(
                                SidebarMenuItemBuilder::new("layout", "Layout").icon(LucideIcon::Ruler).sub(
                                    SidebarMenuSubBuilder::new()
                                        .item(SidebarMenuItemBuilder::new("position", "Position"))
                                        .item(SidebarMenuItemBuilder::new("dimensions", "Dimensions").active(true)),
                                ),
                            ),
                        ),
                    ),
            )
            .footer(
                SidebarFooterBuilder::new()
                    .child(SidebarMenuItemBuilder::new("audit", "Audit Log").icon(LucideIcon::FileText)),
            )
            .build();

        assert_eq!(find_active_id(&panel).as_deref(), Some("dimensions"));

        let (nodes, footer) = panel_to_nav_nodes(&panel);
        assert_eq!(nodes[0].id().as_ref(), "workbench-group-0");
        assert!(matches!(nodes[0].kind, NavNodeKind::Section));
        assert_eq!(nodes[1].id().as_ref(), "summary");
        assert_eq!(nodes[3].id().as_ref(), "layout");
        assert!(nodes[3].is_expanded());
        assert_eq!(footer.len(), 1);
        assert_eq!(footer[0].id().as_ref(), "audit");
    }

    #[test]
    fn test_sidebar_builder_animated_option() {
        let builder_default = SidebarControlBuilder::new("sidebar");
        assert!(builder_default.model.motion_enabled);

        let builder_opt_out = SidebarControlBuilder::new("sidebar").motion_enabled(false);
        assert!(!builder_opt_out.model.motion_enabled);
    }
}

#[cfg(test)]
mod stylesheet_icon_tests {
    use super::*;
    use crate::theme::stylesheet::CommonStylesheet;

    #[test]
    fn sidebar_defaults_survive_content_replacement_and_preserve_local_icons() {
        let sheet = CommonStylesheet::parse("[common.icons]\ndisclosure_collapsed='svg:branch.svg'").unwrap();
        let builder = SidebarControlBuilder::new("sidebar").stylesheet(&sheet).sidebar(SidebarBuilder::new("first"));
        assert_eq!(builder.model.resolved_disclosure_icons().collapsed.svg_path().unwrap().as_ref(), "branch.svg");
        let builder = builder.sidebar(SidebarBuilder::new("replacement"));
        assert_eq!(builder.model.resolved_disclosure_icons().collapsed.svg_path().unwrap().as_ref(), "branch.svg");
        let explicit = DisclosureIcons::default();
        let builder = builder
            .sidebar(SidebarBuilder::new("explicit").disclosure_icons(explicit.clone()))
            .stylesheet(&sheet);
        assert_eq!(builder.model.resolved_disclosure_icons().collapsed, explicit.collapsed);
    }
}
