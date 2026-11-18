use std::rc::Rc;

use gpui::{
    Bounds, ClickEvent, Context, FocusHandle, FocusOutEvent, MouseDownEvent, MouseUpEvent, Pixels, SharedString, Window,
};

use super::super::{
    NavNode, NavNodeKind, SidebarPanelEngineRenderModel, SidebarPanelTemplateHandlers, RenderedNavNode,
    RenderedRailSubmenu,
};
use super::nodes::collapsed_rail_node_visible;
use super::{NavigationFocusTarget, SidebarPanelEngine, SidebarPanelEngineEvent};
use crate::focus::EscapeFocus;
use crate::infra::menu_item::MenuItem;
use crate::infra::state::MenuPath;
use crate::key_handling::{
    ActivateControl, CloseSubmenu, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

pub(super) struct CachedSidebarHandlers {
    collapsed: bool,
    signature: Vec<(SharedString, bool)>,
    handlers: SidebarPanelTemplateHandlers,
}

fn clone_row_handlers(handlers: &SidebarPanelTemplateHandlers) -> SidebarPanelTemplateHandlers {
    SidebarPanelTemplateHandlers {
        row_bounds: handlers.row_bounds.clone(),
        row_hovers: handlers.row_hovers.clone(),
        row_mouse_downs: handlers.row_mouse_downs.clone(),
        row_mouse_ups: handlers.row_mouse_ups.clone(),
        row_mouse_up_outs: handlers.row_mouse_up_outs.clone(),
        row_clicks: handlers.row_clicks.clone(),
        children_height_reports: handlers.children_height_reports.clone(),
        // A struct update from Default constructs and drops seven unused Rc
        // containers, even though every row field above is supplied explicitly.
        rail_submenu_mouse_down_out: None,
        rail_submenu_item_hovers: Vec::new(),
        rail_submenu_item_clicks: Vec::new(),
        rail_submenu_bounds: None,
    }
}

impl SidebarPanelEngine {
    pub(super) fn template_handlers(
        &self,
        model: &SidebarPanelEngineRenderModel,
        cx: &mut Context<Self>,
    ) -> SidebarPanelTemplateHandlers {
        let regions: &[&[RenderedNavNode]] = if model.collapsed {
            &[&model.rail_nodes, &model.rail_footer_nodes]
        } else {
            &[&model.header_nodes, &model.nodes, &model.footer_nodes]
        };
        let mut cache = self.row_handlers.borrow_mut();
        let cached = cache
            .as_ref()
            .filter(|cached| cached.collapsed == model.collapsed && signature_matches(regions, &cached.signature));
        let mut handlers = if let Some(cached) = cached {
            clone_row_handlers(&cached.handlers)
        } else {
            let mut signature = Vec::new();
            for region in regions {
                handler_signature(region, &mut signature);
            }
            let row_count = signature.len();
            let branch_count = signature.iter().filter(|(_, branch)| *branch).count();
            let mut handlers = SidebarPanelTemplateHandlers {
                row_bounds: Rc::new(Vec::with_capacity(if model.collapsed { row_count } else { 0 })),
                row_hovers: Rc::new(Vec::with_capacity(row_count)),
                row_mouse_downs: Rc::new(Vec::with_capacity(row_count)),
                row_mouse_ups: Rc::new(Vec::with_capacity(row_count)),
                row_mouse_up_outs: Rc::new(Vec::with_capacity(row_count)),
                row_clicks: Rc::new(Vec::with_capacity(row_count)),
                children_height_reports: Rc::new(std::collections::HashMap::with_capacity(if model.collapsed {
                    0
                } else {
                    branch_count
                })),
                rail_submenu_mouse_down_out: None,
                rail_submenu_item_hovers: Vec::new(),
                rail_submenu_item_clicks: Vec::new(),
                rail_submenu_bounds: None,
            };

            let interaction = if model.collapsed {
                SidebarNodeInteraction::Rail
            } else {
                SidebarNodeInteraction::Default
            };
            for region in regions {
                push_template_handlers(region, &mut handlers, cx, interaction);
            }
            *cache = Some(CachedSidebarHandlers {
                collapsed: model.collapsed,
                signature,
                handlers: clone_row_handlers(&handlers),
            });
            handlers
        };
        drop(cache);

        if model.collapsed
            && let Some(rail_submenu) = &model.rail_submenu
        {
            handlers.rail_submenu_bounds = Some(Rc::new(cx.listener(|this, bounds: &Bounds<Pixels>, _window, cx| {
                if this.rail_submenu_content_size != Some(bounds.size) {
                    this.rail_submenu_content_size = Some(bounds.size);
                    cx.notify();
                }
            })));
            let parent_node_id = rail_submenu.parent_node_id.clone();
            handlers.rail_submenu_mouse_down_out = Some(Rc::new(cx.listener(Self::handle_rail_submenu_mouse_down_out)));
            handlers.rail_submenu_item_hovers = (0..rail_submenu.items.len())
                .map(|index| {
                    let parent_node_id = parent_node_id.clone();
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_rail_submenu_item_hover(parent_node_id.clone(), index, *hovered, cx);
                    })) as _
                })
                .collect();
            handlers.rail_submenu_item_clicks = self
                .rail_submenu_click_node_ids(&parent_node_id)
                .into_iter()
                .map(|node_id| {
                    Box::new(cx.listener(move |this, event, _window, cx| {
                        this.handle_rail_submenu_item_click(node_id.clone(), event, cx);
                    })) as _
                })
                .collect();
        }

        handlers
    }

    pub(super) fn close_rail_submenu(&mut self, cx: &mut Context<Self>) -> bool {
        if self.open_rail_submenu.is_none()
            && self.rail_submenu_open_submenu.is_none()
            && self.rail_submenu_active_path.is_none()
            && !self.rail_submenu_lifecycle.is_open()
            && !self.rail_submenu_lifecycle.presence().is_animating()
        {
            return false;
        }

        self.open_rail_submenu = None;
        self.rail_submenu_open_submenu = None;
        self.rail_submenu_active_path = None;
        self.rail_submenu_lifecycle.close();
        cx.emit(SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id: None });
        true
    }

    pub(super) fn set_open_rail_submenu(&mut self, node_id: SharedString, cx: &mut Context<Self>) {
        if self.open_rail_submenu.as_ref() == Some(&node_id) {
            self.close_rail_submenu(cx);
        } else {
            self.open_rail_submenu = Some(node_id.clone());
            self.rail_submenu_open_submenu = None;
            self.rail_submenu_active_path = None;
            self.rail_submenu_lifecycle.open();
            cx.emit(SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id: Some(node_id) });
        }
    }

    pub(super) fn rendered_rail_submenu(&self) -> Option<RenderedRailSubmenu> {
        let parent_node_id = self.open_rail_submenu.as_ref()?;
        let parent = self.find_top_level_main_node(parent_node_id)?;
        if !parent.has_children() || !collapsed_rail_node_visible(parent) {
            return None;
        }

        let parent_bounds = *self.rail_node_bounds.get(parent_node_id)?;
        let items = parent.children.iter().filter(|node| node.visible).map(nav_node_to_menu_item).collect();

        Some(RenderedRailSubmenu {
            id: format!("{}-rail-submenu-{}", self.model.id, parent_node_id).into(),
            parent_node_id: parent_node_id.clone(),
            parent_bounds,
            items,
            open_submenu: self.rail_submenu_open_submenu,
            active_path: self.rail_submenu_active_path,
            presence: self.rail_submenu_lifecycle.presence(),
            content_size: self.rail_submenu_content_size,
        })
    }

    pub(super) fn rail_submenu_click_node_ids(&self, parent_node_id: &SharedString) -> Vec<SharedString> {
        let Some(parent) = self.find_top_level_main_node(parent_node_id) else {
            return Vec::new();
        };

        nav_menu_click_node_ids(&parent.children, self.rail_submenu_open_submenu)
    }

    pub(super) fn handle_node_hover(&mut self, node_id: SharedString, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_interact_with_default_node(&node_id) {
            return;
        }

        if hovered {
            if self.hovered_node.as_ref() != Some(&node_id) {
                self.set_hovered_node(Some(node_id), cx);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.set_hovered_node(None, cx);
            if self.pressed_node.as_ref() == Some(&node_id) {
                self.pressed_node = None;
            }
            cx.notify();
        }
    }

    pub(super) fn handle_node_mouse_down(
        &mut self,
        node_id: SharedString,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_interact_with_default_node(&node_id) {
            if let Some(focus_handle) = self.row_focus_handles.get(&node_id) {
                focus_handle.focus(window, cx);
            }
            self.pressed_node = Some(node_id);
            cx.notify();
        }
    }

    pub(super) fn handle_node_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_node.is_some() {
            self.pressed_node = None;
            cx.notify();
        }
    }

    pub(super) fn handle_node_click(
        &mut self,
        node_id: SharedString,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() {
            return;
        }

        if !self.can_interact_with_default_node(&node_id) {
            return;
        }

        if self.find_node(&node_id).is_some_and(|node| node.has_children()) {
            self.toggle_node_expanded(node_id, cx);
        } else {
            self.activate_node(node_id, cx);
        }
    }

    pub(super) fn handle_rail_node_hover(&mut self, node_id: SharedString, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_interact_with_rail_node(&node_id) {
            return;
        }

        if hovered {
            if self.hovered_node.as_ref() != Some(&node_id) {
                self.set_hovered_node(Some(node_id), cx);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.set_hovered_node(None, cx);
            if self.pressed_node.as_ref() == Some(&node_id) {
                self.pressed_node = None;
            }
            cx.notify();
        }
    }

    pub(super) fn handle_rail_node_mouse_down(
        &mut self,
        node_id: SharedString,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_interact_with_rail_node(&node_id) {
            if let Some(focus_handle) = self.rail_focus_handles.get(&node_id) {
                focus_handle.focus(window, cx);
            }
            self.pressed_node = Some(node_id);
            cx.notify();
        }
    }

    pub(super) fn handle_rail_node_click(
        &mut self,
        node_id: SharedString,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() || !self.can_interact_with_rail_node(&node_id) {
            return;
        }

        if self.find_top_level_main_node(&node_id).is_some_and(|node| !node.has_children()) {
            self.close_rail_submenu(cx);
            self.activate_node(node_id, cx);
        } else {
            self.set_open_rail_submenu(node_id, cx);
            cx.notify();
        }
    }

    pub(super) fn handle_rail_node_bounds(
        &mut self,
        node_id: SharedString,
        bounds: &Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.rail_node_bounds.insert(node_id.clone(), *bounds);
        if self.open_rail_submenu.as_ref() == Some(&node_id) {
            self.rail_submenu_lifecycle.set_trigger_bounds(*bounds);
        }
    }

    pub(super) fn handle_rail_submenu_mouse_down_out(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.rail_submenu_lifecycle.dismiss_from_outside_click(event.position) {
            self.close_rail_submenu(cx);
            cx.notify();
        }
    }

    pub(super) fn handle_escape_focus(&mut self, _: &EscapeFocus, window: &mut Window, cx: &mut Context<Self>) {
        if cx.stop_active_drag(window) {
            cx.stop_propagation();
        } else if self.open_rail_submenu.is_some() {
            self.close_rail_submenu(cx);
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    pub(super) fn handle_rail_submenu_item_hover(
        &mut self,
        parent_node_id: SharedString,
        index: usize,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if !hovered || self.open_rail_submenu.as_ref() != Some(&parent_node_id) {
            return;
        }

        let open_submenu = self
            .find_top_level_main_node(&parent_node_id)
            .and_then(|parent| parent.children.iter().filter(|node| node.visible).nth(index))
            .is_some_and(|node| node.enabled && node.has_children())
            .then_some(index);
        let active_path = Some(MenuPath::Root(index));

        if self.rail_submenu_open_submenu != open_submenu || self.rail_submenu_active_path != active_path {
            self.rail_submenu_open_submenu = open_submenu;
            self.rail_submenu_active_path = active_path;
            cx.notify();
        }
    }

    pub(super) fn handle_rail_submenu_item_click(
        &mut self,
        node_id: SharedString,
        event: &ClickEvent,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() {
            return;
        }

        self.close_rail_submenu(cx);
        self.activate_node(node_id, cx);
        cx.notify();
    }

    pub(super) fn activate_node(&mut self, node_id: SharedString, cx: &mut Context<Self>) {
        let Some(node) = self.find_node(&node_id) else {
            return;
        };
        if !node.enabled || node.kind != NavNodeKind::Item || node.has_children() {
            return;
        }

        let label = node.label.clone().unwrap_or_else(|| node_id.clone());
        if self.model.selected_id.as_ref() != Some(&node_id) {
            self.model.selected_id = Some(node_id.clone());
            cx.notify();
        }

        cx.emit(SidebarPanelEngineEvent::Activate { node_id, label });
    }

    pub(super) fn set_hovered_node(&mut self, hovered_node: Option<SharedString>, cx: &mut Context<Self>) {
        if self.hovered_node == hovered_node {
            return;
        }

        if let Some(node_id) = self.hovered_node.take() {
            cx.emit(SidebarPanelEngineEvent::ItemHoverChanged { node_id, hovered: false });
        }

        if let Some(node_id) = hovered_node {
            cx.emit(SidebarPanelEngineEvent::ItemHoverChanged { node_id: node_id.clone(), hovered: true });
            self.hovered_node = Some(node_id);
        }
    }

    pub(super) fn clear_hovered_node(&mut self, cx: &mut Context<Self>) {
        self.set_hovered_node(None, cx);
    }

    pub(super) fn focused_target(&self, window: &Window) -> Option<NavigationFocusTarget> {
        self.visible_focus_nodes
            .iter()
            .find_map(|(target, focus_handle)| focus_handle.is_focused(window).then(|| target.clone()))
    }

    pub(super) fn move_focus(&mut self, window: &mut Window, cx: &mut Context<Self>, direction: FocusDirection) {
        let Some((target, next_focus_handle)) = next_focus_target(&self.visible_focus_nodes, window, direction) else {
            return;
        };

        next_focus_handle.focus(window, cx);
        self.activate_node(target, cx);
    }

    pub(super) fn handle_select_previous_item(
        &mut self,
        _: &SelectPreviousItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_focus(window, cx, FocusDirection::Previous);
    }

    pub(super) fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::Next);
    }

    pub(super) fn handle_select_first_item(
        &mut self,
        _: &SelectFirstItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_focus(window, cx, FocusDirection::First);
    }

    pub(super) fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::Last);
    }

    pub(super) fn handle_open_submenu(&mut self, _: &OpenSubmenu, window: &mut Window, cx: &mut Context<Self>) {
        self.handle_horizontal_arrow(true, window, cx);
    }

    pub(super) fn handle_close_submenu(&mut self, _: &CloseSubmenu, window: &mut Window, cx: &mut Context<Self>) {
        self.handle_horizontal_arrow(false, window, cx);
    }

    fn handle_horizontal_arrow(&mut self, right: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        let rtl =
            crate::infra::locale::LocaleContext::current(cx).direction == crate::infra::locale::LayoutDirection::Rtl;
        let expand = right != rtl;
        if self.model.collapsed {
            if !expand && self.open_rail_submenu.is_some() {
                self.close_rail_submenu(cx);
                cx.notify();
            } else if expand
                && let Some(node_id) = self.focused_target(window)
                && self.find_top_level_main_node(&node_id).is_some_and(|node| node.has_children())
            {
                // The expand arrow opens a rail parent's submenu, like Enter.
                self.set_open_rail_submenu(node_id, cx);
                cx.notify();
            }
            return;
        }
        if let Some(node_id) = self.focused_target(window) {
            self.set_node_expanded(node_id, expand, cx);
        }
    }

    pub(super) fn handle_activate_control(&mut self, _: &ActivateControl, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let Some(node_id) = self.focused_target(window) else {
            return;
        };

        if self.model.collapsed {
            if self.find_top_level_main_node(&node_id).is_some_and(|node| !node.has_children()) {
                self.close_rail_submenu(cx);
                self.activate_node(node_id, cx);
            } else {
                self.set_open_rail_submenu(node_id, cx);
                cx.notify();
            }
        } else if self.find_node(&node_id).is_some_and(|node| node.has_children()) {
            self.toggle_node_expanded(node_id, cx);
        } else {
            self.activate_node(node_id, cx);
        }
    }

    pub(super) fn sync_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.visible_focus_nodes == self.focus_subscription_keys {
            return;
        }

        self.focus_subscriptions.clear();
        self.focus_subscription_keys = self.visible_focus_nodes.clone();

        for (target, focus_handle) in self.focus_subscription_keys.clone() {
            self.focus_subscriptions.push(cx.on_focus_in(&focus_handle, window, {
                let node_id = target.clone();
                move |this, _window, cx| {
                    let focus_changed = this.emit_focus_changed(true, cx);
                    cx.emit(SidebarPanelEngineEvent::ItemFocused {
                        node_id: node_id.clone(),
                        label: this.node_label(&node_id),
                    });
                    if focus_changed {
                        cx.notify();
                    }
                }
            }));
            self.focus_subscriptions.push(cx.on_focus_out(
                &focus_handle,
                window,
                move |this, _: FocusOutEvent, window, cx| {
                    if !this.any_focus_target_focused(window) && this.emit_focus_changed(false, cx) {
                        cx.notify();
                    }
                },
            ));
        }
    }

    pub(super) fn any_focus_target_focused(&self, window: &Window) -> bool {
        self.focus_subscription_keys.iter().any(|(_, focus_handle)| focus_handle.is_focused(window))
    }

    pub(super) fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(SidebarPanelEngineEvent::FocusChanged { focused });
        true
    }
}

