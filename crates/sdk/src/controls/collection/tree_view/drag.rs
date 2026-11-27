//! Keyed tree locations and synchronous, host-owned drop transactions.
use std::rc::Rc;
use gpui::{App, Entity, EntityId, SharedString, WeakEntity, Window};
use crate::infra::drag_drop::{DragDropEvent, DragEndReason, DropProposal, KeyedDrag};
use super::{TreeViewControl, TreeViewEvent};

pub(super) type TreeDrag<T> = KeyedDrag<WeakEntity<TreeViewControl<T>>, SharedString>;
type DropHandler<T> = Rc<dyn Fn(TreeDropProposal<T>, &mut Window, &mut App)>;
type DropPolicy<T> = Rc<dyn Fn(&TreeDropProposal<T>, &App) -> bool>;

/// Before/After refer to siblings, including the entire anchor subtree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeDropPosition {
    Before,
    After,
    Into,
}

/// A location in loaded source data, independent of flattened row indices.
/// Into with no parent or anchor appends to the root collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeDropLocation {
    pub parent_id: Option<SharedString>,
    pub anchor_id: Option<SharedString>,
    pub position: TreeDropPosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeDragEndReason {
    MovedWithinTree,
    MovedAcrossTrees,
    Unchanged,
    Rejected,
    Cancelled,
}

/// One lifecycle per drag. `node_ids` contains normalized moving roots in
/// displayed order; `node_id` is the first root for single-node consumers.
/// The shared session produces all seven phases. Historical Started/Rejected/
/// Ended spellings and tree reason names remain compatible. Mutation phases are
/// emitted only for changed commits, never for no-ops, rejection or cancellation.
/// Downstream matches must allow additional lifecycle variants.
#[derive(Clone, Debug)]
pub enum TreeViewDragEvent {
    /// Committed transfer removal, delivered to the source before target add.
    ItemsRemoved {
        source: EntityId,
        target: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
    },
    /// Committed transfer addition, delivered to the target before completion.
    ItemsAdded {
        source: EntityId,
        target: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
        location: TreeDropLocation,
    },
    /// Committed move within a tree, including reparenting, before completion.
    ItemsReordered {
        target: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
        location: TreeDropLocation,
    },

    Started {
        source: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
    },
    Dropped {
        source: EntityId,
        target: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
        location: TreeDropLocation,
        changed: bool,
    },
    Rejected {
        source: EntityId,
        target: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
        error: SharedString,
    },
    Ended {
        source: EntityId,
        node_id: SharedString,
        node_ids: Vec<SharedString>,
        reason: TreeDragEndReason,
    },
}

/// Opt-in cooperation between trees with the same domain type and scope.
/// The synchronous callback validates/prepares both snapshots, applies them, then
/// calls `committed`; it may call `rejected` without changing either tree.
#[derive(Clone)]
pub struct TreeViewDragDrop<T: Clone + Send + Sync + 'static> {
    pub(super) scope: EntityId,
    pub(super) on_drop: DropHandler<T>,
    pub(super) policy: DropPolicy<T>,
    pub(super) drag_selected: bool,
}
impl<T: Clone + Send + Sync + 'static> TreeViewDragDrop<T> {
    pub fn new(scope: EntityId, on_drop: impl Fn(TreeDropProposal<T>, &mut Window, &mut App) + 'static) -> Self {
        Self { scope, on_drop: Rc::new(on_drop), policy: Rc::new(|_, _| true), drag_selected: false }
    }
    /// Drag visible selected roots as one group when the pressed row is selected.
    /// Selected descendants of selected ancestors are included only through their
    /// subtree. Hidden selections outside those subtrees stay put. Defaults false.
    pub fn drag_selected(mut self, enabled: bool) -> Self {
        self.drag_selected = enabled;
        self
    }
    /// Additional host policy, checked for markers, auto-scroll and again on drop.
    pub fn can_drop(mut self, policy: impl Fn(&TreeDropProposal<T>, &App) -> bool + 'static) -> Self {
        self.policy = Rc::new(policy);
        self
    }
}

