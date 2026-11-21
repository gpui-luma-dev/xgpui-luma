//! Keyed in-window drag sessions and drop proposals. GPUI owns gesture detection,
//! hit testing, and previews; hosts own acceptance policy and collection mutation.

mod appearance;
pub use appearance::{DragDropLook, styled_drag_drop_look};
pub(crate) use appearance::{DragPreview, bind_feedback};
mod binding;
pub(crate) use binding::bind_edge_scroll;
mod reorder;
pub(crate) use reorder::reorder_indices;

use std::{cell::Cell, collections::HashSet, hash::Hash, rc::Rc};
use gpui::{EntityId, SharedString};

pub use binding::{DragDropElementExt, DropEdge, DropZone, KeyedDropTarget, bind_drag_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DragEndReason {
    Transferred,
    Reordered,
    Unchanged,
    Rejected,
    Cancelled,
}

/// Domain-independent notifications. Deliver collection updates before reporting
/// a committed proposal. Keys in mutation notifications use host-reported order.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DragDropEvent<S, K> {
    DragStarted { source: S, keys: Vec<K> },
    ItemsRemoved { source: S, target: S, keys: Vec<K> },
    ItemsAdded { source: S, target: S, keys: Vec<K>, before: Option<K> },
    ItemsReordered { target: S, keys: Vec<K>, before: Option<K> },
    Dropped { source: S, target: S, keys: Vec<K>, before: Option<K>, changed: bool },
    DropRejected { source: S, target: S, keys: Vec<K>, before: Option<K>, error: SharedString },
    DragEnded { source: S, keys: Vec<K>, reason: DragEndReason },
}

