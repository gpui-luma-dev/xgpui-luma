//! Validated same-sidebar moves. The payload owns a complete keyed subtree.
use std::{collections::HashSet, rc::Rc};
use gpui::{Context, Div, EntityId, SharedString, Stateful, Window, div, prelude::*, px};
use crate::infra::drag_drop::{
    bind_drag_source, DragDropElementExt, DragDropEvent, DropEdge, DropProposal, DropZone, KeyedDrag, KeyedDropTarget,
};
use super::{SidebarControl, SidebarEvent, SidebarPresentation};
use super::model::{SidebarMenuItemModel as Item, SidebarPanelModel, panel_to_nav_nodes};

/// Opt-in moves within one Sidebar. An empty group filter allows all content groups.
#[derive(Clone, Default)]
pub struct SidebarDragDrop {
    groups: HashSet<SharedString>,
}
impl SidebarDragDrop {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn allowed_groups(mut self, groups: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.groups = groups.into_iter().map(Into::into).collect();
        self
    }
    fn allows(&self, group: &SharedString) -> bool {
        self.groups.is_empty() || self.groups.contains(group)
    }
}
/// Stable group, parent and sibling gap; `None` means the end of that sibling list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarDropLocation {
    pub group_id: SharedString,
    pub parent_id: Option<SharedString>,
    pub before: Option<SharedString>,
}
/// Sidebar uses the shared seven-phase stream with a structured destination.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SidebarDragEvent {
    DragStarted {
        source: EntityId,
        keys: Vec<SharedString>,
    },
    ItemsRemoved {
        source: EntityId,
        target: EntityId,
        keys: Vec<SharedString>,
    },
    ItemsAdded {
        source: EntityId,
        target: EntityId,
        keys: Vec<SharedString>,
        location: SidebarDropLocation,
    },
    ItemsReordered {
        target: EntityId,
        keys: Vec<SharedString>,
        location: SidebarDropLocation,
    },
    Dropped {
        source: EntityId,
        target: EntityId,
        keys: Vec<SharedString>,
        location: SidebarDropLocation,
        changed: bool,
    },
    DropRejected {
        source: EntityId,
        target: EntityId,
        keys: Vec<SharedString>,
        location: SidebarDropLocation,
        error: SharedString,
    },
    DragEnded {
        source: EntityId,
        keys: Vec<SharedString>,
        reason: crate::infra::drag_drop::DragEndReason,
    },
}

pub(super) type SidebarDrag = KeyedDrag<EntityId, SharedString>;
#[derive(Clone)]
pub struct SidebarDropProposal {
    inner: DropProposal<EntityId, SharedString>,
    location: SidebarDropLocation,
}
impl SidebarDropProposal {
    pub fn source(&self) -> &EntityId {
        self.inner.source()
    }
    pub fn target(&self) -> &EntityId {
        self.inner.target()
    }
    pub fn keys(&self) -> &[SharedString] {
        self.inner.keys()
    }
    pub fn location(&self) -> &SidebarDropLocation {
        &self.location
    }
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}
pub(super) type DropHandler =
    Rc<dyn Fn(&mut SidebarControl, &SidebarDropProposal, &mut Window, &mut Context<SidebarControl>)>;