#[derive(Clone)]
pub struct TreeDropProposal<T: Clone + Send + Sync + 'static> {
    pub(super) inner: DropProposal<WeakEntity<TreeViewControl<T>>, SharedString>,
    pub(super) location: TreeDropLocation,
    pub(super) node_id: SharedString,
    pub(super) target_revision: u64,
    pub(super) scope: EntityId,
}
impl<T: Clone + Send + Sync + 'static> TreeDropProposal<T> {
    pub fn source(&self) -> Option<Entity<TreeViewControl<T>>> {
        self.inner.source().upgrade()
    }
    pub fn target(&self) -> Option<Entity<TreeViewControl<T>>> {
        self.inner.target().upgrade()
    }
    /// First moving root. Use `node_ids` for grouped moves.
    pub fn node_id(&self) -> &SharedString {
        &self.node_id
    }
    /// Normalized moving roots, captured in displayed preorder at drag start.
    /// Each root carries its entire loaded subtree, including hidden descendants.
    pub fn node_ids(&self) -> &[SharedString] {
        self.inner.keys()
    }
    pub fn location(&self) -> &TreeDropLocation {
        &self.location
    }
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Revalidate immediately before mutation, including stale data, disabled
    /// nodes/parents, cycles, subtree ID collisions, and the current host policy.
    pub fn validate(&self, cx: &App) -> Result<(), SharedString> {
        if !self.is_active() {
            return Err("Drag session ended".into());
        }
        let source = self.source().ok_or_else(|| SharedString::from("Source closed"))?;
        let target = self.target().ok_or_else(|| SharedString::from("Target closed"))?;
        let src = source.read(cx);
        let dst = target.read(cx);
        if src.drag_revision != Some(src.revision) || dst.revision != self.target_revision {
            return Err("Tree changed during drag".into());
        }
        if !src.model.enabled || !dst.model.enabled {
            return Err("Tree is disabled".into());
        }
        let config = dst.model.drag_drop.as_ref().ok_or_else(|| SharedString::from("Drop disabled"))?;
        if config.scope != self.scope || src.model.drag_drop.as_ref().is_none_or(|c| c.scope != self.scope) {
            return Err("Foreign drag scope".into());
        }
        if self.node_ids().iter().any(|id| !src.model.index.nodes.get(id).is_some_and(|n| n.enabled)) {
            return Err("Source node is missing or disabled".into());
        }
        let roots = self.node_ids().iter().collect();
        let location = &self.location;
        if dst.has_projection() && location.position != TreeDropPosition::Into {
            return Err("Before/after drops are disabled while filtering or sorting".into());
        }
        if let Some(parent) = &location.parent_id {
            if !dst.model.index.nodes.get(parent).is_some_and(|n| n.enabled && n.branch) {
                return Err("Destination parent is missing or disabled".into());
            }
            if source == target && src.model.index.is_in_subtrees(parent, &roots) {
                return Err("A node cannot move into its own subtree".into());
            }
        }
        match location.position {
            TreeDropPosition::Into => {
                if location.anchor_id != location.parent_id {
                    return Err("Invalid Into target".into());
                }
            }
            TreeDropPosition::Before | TreeDropPosition::After => {
                let anchor = location
                    .anchor_id
                    .as_ref()
                    .and_then(|id| dst.model.index.nodes.get(id))
                    .ok_or_else(|| SharedString::from("Sibling anchor is missing"))?;
                if !anchor.enabled || anchor.parent != location.parent_id {
                    return Err("Sibling anchor changed or is disabled".into());
                }
            }
        }
        if source != target
            && src
                .model
                .index
                .nodes
                .keys()
                .any(|id| dst.model.index.nodes.contains_key(id) && src.model.index.is_in_subtrees(id, &roots))
        {
            return Err("Destination contains an ID from the moved subtree".into());
        }
        if !(config.policy)(self, cx) {
            return Err("Host rejected this destination".into());
        }
        Ok(())
    }

    /// Call only after both host-owned snapshots and transferred state are applied.
    /// Completion is guarded across all retained proposals and preview release.
    pub fn committed(&self, changed: bool, cx: &mut App) {
        if self.inner.is_active() && changed {
            if let Some(source) = self.inner.source().upgrade() {
                source.read(cx).drop_layout.start();
            }
            if self.inner.source().entity_id() != self.inner.target().entity_id()
                && let Some(target) = self.inner.target().upgrade()
            {
                target.read(cx).drop_layout.start();
            }
        }
        for event in self.inner.committed(changed, self.node_ids().to_vec()) {
            deliver(event, Some(&self.location), cx);
        }
    }
    pub fn rejected(&self, error: impl Into<SharedString>, cx: &mut App) {
        for event in self.inner.rejected(error) {
            deliver(event, Some(&self.location), cx);
        }
    }
}