pub(super) fn nav_node_to_menu_item(node: &NavNode) -> MenuItem {
    let label = node.label.clone().unwrap_or_else(|| node.id.clone());
    let mut item = MenuItem::new(node.id.clone()).label(label).enabled(node.enabled);

    if let Some(icon) = node.icon {
        item = item.icon(icon);
    }

    let submenu = node.children.iter().filter(|child| child.visible).map(nav_node_to_menu_item).collect::<Vec<_>>();
    if !submenu.is_empty() {
        item = item.submenu(submenu);
    }

    item
}

pub(super) fn nav_menu_click_node_ids(nodes: &[NavNode], open_submenu: Option<usize>) -> Vec<SharedString> {
    let visible_nodes = nodes.iter().filter(|node| node.visible);
    let mut ids = Vec::new();

    for (index, node) in visible_nodes.enumerate() {
        if !node.enabled {
            continue;
        }

        if !node.has_children() {
            ids.push(node.id.clone());
        } else if open_submenu == Some(index) {
            ids.extend(
                node.children
                    .iter()
                    .filter(|child| child.visible && child.enabled && !child.has_children())
                    .map(|child| child.id.clone()),
            );
        }
    }

    ids
}

#[derive(Clone, Copy)]
pub(super) enum SidebarNodeInteraction {
    Default,
    Rail,
}

