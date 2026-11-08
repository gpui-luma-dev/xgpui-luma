//! Small preserving-update fixture in Controls → TreeView.
use std::sync::Arc;

use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::tree_view::{TreeNode, TreeView};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, prelude::*};

use super::super::event_stream::ControlEventStream;

#[derive(Clone, Copy)]
enum Update {
    Reorder,
    MoveGuide,
    RemoveSelected,
    Invalid,
    Reset,
}

pub(super) struct StateUpdates {
    look: Arc<ShadcnLook>,
    tree: TreeView<SharedString>,
    buttons: Vec<Entity<Button>>,
    events: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl StateUpdates {
    pub(super) fn tree(&self) -> TreeView<SharedString> {
        self.tree.clone()
    }

    pub(super) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let tree = shadcn::TreeView::new("tree-state-updates")
            .look(&look)
            .require_focus_for_scroll(true)
            .wrap_navigation(false)
            .try_items(sample_items())
            .expect("state-update fixture IDs are unique")
            .spawn(cx);
        let events = cx.new(|cx| ControlEventStream::new(
            cx, look.clone(), "tree-state-update-events",
            "Select a node and change expansion, then reorder or move Guide. Invalid updates must leave the tree unchanged.",
        ));
        let subscriptions = vec![cx.subscribe(&tree, {
            let events = events.clone();
            move |_, _, event, cx| {
                cx.notify();
                if let Some(line) = super::format_tree_view_event(event) {
                    events.update(cx, |events, cx| events.append_line(&line, cx));
                }
            }
        })];
        let buttons = [
            ("reorder", "Reorder", Update::Reorder),
            ("move", "Move Guide", Update::MoveGuide),
            ("remove", "Remove selected", Update::RemoveSelected),
            ("invalid", "Try duplicate ID", Update::Invalid),
            ("reset", "Reset", Update::Reset),
        ]
        .into_iter()
        .map(|(id, label, update)| {
            shadcn::Button::new(format!("tree-state-{id}"))
                .look(&look)
                .secondary()
                .label(label)
                .with_template_modifier(move |root, _| root.debug_selector(move || format!("tree-state-{id}")))
                .spawn(cx)
                .on_click(cx, move |this, cx| {
                    this.apply(update, cx);
                })
        })
        .collect();
        Self { look, tree, buttons, events, _subscriptions: subscriptions }
    }

    fn sync_actions(&self, cx: &mut Context<Self>) {
        let tree = self.tree.read(cx);
        let items = tree.items();
        let reorder = items.len() > 1 || items.iter().any(|root| root.children.len() > 1);
        let move_guide =
            items.len() > 1 && items.iter().any(|root| root.children.iter().any(|child| child.id == "guide"));
        let remove = !tree.selected_ids().is_empty();
        for (button, enabled) in self.buttons.iter().zip([reorder, move_guide, remove, true, true]) {
            button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        }
    }
    fn apply(&mut self, update: Update, cx: &mut Context<Self>) {
        let outcome = self.tree.update(cx, |tree, cx| {
            if matches!(update, Update::Reset) {
                tree.set_items(sample_items(), cx);
                return "Reset data, selection, and expansion".to_owned();
            }
            let mut items = tree.items().to_vec();
            match update {
                Update::Reorder => {
                    items.reverse();
                    for root in &mut items {
                        root.children.reverse();
                    }
                }
                Update::MoveGuide => {
                    let Some(source) =
                        items.iter().position(|node| node.children.iter().any(|child| child.id == "guide"))
                    else {
                        return "Guide is missing; reset to restore it".to_owned();
                    };
                    let Some(target) = items.iter().position(|node| node.id != items[source].id) else {
                        return "A second folder is required; reset to restore it".to_owned();
                    };
                    if let Some(index) = items[source].children.iter().position(|node| node.id == "guide") {
                        let guide = items[source].children.remove(index);
                        items[target].children.push(guide);
                    }
                }
                Update::RemoveSelected => {
                    items.retain(|node| !tree.selected_ids().contains(&node.id));
                    for root in &mut items {
                        root.children.retain(|node| !tree.selected_ids().contains(&node.id));
                    }
                }
                Update::Invalid => {
                    let duplicate = || TreeNode::new("duplicate", "Duplicate", SharedString::from("duplicate"));
                    items.push(duplicate().children([duplicate()]));
                }
                Update::Reset => {}
            }
            match tree.replace_items(items, cx) {
                Ok(()) => "Applied preserving update".to_owned(),
                Err(error) => format!("Rejected: {error}; tree unchanged"),
            }
        });
        self.events.update(cx, |events, cx| events.append_line(&outcome, cx));
        cx.notify();
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tree.update(cx, |tree, cx| {
            tree.set_template(look.tree_view_template(), cx);
            tree.set_scrollbar_template(look.scrollbar_template(), cx);
        });
        for button in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for StateUpdates {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_actions(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(div().text_color(chrome.title_text).child("State-preserving updates"))
                .child(div().flex().flex_wrap().gap(px(8.0)).children(self.buttons.iter().cloned()))
                .child(
                    shadcn::Frame::new("tree-state-updates-frame")
                        .look(&self.look)
                        .w(px(360.0))
                        .h(px(220.0))
                        .overflow_hidden()
                        .border_1()
                        .rounded(px(8.0))
                        .child(self.tree.clone())
                        .render(cx),
                )
                .child(self.events.clone())
        })
    }
}