pub(super) fn deliver<T: Clone + Send + Sync + 'static>(
    event: DragDropEvent<WeakEntity<TreeViewControl<T>>, SharedString>,
    location: Option<&TreeDropLocation>,
    cx: &mut App,
) {
    let owner = event.list().clone();
    let mapped = project_event(event, location);
    if let Some(event) = mapped {
        let _ = owner.update(cx, |tree, cx| {
            if matches!(event, TreeViewDragEvent::Ended { .. }) {
                let changes = tree.rows_pointer.release();
                tree.emit_row_changes(changes, cx);
            }
            cx.emit(TreeViewEvent::DragDrop(event));
            cx.notify();
        });
    }
}

/// Project every shared lifecycle phase without inventing mutation events.
pub(super) fn project_event<T: Clone + Send + Sync + 'static>(
    event: DragDropEvent<WeakEntity<TreeViewControl<T>>, SharedString>,
    location: Option<&TreeDropLocation>,
) -> Option<TreeViewDragEvent> {
    match event {
        DragDropEvent::DragStarted { source, keys } => keys.first().map(|id| TreeViewDragEvent::Started {
            source: source.entity_id(),
            node_id: id.clone(),
            node_ids: keys.clone(),
        }),
        DragDropEvent::DragEnded { source, keys, reason } => keys.first().map(|id| TreeViewDragEvent::Ended {
            source: source.entity_id(),
            node_id: id.clone(),
            node_ids: keys.clone(),
            reason: match reason {
                DragEndReason::Reordered => TreeDragEndReason::MovedWithinTree,
                DragEndReason::Transferred => TreeDragEndReason::MovedAcrossTrees,
                DragEndReason::Unchanged => TreeDragEndReason::Unchanged,
                DragEndReason::Rejected => TreeDragEndReason::Rejected,
                DragEndReason::Cancelled => TreeDragEndReason::Cancelled,
            },
        }),
        DragDropEvent::Dropped { source, target, keys, changed, .. } => {
            keys.first().zip(location).map(|(id, location)| TreeViewDragEvent::Dropped {
                source: source.entity_id(),
                target: target.entity_id(),
                node_id: id.clone(),
                node_ids: keys.clone(),
                location: location.clone(),
                changed,
            })
        }
        DragDropEvent::DropRejected { source, target, keys, error, .. } => {
            keys.first().map(|id| TreeViewDragEvent::Rejected {
                source: source.entity_id(),
                target: target.entity_id(),
                node_id: id.clone(),
                node_ids: keys.clone(),
                error,
            })
        }
        DragDropEvent::ItemsRemoved { source, target, keys } => {
            keys.first().map(|id| TreeViewDragEvent::ItemsRemoved {
                source: source.entity_id(),
                target: target.entity_id(),
                node_id: id.clone(),
                node_ids: keys.clone(),
            })
        }
        DragDropEvent::ItemsAdded { source, target, keys, .. } => {
            keys.first().zip(location).map(|(id, location)| TreeViewDragEvent::ItemsAdded {
                source: source.entity_id(),
                target: target.entity_id(),
                node_id: id.clone(),
                node_ids: keys.clone(),
                location: location.clone(),
            })
        }
        DragDropEvent::ItemsReordered { target, keys, .. } => {
            keys.first().zip(location).map(|(id, location)| TreeViewDragEvent::ItemsReordered {
                target: target.entity_id(),
                node_id: id.clone(),
                node_ids: keys.clone(),
                location: location.clone(),
            })
        }
    }
}
