//! Persistent ListBox drag/drop configuration and host-owned transactions.
use std::{hash::Hash, rc::Rc};
use gpui::{App, Context, Div, EntityId, SharedString, Stateful, WeakEntity, Window, div, prelude::*, px};
use crate::controls::tooltip::TooltipLook;
use crate::infra::drag_drop::{DragDropEvent, DropProposal, KeyedDrag, bind_drag_source};
use super::{ListBoxAxis, ListBoxEvent, ListBoxScrollHandle};

pub(super) struct ListBoxEventSource<K: 'static> {
    pub motion: crate::motion::DropLayoutMotion<K>,
}
impl<K: 'static> Default for ListBoxEventSource<K> {
    fn default() -> Self {
        Self { motion: Default::default() }
    }
}
impl<K: 'static> gpui::EventEmitter<ListBoxEvent<K>> for ListBoxEventSource<K> {}

type ListDrag<K> = KeyedDrag<WeakEntity<ListBoxEventSource<K>>, K>;
type DropHandler<M, K> = Rc<dyn Fn(&mut M, ListBoxDropProposal<K>, &mut Window, &mut Context<M>)>;

/// Opt-in cooperation between lists with the same key type and scope.
/// The handler validates/prepares host data, applies all changes, then calls
/// `committed` or `rejected`. An unanswered synchronous proposal is rejected.
pub struct ListBoxDragDrop<M: 'static, K: 'static> {
    scope: EntityId,
    on_drop: DropHandler<M, K>,
}
impl<M: 'static, K: 'static> Clone for ListBoxDragDrop<M, K> {
    fn clone(&self) -> Self {
        Self { scope: self.scope, on_drop: self.on_drop.clone() }
    }
}
impl<M: 'static, K: 'static> ListBoxDragDrop<M, K> {
    pub fn new(
        scope: EntityId,
        on_drop: impl Fn(&mut M, ListBoxDropProposal<K>, &mut Window, &mut Context<M>) + 'static,
    ) -> Self {
        Self { scope, on_drop: Rc::new(on_drop) }
    }
}

/// A keyed insertion proposal. Source/target IDs identify persistent SDK lists,
/// not their host entities or render-time DOM elements. The host owns mutation;
/// completion automatically routes lifecycle events to the registered controls.
#[derive(Clone)]
pub struct ListBoxDropProposal<K: 'static> {
    inner: DropProposal<WeakEntity<ListBoxEventSource<K>>, K>,
}
impl<K: Clone + 'static> ListBoxDropProposal<K> {
    pub fn source(&self) -> EntityId {
        self.inner.source().entity_id()
    }
    pub fn target(&self) -> EntityId {
        self.inner.target().entity_id()
    }
    pub fn keys(&self) -> &[K] {
        self.inner.keys()
    }
    pub fn before(&self) -> Option<&K> {
        self.inner.before()
    }
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
    /// Check session and control lifetimes immediately before host mutation.
    /// The host must also validate keys and its application policy against data.
    pub fn validate(&self) -> Result<(), SharedString> {
        if !self.is_active() {
            return Err("Drag session ended".into());
        }
        if self.inner.source().upgrade().is_none() {
            return Err("Source closed".into());
        }
        if self.inner.target().upgrade().is_none() {
            return Err("Target closed".into());
        }
        Ok(())
    }
    /// Report acceptance only after all host data and selection updates commit.
    /// A no-op emits Dropped(changed=false) and DragEnded(Unchanged) only.
    pub fn committed(&self, changed: bool, ordered_keys: Vec<K>, cx: &mut App) {
        deliver_batch(self.inner.committed(changed, ordered_keys), cx);
    }
    /// Reject without mutating host data. Completion is guarded across clones.
    pub fn rejected(&self, error: impl Into<SharedString>, cx: &mut App) {
        deliver_batch(self.inner.rejected(error), cx);
    }
}

pub(super) fn emit_batch<K: Clone + 'static>(
    source: WeakEntity<ListBoxEventSource<K>>,
    events: Vec<ListBoxEvent<K>>,
    cx: &mut App,
) {
    if events.is_empty() {
        return;
    }
    cx.defer(move |cx| {
        let _ = source.update(cx, |_, cx| {
            for event in events {
                cx.emit(event);
            }
        });
    });
}