impl<S, K> DragDropEvent<S, K> {
    /// Source for lifecycle/removal notifications; destination for drop/add/reorder.
    pub fn list(&self) -> &S {
        match self {
            Self::DragStarted { source, .. } | Self::ItemsRemoved { source, .. } | Self::DragEnded { source, .. } => {
                source
            }
            Self::ItemsAdded { target, .. }
            | Self::ItemsReordered { target, .. }
            | Self::Dropped { target, .. }
            | Self::DropRejected { target, .. } => target,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Pending,
    Active,
    Ended,
}

/// Immutable key capture with shared lifecycle across native payload/preview clones.
/// `S` identifies a host collection; `scope` identifies cooperating drop surfaces.
/// Use a common owner entity ID for surfaces allowed to exchange items.
#[derive(Clone, Debug)]
pub struct KeyedDrag<S, K> {
    scope: EntityId,
    source: S,
    keys: Vec<K>,
    phase: Rc<Cell<Phase>>,
}

impl<S: Clone, K: Clone + Eq + Hash> KeyedDrag<S, K> {
    /// Empty or duplicate-key captures are not draggable. Capture alone neither
    /// starts a gesture nor changes selection. Hosts may decline individual rows.
    pub fn new(scope: EntityId, source: S, keys: Vec<K>) -> Option<Self> {
        if keys.is_empty() || keys.iter().collect::<HashSet<_>>().len() != keys.len() {
            return None;
        }
        Some(Self { scope, source, keys, phase: Rc::new(Cell::new(Phase::Pending)) })
    }
}

impl<S: Clone, K: Clone> KeyedDrag<S, K> {
    pub fn source(&self) -> &S {
        &self.source
    }
    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    /// Only a live session from the same scope can hover, auto-scroll, or drop.
    pub fn accepts(&self, scope: EntityId) -> bool {
        self.scope == scope && self.phase.get() == Phase::Active
    }

    fn start(&self) -> Option<DragDropEvent<S, K>> {
        if self.phase.get() != Phase::Pending {
            return None;
        }
        self.phase.set(Phase::Active);
        Some(DragDropEvent::DragStarted { source: self.source.clone(), keys: self.keys.clone() })
    }

    fn end(&self, reason: DragEndReason) -> Option<DragDropEvent<S, K>> {
        if self.phase.get() != Phase::Active {
            return None;
        }
        self.phase.set(Phase::Ended);
        Some(DragDropEvent::DragEnded { source: self.source.clone(), keys: self.keys.clone(), reason })
    }

    /// Native preview release uses this for cancellation. Repeated cancellation
    /// or cancellation after a reported drop produces no duplicate notification.
    pub fn cancel(&self) -> Option<DragDropEvent<S, K>> {
        self.end(DragEndReason::Cancelled)
    }

    /// A keyed gap, resolved by the host against its current collection on drop.
    /// `None` means append, including an empty destination. Proposals do not mutate.
    pub fn propose(&self, scope: EntityId, target: S, before: Option<K>) -> Option<DropProposal<S, K>> {
        self.accepts(scope).then(|| DropProposal { drag: self.clone(), target, before })
    }
}

/// Host mutation request. Retains its session so all proposals and preview release
/// share the same completion guard. Call `is_active` before any deferred mutation.
#[derive(Clone, Debug)]
pub struct DropProposal<S, K> {
    drag: KeyedDrag<S, K>,
    target: S,
    before: Option<K>,
}

impl<S: Clone, K: Clone> DropProposal<S, K> {
    pub fn source(&self) -> &S {
        self.drag.source()
    }
    pub fn target(&self) -> &S {
        &self.target
    }
    pub fn keys(&self) -> &[K] {
        self.drag.keys()
    }
    pub fn before(&self) -> Option<&K> {
        self.before.as_ref()
    }
    /// Whether this proposal belongs to this exact captured session.
    pub fn is_from(&self, drag: &KeyedDrag<S, K>) -> bool {
        Rc::ptr_eq(&self.drag.phase, &drag.phase)
    }

    pub fn is_active(&self) -> bool {
        self.drag.phase.get() == Phase::Active
    }

    /// Report host validation failure without moving any data.
    pub fn rejected(&self, error: impl Into<SharedString>) -> Vec<DragDropEvent<S, K>> {
        let Some(ended) = self.drag.end(DragEndReason::Rejected) else {
            return Vec::new();
        };
        vec![
            DragDropEvent::DropRejected {
                source: self.drag.source.clone(),
                target: self.target.clone(),
                keys: self.drag.keys.clone(),
                before: self.before.clone(),
                error: error.into(),
            },
            ended,
        ]
    }
}

impl<S: Clone + Eq, K: Clone> DropProposal<S, K> {
    /// Report a completed host transaction. `ordered_keys` are the moved keys in
    /// final destination order. A no-op reports Dropped/DragEnded only. This method
    /// never applies snapshots, selection, focus, or domain changes itself.
    pub fn committed(&self, changed: bool, ordered_keys: Vec<K>) -> Vec<DragDropEvent<S, K>> {
        let same_list = self.drag.source == self.target;
        let reason = if !changed {
            DragEndReason::Unchanged
        } else if same_list {
            DragEndReason::Reordered
        } else {
            DragEndReason::Transferred
        };
        let Some(ended) = self.drag.end(reason) else {
            return Vec::new();
        };
        let mut events = Vec::new();
        if changed {
            if same_list {
                events.push(DragDropEvent::ItemsReordered {
                    target: self.target.clone(),
                    keys: ordered_keys.clone(),
                    before: self.before.clone(),
                });
            } else {
                events.push(DragDropEvent::ItemsRemoved {
                    source: self.drag.source.clone(),
                    target: self.target.clone(),
                    keys: ordered_keys.clone(),
                });
                events.push(DragDropEvent::ItemsAdded {
                    source: self.drag.source.clone(),
                    target: self.target.clone(),
                    keys: ordered_keys.clone(),
                    before: self.before.clone(),
                });
            }
        }
        events.push(DragDropEvent::Dropped {
            source: self.drag.source.clone(),
            target: self.target.clone(),
            keys: ordered_keys,
            before: self.before.clone(),
            changed,
        });
        events.push(ended);
        events
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "test-support"))]
mod dispatch_tests;

/// Shared linear edge response in content coordinates; positive moves forward.
pub(crate) fn edge_scroll_delta(position: f32, extent: f32, elapsed: std::time::Duration, max_speed: f32) -> f32 {
    let edge = 32.0_f32.min(extent / 4.0);
    if edge <= 0.0 {
        return 0.0;
    }
    let leading = ((edge - position) / edge).clamp(0.0, 1.0);
    let trailing = ((edge - (extent - position)) / edge).clamp(0.0, 1.0);
    (trailing - leading) * max_speed * elapsed.as_secs_f32().min(0.05)
}