fn find<'a>(items: &'a [Item], id: &SharedString) -> Option<&'a Item> {
    items.iter().find_map(|item| {
        if item.id == *id {
            Some(item)
        } else {
            find(&item.children, id)
        }
    })
}
fn siblings_mut<'a>(items: &'a mut Vec<Item>, parent: Option<&SharedString>) -> Option<&'a mut Vec<Item>> {
    let Some(parent) = parent else { return Some(items) };
    for item in items {
        if item.id == *parent {
            return Some(&mut item.children);
        }
        if let Some(result) = siblings_mut(&mut item.children, Some(parent)) {
            return Some(result);
        }
    }
    None
}
fn remove(items: &mut Vec<Item>, key: &SharedString) -> Option<Item> {
    if let Some(i) = items.iter().position(|item| item.id == *key) {
        return Some(items.remove(i));
    }
    items.iter_mut().find_map(|item| remove(&mut item.children, key))
}
fn locate(
    items: &[Item],
    key: &SharedString,
    group: &SharedString,
    parent: Option<&SharedString>,
) -> Option<SidebarDropLocation> {
    for (index, item) in items.iter().enumerate() {
        if item.id == *key {
            return Some(SidebarDropLocation {
                group_id: group.clone(),
                parent_id: parent.cloned(),
                before: items.get(index + 1).map(|item| item.id.clone()),
            });
        }
        if let Some(location) = locate(&item.children, key, group, Some(&item.id)) {
            return Some(location);
        }
    }
    None
}
fn group_id(group: &super::model::SidebarGroupModel) -> Option<&SharedString> {
    group.id.as_ref().or_else(|| group.menu.as_ref().map(|menu| &menu.id))
}
pub(super) fn location(panel: &SidebarPanelModel, key: &SharedString) -> Option<SidebarDropLocation> {
    panel
        .content
        .as_ref()?
        .groups
        .iter()
        .find_map(|group| locate(&group.menu.as_ref()?.items, key, group_id(group)?, None))
}
pub(super) fn item<'a>(panel: &'a SidebarPanelModel, key: &SharedString) -> Option<&'a Item> {
    panel.content.as_ref()?.groups.iter().find_map(|group| find(&group.menu.as_ref()?.items, key))
}
fn validate_ids(panel: &SidebarPanelModel) -> Result<(), SharedString> {
    fn visit(items: &[Item], ids: &mut HashSet<SharedString>) -> bool {
        items.iter().all(|item| ids.insert(item.id.clone()) && visit(&item.children, ids))
    }
    let mut ids = HashSet::new();
    let mut groups = HashSet::new();
    if let Some(content) = &panel.content {
        for (index, group) in content.groups.iter().enumerate() {
            if group.label.is_some() {
                ids.insert(group.id.clone().unwrap_or_else(|| format!("{}-group-{index}", panel.id).into()));
            }
        }
    }
    if let Some(content) = &panel.content {
        for group in &content.groups {
            if let Some(id) = group_id(group)
                && !groups.insert(id.clone())
            {
                return Err("Duplicate Sidebar group ID".into());
            }
            if let Some(menu) = &group.menu
                && !visit(&menu.items, &mut ids)
            {
                return Err("Duplicate Sidebar item ID".into());
            }
        }
    }
    if let Some(footer) = &panel.footer
        && !visit(&footer.items, &mut ids)
    {
        return Err("Duplicate Sidebar item ID".into());
    }
    Ok(())
}
pub(super) fn move_item(
    panel: &mut SidebarPanelModel,
    key: &SharedString,
    dest: &SidebarDropLocation,
    config: &SidebarDragDrop,
) -> Result<bool, SharedString> {
    validate_ids(panel)?;
    let source = location(panel, key).ok_or_else(|| SharedString::from("Source item is missing or fixed"))?;
    if !config.allows(&source.group_id) || !config.allows(&dest.group_id) {
        return Err("Group does not allow moves".into());
    }
    let moving = item(panel, key).ok_or_else(|| SharedString::from("Missing source"))?;
    if moving.disabled || !moving.draggable {
        return Err("Source is disabled or fixed".into());
    }
    // Disabled ancestors prevent moving their descendants too.
    fn enabled_path(items: &[Item], key: &SharedString) -> Option<bool> {
        for item in items {
            if item.id == *key {
                return Some(!item.disabled);
            }
            if let Some(enabled) = enabled_path(&item.children, key) {
                return Some(enabled && !item.disabled);
            }
        }
        None
    }
    let content = panel.content.as_ref().ok_or_else(|| SharedString::from("Missing content"))?;
    if !content
        .groups
        .iter()
        .any(|g| g.menu.as_ref().and_then(|m| enabled_path(&m.items, key)) == Some(true))
    {
        return Err("Source ancestor is disabled".into());
    }
    if let Some(parent) = &dest.parent_id
        && (parent == key || find(&moving.children, parent).is_some())
    {
        return Err("Cannot move an item into its own subtree".into());
    }
    let target_group = content
        .groups
        .iter()
        .position(|g| group_id(g) == Some(&dest.group_id))
        .ok_or_else(|| SharedString::from("Missing destination group"))?;
    let target_items = &content.groups[target_group]
        .menu
        .as_ref()
        .ok_or_else(|| SharedString::from("Missing destination menu"))?
        .items;
    if let Some(parent) = &dest.parent_id {
        let target = find(target_items, parent).ok_or_else(|| SharedString::from("Missing destination parent"))?;
        if !target.branch || enabled_path(target_items, parent) != Some(true) {
            return Err("Destination is disabled or is not a branch".into());
        }
    }
    let mut next = panel.clone();
    let groups = &mut next.content.as_mut().ok_or_else(|| SharedString::from("Missing content"))?.groups;
    // Resolve the gap before removal; dropping before oneself is a no-op.
    let target = siblings_mut(
        &mut groups[target_group].menu.as_mut().ok_or_else(|| SharedString::from("Missing menu"))?.items,
        dest.parent_id.as_ref(),
    )
    .ok_or_else(|| SharedString::from("Missing parent"))?;
    let mut gap = match &dest.before {
        Some(before) => target
            .iter()
            .position(|i| i.id == *before)
            .ok_or_else(|| SharedString::from("Anchor is not a destination sibling"))?,
        None => target.len(),
    };
    if source.group_id == dest.group_id && source.parent_id == dest.parent_id {
        let index = target.iter().position(|i| i.id == *key).ok_or_else(|| SharedString::from("Stale source"))?;
        if index < gap {
            gap -= 1
        }
        if gap == index {
            return Ok(false);
        }
    }
    let moving = groups
        .iter_mut()
        .find_map(|g| remove(&mut g.menu.as_mut()?.items, key))
        .ok_or_else(|| SharedString::from("Missing source"))?;
    let target = siblings_mut(
        &mut groups[target_group].menu.as_mut().ok_or_else(|| SharedString::from("Missing menu"))?.items,
        dest.parent_id.as_ref(),
    )
    .ok_or_else(|| SharedString::from("Missing parent"))?;
    target.insert(gap, moving);
    *panel = next;
    Ok(true)
}
pub(super) fn set_expanded(panel: &mut SidebarPanelModel, key: &SharedString, expanded: bool) {
    fn apply(items: &mut [Item], key: &SharedString, expanded: bool) {
        for item in items {
            if item.id == *key {
                item.expanded = Some(expanded);
            }
            apply(&mut item.children, key, expanded);
        }
    }
    if let Some(content) = &mut panel.content {
        for group in &mut content.groups {
            if let Some(menu) = &mut group.menu {
                apply(&mut menu.items, key, expanded);
            }
        }
    }
}
impl SidebarControl {
    /// Retained keyed post-drop geometry and running transitions.
    pub fn drop_layout_motion(&self, cx: &gpui::App) -> crate::motion::DropLayoutMotion<SharedString> {
        self.panel.read(cx).drop_layout.clone()
    }
    /// Current keyed position, including group and parent, after SDK-owned moves.
    pub fn item_location(&self, id: impl Into<SharedString>) -> Option<SidebarDropLocation> {
        location(self.model.sidebar.as_ref()?, &id.into())
    }
    /// Selected navigation identity, preserved across moves and presentations.
    pub fn selected_id(&self) -> Option<&SharedString> {
        self.model.selected_id.as_ref()
    }