fn deliver_batch<K: Clone + 'static>(events: Vec<DragDropEvent<WeakEntity<ListBoxEventSource<K>>, K>>, cx: &mut App) {
    if events.is_empty() {
        return;
    }
    // Queue the complete transaction together; every callback observes all host
    // mutations, and source removal precedes target addition and completion.
    cx.defer(move |cx| {
        for event in events {
            let recipient = event.list().clone();
            if let Some(owner) = recipient.upgrade() {
                match &event {
                    DragDropEvent::DragStarted { .. } => owner.read(cx).motion.cancel(),
                    DragDropEvent::ItemsReordered { .. }
                    | DragDropEvent::ItemsAdded { .. }
                    | DragDropEvent::ItemsRemoved { .. } => owner.read(cx).motion.start(),
                    _ => {}
                }
            }
            let mapped = match event {
                DragDropEvent::DragStarted { source, keys } => {
                    DragDropEvent::DragStarted { source: source.entity_id(), keys }
                }
                DragDropEvent::ItemsRemoved { source, target, keys } => {
                    DragDropEvent::ItemsRemoved { source: source.entity_id(), target: target.entity_id(), keys }
                }
                DragDropEvent::ItemsAdded { source, target, keys, before } => {
                    DragDropEvent::ItemsAdded { source: source.entity_id(), target: target.entity_id(), keys, before }
                }
                DragDropEvent::ItemsReordered { target, keys, before } => {
                    DragDropEvent::ItemsReordered { target: target.entity_id(), keys, before }
                }
                DragDropEvent::Dropped { source, target, keys, before, changed } => DragDropEvent::Dropped {
                    source: source.entity_id(),
                    target: target.entity_id(),
                    keys,
                    before,
                    changed,
                },
                DragDropEvent::DropRejected { source, target, keys, before, error } => DragDropEvent::DropRejected {
                    source: source.entity_id(),
                    target: target.entity_id(),
                    keys,
                    before,
                    error,
                },
                DragDropEvent::DragEnded { source, keys, reason } => {
                    DragDropEvent::DragEnded { source: source.entity_id(), keys, reason }
                }
            };
            let _ = recipient.update(cx, |_, cx| cx.emit(ListBoxEvent::DragDrop(mapped)));
        }
    });
}

/// Render-time SDK binding shared by Looks. Obtain through `render_parts`;
/// it holds controls weakly, so retained native previews cannot retain handlers.
#[doc(hidden)]
pub struct ListBoxDragDropBinding<M: 'static, K: 'static> {
    pub(super) source: WeakEntity<ListBoxEventSource<K>>,
    pub(super) config: ListBoxDragDrop<M, K>,
    pub(super) preview_style: Option<Rc<TooltipLook>>,
}
impl<M: 'static, K: 'static> Clone for ListBoxDragDropBinding<M, K> {
    fn clone(&self) -> Self {
        Self { source: self.source.clone(), config: self.config.clone(), preview_style: self.preview_style.clone() }
    }
}
impl<M: 'static, K: Clone + Eq + Hash + 'static> ListBoxDragDropBinding<M, K> {
    /// Paint the floating drag label with a look's tooltip chrome (background,
    /// foreground, padding, radius, typography, shadow). Without a style the
    /// preview is a bare label that inherits nothing and may be unreadable.
    pub fn preview_style(mut self, style: TooltipLook) -> Self {
        self.preview_style = Some(Rc::new(style));
        self
    }

    pub fn bind_row(
        &self,
        surface: Stateful<Div>,
        keys: Option<Vec<K>>,
        label: SharedString,
        cx: &mut Context<M>,
    ) -> Stateful<Div> {
        let Some(drag) = keys.and_then(|keys| KeyedDrag::new(self.config.scope, self.source.clone(), keys)) else {
            return surface;
        };
        let style = self.preview_style.clone();
        bind_drag_source(
            surface.cursor_grab(),
            drag,
            move |_, _, _, cx| cx.new(|_| Preview { label: label.clone(), style: style.clone() }),
            cx,
            |_, event, cx| deliver_batch(vec![event], cx),
        )
    }
    pub fn bind_target(&self, surface: Stateful<Div>, before: Option<K>, cx: &Context<M>) -> Stateful<Div> {
        let scope = self.config.scope;
        let target = self.source.clone();
        let owner = cx.weak_entity();
        let handler = self.config.on_drop.clone();
        surface
            .can_drop(move |value, _, _| value.downcast_ref::<ListDrag<K>>().is_some_and(|drag| drag.accepts(scope)))
            .on_drop(move |drag: &ListDrag<K>, window, cx| {
                cx.stop_propagation();
                if let Some(inner) = drag.propose(scope, target.clone(), before.clone()) {
                    let proposal = ListBoxDropProposal { inner };
                    if let Err(error) = proposal.validate() {
                        proposal.rejected(error, cx);
                        return;
                    }
                    let _ = owner.update(cx, |owner, cx| handler(owner, proposal.clone(), window, cx));
                    if proposal.is_active() {
                        proposal.rejected("owner did not accept the drop", cx);
                    }
                }
            })
    }
    pub fn highlight(&self, surface: Stateful<Div>, color: gpui::Hsla) -> Stateful<Div> {
        let scope = self.config.scope;
        surface.drag_over::<ListDrag<K>>(move |style, drag, _, _| {
            if drag.accepts(scope) {
                style.border_color(color)
            } else {
                style
            }
        })
    }
    pub fn bind_viewport(
        &self,
        surface: Stateful<Div>,
        scroll: &ListBoxScrollHandle<K>,
        axis: ListBoxAxis,
    ) -> Stateful<Div> {
        let scope = self.config.scope;
        scroll.bind_drag_auto_scroll(surface, axis, move |drag: &ListDrag<K>| drag.accepts(scope))
    }
}

struct Preview {
    label: SharedString,
    style: Option<Rc<TooltipLook>>,
}
impl gpui::Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let preview = div().child(self.label.clone());
        let Some(style) = &self.style else {
            return preview;
        };
        preview
            .max_w(px(style.max_width))
            .px(px(style.padding))
            .py(px(style.padding_y))
            .rounded(px(style.radius))
            .shadow(style.shadow.clone())
            .bg(style.background)
            .text_color(style.foreground)
            .text_size(px(style.text_size))
            .line_height(px(style.line_height))
    }
}
