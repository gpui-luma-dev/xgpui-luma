//! Explicit positioning in Controls → TreeView; all interactive chrome uses SDK controls.
use xgpui_luma::controls::tree_view::TreeViewEventExt;
use std::sync::Arc;
use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::{
    button::{Button},
    selector::{Selector, SelectorItem},
    tree_view::{TreeNode, TreeView, TreeViewEvent},
};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, prelude::*};
use super::ControlEventStream;

pub(super) struct Positioning {
    look: Arc<ShadcnLook>,
    tree: TreeView<()>,
    target: Entity<Selector>,
    movement: Entity<Selector>,
    buttons: Vec<Entity<Button>>,
    events: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}
impl Positioning {
    pub(super) fn tree(&self) -> TreeView<()> {
        self.tree.clone()
    }

    pub(super) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let mut items: Vec<_> =
            (0..100).map(|i| TreeNode::new(format!("row-{i}"), format!("Document {i:02}"), ())).collect();
        items.push(TreeNode::new("folder", "Example folder", ()).children([
            TreeNode::new("nested", "Nested folder", ()).children([TreeNode::new("hidden", "Hidden document", ())]),
        ]));
        let tree = shadcn::TreeView::new("tree-positioning")
            .look(&look)
            .items(items)
            .expand_on_row_click(false)
            .wrap_navigation(false)
            .require_focus_for_scroll(true)
            .spawn(cx);
        let target = shadcn::Selector::new("tree-position-target")
            .look(&look)
            .label("Target")
            .items([
                SelectorItem::new("row-0").label("First document"),
                SelectorItem::new("row-50").label("Document 50"),
                SelectorItem::new("row-99").label("Document 99"),
                SelectorItem::new("hidden").label("Hidden document"),
            ])
            .selected_id("row-50")
            .spawn(cx);
        let movement = shadcn::Selector::new("tree-position-movement")
            .look(&look)
            .label("Movement")
            .items([
                SelectorItem::new("show").label("Show"),
                SelectorItem::new("center").label("Center"),
                SelectorItem::new("show-smooth").label("Smooth show"),
                SelectorItem::new("center-smooth").label("Smooth center"),
                SelectorItem::new("reveal").label("Open ancestors and show"),
                SelectorItem::new("reveal-smooth").label("Open ancestors and smooth show"),
            ])
            .selected_id("center-smooth")
            .spawn(cx);
        let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "tree-position-events",
            "Positioning leaves selection and tree focus unchanged. Scroll, use a navigation key, or issue another request to interrupt smooth movement."));
        let mut subscriptions = {
            tree.clone().on_event(cx, |this: &mut Self, event, cx| {
                cx.notify();
                let line = match event {
                    TreeViewEvent::ScrollChanged { top_index } => Some(format!("Top row: {top_index}")),
                    TreeViewEvent::NodeExpanded { node_id, .. } => Some(format!("Opened: {node_id}")),
                    TreeViewEvent::NodeCollapsed { node_id, .. } => Some(format!("Closed: {node_id}")),
                    TreeViewEvent::SelectionChanged { selected_ids } => {
                        let mut ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
                        ids.sort();
                        Some(format!("Selected: {}", ids.join(", ")))
                    }
                    _ => None,
                };
                if let Some(line) = line {
                    this.events.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            });
            Vec::new()
        };
        subscriptions.push(cx.observe(&target, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&movement, |_, _, cx| cx.notify()));
        let buttons = [("go", "Go"), ("collapse", "Close & show folder")]
            .into_iter()
            .map(|(id, label)| {
                shadcn::Button::new(format!("tree-position-{id}"))
                    .look(&look)
                    .secondary()
                    .label(label)
                    .with_template_modifier(move |root, _| root.debug_selector(move || format!("tree-position-{id}")))
                    .spawn(cx)
                    .on_click(cx, move |this: &mut Self, cx| {
                        if id == "go" {
                            this.go(cx);
                        } else {
                            this.close_folder(cx);
                        }
                    })
            })
            .collect();
        Self { look, tree, target, movement, buttons, events, _subscriptions: subscriptions }
    }
    fn sync_actions(&self, cx: &mut Context<Self>) {
        let target = self.target.read(cx).selected_id();
        let reveal = self
            .movement
            .read(cx)
            .selected_id()
            .is_some_and(|id| matches!(id.as_ref(), "reveal" | "reveal-smooth"));
        let enabled = target.is_some_and(|id| reveal || self.tree.read(cx).visible_ids().contains(id));
        self.buttons[0].update(cx, |button, cx| button.set_enabled(enabled, cx));
    }
    fn close_folder(&mut self, cx: &mut Context<Self>) {
        let was_open = self.tree.update(cx, |tree, cx| {
            let was_open = tree.is_expanded(&"folder".into());
            tree.collapse("folder", cx);
            tree.scroll_to("folder", cx);
            was_open
        });
        let status = if was_open { "closed" } else { "already closed" };
        self.events.update(cx, |stream, cx| {
            stream.append_line(&format!("Example folder: {status}; bringing it into view"), cx);
        });
    }
    fn go(&mut self, cx: &mut Context<Self>) {
        let target = self.target.read(cx).selected_id().cloned().unwrap_or_else(|| SharedString::from("row-50"));
        let movement = self.movement.read(cx).selected_id().cloned().unwrap_or_else(|| SharedString::from("show"));
        let accepted = self.tree.update(cx, |tree, cx| match movement.as_ref() {
            "center" => tree.scroll_to_center(target.clone(), cx),
            "show-smooth" => tree.scroll_to_smooth(target.clone(), cx),
            "center-smooth" => tree.scroll_to_center_smooth(target.clone(), cx),
            "reveal" => tree.reveal_node(target.clone(), cx),
            "reveal-smooth" => tree.reveal_node_smooth(target.clone(), cx),
            _ => tree.scroll_to(target.clone(), cx),
        });
        let result = if accepted {
            "requested"
        } else {
            "ignored: target is hidden or missing"
        };
        self.events
            .update(cx, |stream, cx| stream.append_line(&format!("{movement}: {target} · {result}"), cx));
    }
    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tree.update(cx, |tree, cx| {
            tree.set_template(look.tree_view_template(), cx);
            tree.set_scrollbar_template(look.scrollbar_template(), cx);
        });
        self.target.update(cx, |_, cx| cx.notify());
        self.movement.update(cx, |_, cx| cx.notify());
        for button in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        self.events.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for Positioning {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_actions(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            div().w_full().flex().flex_col().gap(px(12.0))
                .child(div().text_color(chrome.title_text).child("Positioning"))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(
                    "Choose a target and movement, then Go. A hidden document needs “Open ancestors”. Close & show folder returns to Example folder and closes it."))
                .child(div().flex().flex_wrap().gap(px(8.0)).child(self.target.clone()).child(self.movement.clone()).children(self.buttons.iter().cloned()))
                .child(shadcn::Frame::new("tree-positioning-frame").look(&self.look).w(px(360.0)).h(px(240.0)).overflow_hidden().border_1().rounded(px(8.0)).child(self.tree.clone()).render(cx))
                .child(self.events.clone())
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};
    fn settle(cx: &mut VisualTestContext) {
        for _ in 0..16 {
            cx.executor().advance_clock(std::time::Duration::from_millis(25));
            cx.update(|window, app| {
                window.simulate_next_frame(app);
            });
            cx.run_until_parked();
        }
    }
    fn click_go(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        let go = cx.debug_bounds("tree-position-go").unwrap().center();
        cx.simulate_click(go, Default::default());
    }

    #[test]
    fn tree_view_positioning_fixture_go_button_centers_and_reveals_without_selecting() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Positioning::new(Arc::new(ShadcnLook::built_in()), cx)
        });
        cx.run_until_parked();
        click_go(cx);
        settle(cx);
        let row = cx.debug_bounds("tree-positioning/node/row-50").unwrap();
        let viewport = cx.debug_bounds("tree-positioning-viewport").unwrap();
        assert!((row.center().y - viewport.center().y).abs() < px(1.0));
        cx.update(|_, app| {
            view.read(app).target.clone().update(app, |selector, cx| {
                selector.set_selected_id("hidden", cx);
            });
            view.read(app).movement.clone().update(app, |selector, cx| {
                selector.set_selected_id("show", cx);
            });
        });
        click_go(cx);
        settle(cx);
        cx.update(|window, app| assert!(view.read(app).buttons[0].read(app).render_model(window).state.disabled));
        cx.update(|_, app| assert!(!view.read(app).tree.read(app).is_expanded(&"folder".into())));
        cx.update(|_, app| {
            view.read(app).movement.clone().update(app, |selector, cx| {
                selector.set_selected_id("reveal-smooth", cx);
            });
        });
        click_go(cx);
        settle(cx);
        assert!(cx.debug_bounds("tree-positioning/node/hidden").is_some());
        cx.update(|window, app| assert!(!view.read(app).buttons[0].read(app).render_model(window).state.disabled));
        cx.update(|_, app| {
            let tree = view.read(app).tree.read(app);
            assert!(tree.is_expanded(&"folder".into()) && tree.is_expanded(&"nested".into()));
            assert!(tree.selected_ids().is_empty());
            assert!(tree.active_node_id().is_none());
        });
    }
    #[test]
    fn tree_view_positioning_close_button_shows_offscreen_folder_and_hides_descendants() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Positioning::new(Arc::new(ShadcnLook::built_in()), cx)
        });
        let tree = cx.update(|_, app| view.read(app).tree.clone());
        tree.update(cx, |tree, cx| tree.set_motion_enabled(false, cx));
        // Exercise an open folder, a closed folder offscreen, and an already
        // closed folder in view through the actual SDK button.
        for (open, offscreen) in [(true, true), (true, false), (false, true), (false, false)] {
            tree.update(cx, |tree, cx| {
                if open {
                    tree.reveal_node("hidden", cx);
                }
                if offscreen {
                    tree.scroll_to("row-0", cx);
                }
            });
            settle(cx);
            if offscreen {
                assert!(cx.debug_bounds("tree-positioning/node/folder").is_none());
            }
            let button = cx.debug_bounds("tree-position-collapse").unwrap().center();
            cx.simulate_click(button, Default::default());
            settle(cx);
            let folder = cx.debug_bounds("tree-positioning/node/folder").unwrap();
            let viewport = cx.debug_bounds("tree-positioning-viewport").unwrap();
            assert!(folder.top() >= viewport.top() && folder.bottom() <= viewport.bottom() + px(1.0));
            assert!(cx.debug_bounds("tree-positioning/node/hidden").is_none());
            cx.update(|_, app| {
                assert!(!tree.read(app).is_expanded(&"folder".into()));
                assert!(!tree.read(app).visible_ids().contains(&"hidden".into()));
                assert!(tree.read(app).selected_ids().is_empty());
            });
        }
    }
}