fn sample_items() -> Vec<TreeNode<SharedString>> {
    let node = |id: &'static str, label: &'static str| TreeNode::new(id, label, SharedString::from(id));
    vec![
        node("documents", "Documents")
            .expanded(true)
            .children([node("readme", "Readme"), node("guide", "Guide")]),
        node("archive", "Archive").branch(true).expanded(true),
    ]
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};

    fn click(cx: &mut VisualTestContext, id: &'static str) {
        let bounds = cx.debug_bounds(id).unwrap_or_else(|| panic!("missing button {id}"));
        cx.simulate_click(bounds.center(), Default::default());
        cx.run_until_parked();
    }

    #[test]
    fn tree_view_state_fixture_preserves_and_rejects_updates_through_sdk_buttons() {
        let mut app = TestAppContext::single();
        let (fixture, cx) = app.add_window_view(|_, cx| StateUpdates::new(Arc::new(ShadcnLook::built_in()), cx));
        cx.run_until_parked();
        cx.update(|window, app| assert!(fixture.read(app).buttons[2].read(app).render_model(window).state.disabled));
        let tree = cx.update(|_, app| fixture.read(app).tree.clone());
        tree.update(cx, |tree, cx| {
            tree.select_node_by_id("guide", cx);
            tree.collapse("archive", cx);
        });
        click(cx, "tree-state-reorder");
        cx.update(|_, app| {
            let tree = tree.read(app);
            assert_eq!(tree.items()[0].id.as_ref(), "archive");
            assert!(tree.selected_ids().contains("guide"));
            assert!(!tree.is_expanded(&"archive".into()));
        });
        click(cx, "tree-state-move");
        cx.update(|_, app| {
            let tree = tree.read(app);
            assert_eq!(tree.items()[0].children[0].id.as_ref(), "guide");
            assert!(tree.selected_ids().contains("guide"));
            assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("archive"));
            assert!(!tree.is_expanded(&"archive".into()));
        });
        click(cx, "tree-state-invalid");
        cx.update(|_, app| {
            let tree = tree.read(app);
            assert_eq!(tree.items().len(), 2);
            assert_eq!(tree.items()[0].children[0].id.as_ref(), "guide");
            assert!(tree.selected_ids().contains("guide"));
        });
        click(cx, "tree-state-remove");
        cx.update(|_, app| {
            assert!(tree.read(app).items()[0].children.is_empty());
            assert!(tree.read(app).selected_ids().is_empty());
        });
        cx.update(|window, app| {
            assert!(fixture.read(app).buttons[1].read(app).render_model(window).state.disabled);
            assert!(fixture.read(app).buttons[2].read(app).render_model(window).state.disabled);
        });
        click(cx, "tree-state-reset");
        cx.update(|window, app| assert!(!fixture.read(app).buttons[1].read(app).render_model(window).state.disabled));
        cx.update(|_, app| {
            let tree = tree.read(app);
            assert_eq!(tree.items()[0].id.as_ref(), "documents");
            assert_eq!(tree.items()[0].children.len(), 2);
            assert!(tree.is_expanded(&"archive".into()));
            assert!(tree.selected_ids().is_empty());
            assert!(tree.active_node_id().is_none());
        });
    }
}