pub(super) fn push_template_handlers(
    nodes: &[RenderedNavNode],
    handlers: &mut SidebarPanelTemplateHandlers,
    cx: &mut Context<SidebarPanelEngine>,
    interaction: SidebarNodeInteraction,
) {
    for node in nodes {
        let hover_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                Rc::make_mut(&mut handlers.row_hovers).push(Rc::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_node_hover(hover_id.clone(), *hovered, cx);
                })));
            }
            SidebarNodeInteraction::Rail => {
                Rc::make_mut(&mut handlers.row_hovers).push(Rc::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_rail_node_hover(hover_id.clone(), *hovered, cx);
                })));
            }
        }

        if matches!(interaction, SidebarNodeInteraction::Rail) {
            let bounds_id = node.id.clone();
            Rc::make_mut(&mut handlers.row_bounds).push(Rc::new(cx.listener(move |this, bounds, window, cx| {
                this.handle_rail_node_bounds(bounds_id.clone(), bounds, window, cx);
            })));
        }

        let mouse_down_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                Rc::make_mut(&mut handlers.row_mouse_downs).push(Rc::new(cx.listener(
                    move |this, event, window, cx| {
                        this.handle_node_mouse_down(mouse_down_id.clone(), event, window, cx);
                    },
                )));
            }
            SidebarNodeInteraction::Rail => {
                Rc::make_mut(&mut handlers.row_mouse_downs).push(Rc::new(cx.listener(
                    move |this, event, window, cx| {
                        this.handle_rail_node_mouse_down(mouse_down_id.clone(), event, window, cx);
                    },
                )));
            }
        }

        Rc::make_mut(&mut handlers.row_mouse_ups).push(Rc::new(cx.listener(SidebarPanelEngine::handle_node_mouse_up)));
        Rc::make_mut(&mut handlers.row_mouse_up_outs)
            .push(Rc::new(cx.listener(SidebarPanelEngine::handle_node_mouse_up)));

        let click_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                Rc::make_mut(&mut handlers.row_clicks).push(Rc::new(cx.listener(move |this, event, window, cx| {
                    this.handle_node_click(click_id.clone(), event, window, cx);
                })));
            }
            SidebarNodeInteraction::Rail => {
                Rc::make_mut(&mut handlers.row_clicks).push(Rc::new(cx.listener(move |this, event, window, cx| {
                    this.handle_rail_node_click(click_id.clone(), event, window, cx);
                })));
            }
        }

        if matches!(interaction, SidebarNodeInteraction::Default) && node.has_children {
            let node_id = node.id.clone();
            Rc::make_mut(&mut handlers.children_height_reports).insert(
                node_id.clone(),
                Rc::new(cx.listener(move |this, height, _window, cx| {
                    this.set_branch_height(node_id.clone(), *height, cx);
                })),
            );
        }

        push_template_handlers(&node.children, handlers, cx, interaction);
    }
}