    pub fn set_drag_drop(&mut self, config: Option<SidebarDragDrop>, cx: &mut Context<Self>) {
        self.cancel_drag(cx);
        self.model.drag_drop = config;
        self.panel.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
    /// Override automatic moves. Answer synchronously with commit_drop/reject_drop.
    pub fn set_drop_handler(
        &mut self,
        handler: impl Fn(&mut Self, &SidebarDropProposal, &mut Window, &mut Context<Self>) + 'static,
    ) {
        self.drop_handler = Some(Rc::new(handler));
    }
    /// Validate and atomically move one keyed subtree, retaining navigation state.
    pub fn commit_drop(&mut self, p: &SidebarDropProposal, cx: &mut Context<Self>) -> Result<bool, SharedString> {
        let result = (|| {
            if !self.model.enabled
                || self.model.presentation != SidebarPresentation::Expanded
                || !p.is_active()
                || *p.source() != cx.entity_id()
                || *p.target() != cx.entity_id()
                || !self.active_drag.as_ref().is_some_and(|d| p.inner.is_from(d))
            {
                return Err("Sidebar drag is disabled, foreign or ended".into());
            }
            let config =
                self.model.drag_drop.as_ref().ok_or_else(|| SharedString::from("Sidebar moves are disabled"))?;
            if p.keys().len() != 1 {
                return Err("Sidebar moves require one subtree".into());
            }
            let panel = self.model.sidebar.as_mut().ok_or_else(|| SharedString::from("Missing Sidebar data"))?;
            move_item(panel, &p.keys()[0], &p.location, config)
        })();
        match result {
            Ok(changed) => {
                if changed && let Some(panel) = &self.model.sidebar {
                    self.panel.read(cx).drop_layout.start();
                    let (nodes, _) = panel_to_nav_nodes(panel);
                    self.panel.update(cx, |engine, cx| engine.move_items(nodes, cx));
                }
                for event in p.inner.committed(changed, p.keys().to_vec()) {
                    self.drop_event(event, &p.location, cx);
                }
                Ok(changed)
            }
            Err(error) => {
                self.reject_drop(p, error.clone(), cx);
                Err(error)
            }
        }
    }
    /// Reject a live proposal from this Sidebar without changing navigation data.
    pub fn reject_drop(&mut self, p: &SidebarDropProposal, error: impl Into<SharedString>, cx: &mut Context<Self>) {
        if self.active_drag.as_ref().is_some_and(|d| p.inner.is_from(d)) {
            for event in p.inner.rejected(error) {
                self.drop_event(event, &p.location, cx);
            }
        }
    }
    pub(super) fn cancel_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(event) = self.active_drag.take().and_then(|d| d.cancel()) {
            self.drag_event(event, cx);
        }
    }
    fn drag_event(&mut self, event: DragDropEvent<EntityId, SharedString>, cx: &mut Context<Self>) {
        if matches!(event, DragDropEvent::DragStarted { .. }) {
            self.panel.read(cx).drop_layout.cancel();
        }
        if matches!(event, DragDropEvent::DragEnded { .. }) {
            self.active_drag = None;
        }
        let event = match event {
            DragDropEvent::DragStarted { source, keys } => SidebarDragEvent::DragStarted { source, keys },
            DragDropEvent::DragEnded { source, keys, reason } => SidebarDragEvent::DragEnded { source, keys, reason },
            _ => return,
        };
        cx.emit(SidebarEvent::DragDrop(event));
        self.panel.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
    fn drop_event(
        &mut self,
        event: DragDropEvent<EntityId, SharedString>,
        location: &SidebarDropLocation,
        cx: &mut Context<Self>,
    ) {
        let event = match event {
            DragDropEvent::ItemsReordered { target, keys, .. } => {
                SidebarDragEvent::ItemsReordered { target, keys, location: location.clone() }
            }
            DragDropEvent::Dropped { source, target, keys, changed, .. } => {
                SidebarDragEvent::Dropped { source, target, keys, changed, location: location.clone() }
            }
            DragDropEvent::DropRejected { source, target, keys, error, .. } => {
                SidebarDragEvent::DropRejected { source, target, keys, error, location: location.clone() }
            }
            event => {
                self.drag_event(event, cx);
                return;
            }
        };
        cx.emit(SidebarEvent::DragDrop(event));
    }
    fn drop_item(
        &mut self,
        inner: DropProposal<EntityId, SharedString>,
        location: SidebarDropLocation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let proposal = SidebarDropProposal { inner, location };
        if let Some(handler) = self.drop_handler.clone() {
            handler(self, &proposal, window, cx);
            if proposal.is_active() {
                self.reject_drop(&proposal, "owner did not accept the drop", cx);
            }
        } else {
            let _ = self.commit_drop(&proposal, cx);
        }
    }
    fn bind_item(
        &self,
        mut surface: Stateful<Div>,
        id: &SharedString,
        style: std::sync::Arc<xgpui_luma_style::Resolved>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Some(config) = &self.model.drag_drop else {
            return surface;
        };
        if !self.model.enabled || self.model.presentation != SidebarPresentation::Expanded {
            return surface;
        }
        let Some(panel) = &self.model.sidebar else {
            return surface;
        };
        #[cfg(any(test, feature = "test-support"))]
        {
            let id = id.clone();
            surface = surface.debug_selector(move || format!("sidebar-drag-{id}"));
        }
        let scope = cx.entity_id();
        let mut destinations = Vec::new();
        let mut branch = false;
        if let Some(after) = location(panel, id) {
            if !config.allows(&after.group_id) {
                return surface;
            }
            let Some(item) = item(panel, id) else { return surface };
            if item.disabled {
                return surface;
            }
            branch = item.branch;
            if item.draggable
                && let Some(drag) = SidebarDrag::new(scope, scope, vec![id.clone()])
            {
                let preview_style = style.clone();
                let owner = cx.weak_entity();
                let label = item.label.clone();
                surface = bind_drag_source(
                    surface.cursor_grab(),
                    drag,
                    move |drag, offset, _, cx| {
                        let _ = owner.update(cx, |sidebar, cx| {
                            sidebar.active_drag = Some(drag.clone());
                            sidebar.panel.update(cx, |_, cx| cx.notify());
                            cx.notify();
                        });
                        cx.new(|_| crate::infra::drag_drop::DragPreview {
                            label: label.clone(),
                            offset,
                            style: preview_style.clone(),
                            selector: "sidebar-drag-preview",
                        })
                    },
                    cx,
                    Self::drag_event,
                );
            }
            let mut before = after.clone();
            before.before = Some(id.clone());
            destinations.push((Some(DropEdge::Before), before));
            destinations.push((Some(DropEdge::After), after.clone()));
            if item.branch {
                destinations.push((
                    None,
                    SidebarDropLocation { group_id: after.group_id, parent_id: Some(id.clone()), before: None },
                ));
            }
        } else if let Some(content) = &panel.content {
            for (index, group) in content.groups.iter().enumerate() {
                let render_id = group.id.clone().unwrap_or_else(|| format!("{}-group-{index}", panel.id).into());
                if render_id == *id
                    && let Some(group) = group_id(group).filter(|g| config.allows(g))
                {
                    destinations
                        .push((None, SidebarDropLocation { group_id: group.clone(), parent_id: None, before: None }));
                }
            }
        }
        if cx.has_active_drag() {
            for (edge, location) in destinations {
                let zone = DropZone {
                    axis: gpui::Axis::Vertical,
                    edge: edge.unwrap_or(DropEdge::Before),
                    item_extent: px(0.),
                    following_gap: px(0.),
                    marker_width: px(crate::theme::stylesheet::SharedGeometry::sdk().drag_marker_width),
                };
                let target = match edge {
                    Some(edge) => {
                        if !branch {
                            zone.apply_measured(div().id(if edge == DropEdge::Before {
                                "sidebar-before"
                            } else {
                                "sidebar-after"
                            }))
                        } else if edge == DropEdge::Before {
                            zone.apply_measured(div().id("sidebar-before")).bottom(gpui::relative(0.75))
                        } else {
                            zone.apply_measured(div().id("sidebar-after")).top(gpui::relative(0.75))
                        }
                    }
                    None => div()
                        .id("sidebar-into")
                        .border(px(crate::theme::stylesheet::SharedGeometry::sdk().drag_border_width))
                        .absolute()
                        .top(gpui::relative(0.25))
                        .h(gpui::relative(0.5))
                        .w_full(),
                };
                let owner = cx.weak_entity();
                let destination = location.clone();
                let target = crate::infra::drag_drop::bind_feedback(
                    target,
                    style.clone(),
                    scope,
                    move |drag: &SidebarDrag, app| {
                        drag.accepts(scope)
                            && owner.upgrade().is_some_and(|owner| {
                                let sidebar = owner.read(app);
                                sidebar.model.sidebar.as_ref().zip(sidebar.model.drag_drop.as_ref()).is_some_and(
                                    |(panel, config)| {
                                        let mut snapshot = panel.clone();
                                        drag.keys().first().is_some_and(|key| {
                                            move_item(&mut snapshot, key, &destination, config).unwrap_or(false)
                                        })
                                    },
                                )
                            })
                    },
                );
                surface = surface.relative().child(KeyedDropTarget::new(scope, scope, location.before.clone()).bind(
                    target,
                    cx,
                    move |this, proposal, window, cx| this.drop_item(proposal, location.clone(), window, cx),
                ));
            }
        }
        surface.cancel_drag_on_escape()
    }
}
/// Retained owner binding for native row templates. Attach to the row, not its subtree.
#[derive(Clone)]
pub struct SidebarItemDragBinding {
    pub(crate) owner: gpui::WeakEntity<SidebarControl>,
    pub(crate) id: SharedString,
}
impl SidebarItemDragBinding {
    pub fn bind(
        &self,
        surface: Stateful<Div>,
        style: std::sync::Arc<xgpui_luma_style::Resolved>,
        cx: &mut gpui::App,
    ) -> Stateful<Div> {
        self.owner
            .update(cx, |sidebar, cx| sidebar.bind_item(surface, &self.id, style, cx))
            .unwrap_or_else(|_| div().id("released-sidebar-row"))
    }
}