// Compare the rendered preorder directly so cache hits do not allocate a signature.
// Inspecting the projection also catches children disappearing after a collapse animation.
fn signature_matches(regions: &[&[RenderedNavNode]], signature: &[(SharedString, bool)]) -> bool {
    fn matches(nodes: &[RenderedNavNode], signature: &[(SharedString, bool)], index: &mut usize) -> bool {
        for node in nodes {
            if !signature.get(*index).is_some_and(|(id, branch)| *id == node.id && *branch == node.has_children) {
                return false;
            }
            *index += 1;
            if !matches(&node.children, signature, index) {
                return false;
            }
        }
        true
    }
    let mut index = 0;
    regions.iter().all(|nodes| matches(nodes, signature, &mut index)) && index == signature.len()
}

// Match the preorder used by handler construction, including rendered children.
fn handler_signature(nodes: &[RenderedNavNode], signature: &mut Vec<(SharedString, bool)>) {
    for node in nodes {
        signature.push((node.id.clone(), node.has_children));
        handler_signature(&node.children, signature);
    }
}

#[derive(Clone, Copy)]
pub(super) enum FocusDirection {
    Previous,
    Next,
    First,
    Last,
}

pub(super) fn next_focus_target(
    focus_nodes: &[(NavigationFocusTarget, FocusHandle)],
    window: &Window,
    direction: FocusDirection,
) -> Option<(NavigationFocusTarget, FocusHandle)> {
    if focus_nodes.is_empty() {
        return None;
    }

    match direction {
        FocusDirection::First => focus_nodes.first().cloned(),
        FocusDirection::Last => focus_nodes.last().cloned(),
        FocusDirection::Previous | FocusDirection::Next => {
            let current_index = focus_nodes.iter().position(|(_, focus_handle)| focus_handle.is_focused(window));
            let next_index = match (current_index, direction) {
                (Some(index), FocusDirection::Previous) => index.checked_sub(1).unwrap_or(focus_nodes.len() - 1),
                (Some(index), FocusDirection::Next) => (index + 1) % focus_nodes.len(),
                (None, FocusDirection::Previous) => focus_nodes.len() - 1,
                (None, FocusDirection::Next) => 0,
                _ => unreachable!(),
            };

            focus_nodes.get(next_index).cloned()
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn row_handlers_reuse_and_follow_projection_and_mode_changes() {
    let mut app = gpui::TestAppContext::single();
    let (engine, cx) = app.add_window_view(|_, cx| {
        SidebarPanelEngine::from_builder(
            super::super::SidebarPanelEngineBuilder::new("cached-handlers")
                .item(NavNode::new("first").icon(lucide_svg_static::Icon::Search))
                .item(
                    NavNode::new("branch")
                        .icon(lucide_svg_static::Icon::Search)
                        .expanded(true)
                        .child(NavNode::new("child")),
                ),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, app| {
        engine.update(app, |view, cx| {
            let model = view.render_model(window, cx);
            let first = view.template_handlers(&model, cx);
            let (second, allocations, _) = crate::perf_measure::allocations_for(|| view.template_handlers(&model, cx));
            assert_eq!(allocations, 0, "warm row-handler binding must not allocate temporary defaults");
            assert!(Rc::ptr_eq(&first.row_bounds, &second.row_bounds));
            assert!(Rc::ptr_eq(&first.row_hovers, &second.row_hovers));
            assert!(Rc::ptr_eq(&first.row_mouse_downs, &second.row_mouse_downs));
            assert!(Rc::ptr_eq(&first.row_mouse_ups, &second.row_mouse_ups));
            assert!(Rc::ptr_eq(&first.row_mouse_up_outs, &second.row_mouse_up_outs));
            assert!(Rc::ptr_eq(&first.row_clicks, &second.row_clicks));
            assert!(Rc::ptr_eq(&first.children_height_reports, &second.children_height_reports));
            assert!(Rc::ptr_eq(&first.row_hovers[0], &second.row_hovers[0]));
            assert!(Rc::ptr_eq(&first.row_clicks[0], &second.row_clicks[0]));
            assert!(Rc::ptr_eq(&first.children_height_reports["branch"], &second.children_height_reports["branch"]));
            // Animation completion changes the projection even without a model mutation.
            let mut collapsed_projection = view.render_model(window, cx);
            collapsed_projection.nodes[1].children.clear();
            let collapsed_children = view.template_handlers(&collapsed_projection, cx);
            assert_eq!(collapsed_children.row_hovers.len(), first.row_hovers.len() - 1);
            assert!(!Rc::ptr_eq(&first.row_hovers, &collapsed_children.row_hovers));
            view.model.nodes.swap(0, 1);
            let model = view.render_model(window, cx);
            let reordered = view.template_handlers(&model, cx);
            assert!(!Rc::ptr_eq(&first.row_hovers[0], &reordered.row_hovers[0]));
            view.model.collapsed = true;
            let model = view.render_model(window, cx);
            let rail = view.template_handlers(&model, cx);
            assert!(!Rc::ptr_eq(&reordered.row_hovers[0], &rail.row_hovers[0]));
            assert_eq!(rail.row_bounds.len(), rail.row_hovers.len());
            assert!(rail.children_height_reports.is_empty());
            view.model.nodes.clear();
            let model = view.render_model(window, cx);
            let empty = view.template_handlers(&model, cx);
            assert!(empty.row_hovers.is_empty());
            assert!(view.row_handlers.borrow().as_ref().unwrap().handlers.row_hovers.is_empty());
        })
    });
}

#[cfg(all(test, feature = "test-support"))]
#[path = "perf_tests.rs"]
mod perf_tests;
