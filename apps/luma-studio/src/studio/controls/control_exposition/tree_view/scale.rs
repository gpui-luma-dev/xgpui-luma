//! Manual scale diagnostics; all interactive chrome uses SDK controls.
use xgpui_luma::controls::tree_view::TreeViewEventExt;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use gpui::{
    AnyElement, App, Context, Div, Entity, Render, SharedString, Stateful, Subscription, Window, div, prelude::*, px,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::{
    button::{Button},
    tree_view::{
        FlatTreeNode, TreeNode, TreeView, TreeViewEvent, TreeViewRenderModel, TreeViewTemplate,
        TreeViewTemplateHandlers,
    },
};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, prelude::*};
use super::ControlEventStream;

#[derive(Default)]
struct Counts {
    rows: HashSet<SharedString>,
    calls: usize,
}
struct CountedTemplate {
    base: Arc<dyn TreeViewTemplate<()>>,
    counts: Arc<Mutex<Counts>>,
}
impl TreeViewTemplate<()> for CountedTemplate {
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.base.render(model, body, window, cx)
    }
    fn render_node(
        &self,
        node: &FlatTreeNode<'_, ()>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mut counts = self.counts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        counts.calls += 1;
        counts.rows.insert(node.id.clone());
        drop(counts);
        self.base.render_node(node, handlers, window, cx)
    }
}
pub(super) struct Scale {
    look: Arc<ShadcnLook>,
    tree: TreeView<()>,
    buttons: Vec<(&'static str, Entity<Button>)>,
    counts: Arc<Mutex<Counts>>,
    sample: Option<(usize, usize)>,
    events: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}
impl Scale {
    pub(super) fn tree(&self) -> TreeView<()> {
        self.tree.clone()
    }

    pub(super) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let counts = Arc::new(Mutex::new(Counts::default()));
        let tree = shadcn::TreeView::new("tree-scale")
            .look(&look)
            .items(items())
            .template(Arc::new(CountedTemplate { base: look.tree_view_template(), counts: counts.clone() }))
            .expand_on_row_click(false)
            .wrap_navigation(false)
            .require_focus_for_scroll(true)
            .spawn(cx);
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "tree-scale-events",
                "Jump, update, filter or collapse the tree, then Sample counts. Counts accumulate until Reset counts.",
            )
        });
        let subscriptions = {
            tree.clone().on_event(cx, |this: &mut Self, event, cx| {
                if matches!(
                    event,
                    TreeViewEvent::NodeExpanded { .. }
                        | TreeViewEvent::NodeCollapsed { .. }
                        | TreeViewEvent::SelectionChanged { .. }
                ) {
                    cx.notify();
                }
                if let TreeViewEvent::NodeCollapsed { node_id, .. } = event {
                    this.log(format!("Closed: {node_id}"), cx);
                }
            });
            Vec::new()
        };
        let buttons = [
            ("first", "First"),
            ("middle", "Middle"),
            ("deep", "Deep leaf"),
            ("reverse", "Reverse siblings"),
            ("folder", "Toggle folder 50"),
            ("filter", "Filter folder 99"),
            ("clear-filter", "Clear filter"),
            ("reset", "Reset tree"),
            ("sample", "Sample counts"),
            ("reset-counts", "Reset counts"),
        ]
        .into_iter()
        .map(|(id, label)| {
            let button = shadcn::Button::new(format!("tree-scale-{id}"))
                .look(&look)
                .secondary()
                .label(label)
                .with_template_modifier(move |root, _| root.debug_selector(move || format!("tree-scale-{id}")))
                .spawn(cx)
                .on_click(cx, move |this: &mut Self, cx| {
                    this.act(id, cx);
                });
            (id, button)
        })
        .collect();
        Self { look, tree, buttons, counts, sample: None, events, _subscriptions: subscriptions }
    }
    fn action_enabled(&self, action: &str, cx: &App) -> bool {
        let tree = self.tree.read(cx);
        match action {
            "first" => tree.visible_ids().contains(&"folder-0".into()),
            "middle" => tree.visible_ids().contains(&"file-50-50".into()),
            // Reveal may open collapsed ancestors, but cannot bypass a filter.
            "deep" | "folder" | "filter" => !tree.has_projection(),
            "clear-filter" => tree.has_projection(),
            _ => true,
        }
    }
    fn sync_actions(&self, cx: &mut Context<Self>) {
        for (id, button) in &self.buttons {
            let enabled = self.action_enabled(id, cx);
            button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        }
    }
    fn log(&mut self, line: String, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(&line, cx));
    }
    fn act(&mut self, action: &str, cx: &mut Context<Self>) {
        if !self.action_enabled(action, cx) {
            return;
        }
        if action == "sample" {
            let counts = self.counts.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let sample = (counts.rows.len(), counts.calls);
            drop(counts);
            self.sample = Some(sample);
            self.log(
                format!("Sample: {} unique rows built; {} row-template calls since reset", sample.0, sample.1),
                cx,
            );
        } else if action == "reset-counts" {
            *self.counts.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Counts::default();
            self.sample = None;
        } else {
            let result = self.tree.update(cx, |tree, cx| {
                match action {
                    "first" => {
                        if !tree.scroll_to("folder-0", cx) {
                            return "First folder is hidden; clear the filter";
                        }
                    }
                    "middle" => {
                        if !tree.scroll_to_center("file-50-50", cx) {
                            return "Middle document is hidden; clear the filter and open folder 50";
                        }
                    }
                    "deep" => {
                        if !tree.reveal_node("deep-file", cx) {
                            return "Deep leaf is filtered out; clear the filter";
                        }
                    }
                    "reverse" => {
                        let mut data = tree.items().to_vec();
                        reverse(&mut data);
                        if tree.replace_items(data, cx).is_err() {
                            return "Update rejected";
                        }
                    }
                    "folder" => {
                        tree.toggle_expand("folder-50", cx);
                    }
                    "filter" => tree.set_filter(|node| node.id.starts_with("file-99-"), cx),
                    "clear-filter" => tree.clear_filter(cx),
                    "reset" => {
                        tree.clear_filter(cx);
                        tree.set_items(items(), cx);
                    }
                    _ => {}
                }
                "Applied"
            });
            self.log(format!("{action}: {result}"), cx);
        }
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let counts = self.counts.clone();
        self.tree.update(cx, |tree, cx| {
            tree.set_template(Arc::new(CountedTemplate { base: look.tree_view_template(), counts }), cx);
            tree.set_scrollbar_template(look.scrollbar_template(), cx);
        });
        for (_, button) in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for Scale {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_actions(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let tree = self.tree.read(cx);
            let sample = self.sample.map_or_else(
                || "Counts not sampled".to_owned(),
                |(rows, calls)| format!("Sampled since reset: {rows} unique rows built · {calls} row-template calls"),
            );
            div().w_full().flex().flex_col().gap(px(12.0))
                .child(div().text_color(chrome.title_text).child("Large tree — 10,117 nodes"))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child("100 folders × 100 documents, plus 16 nested folders and a deep leaf. Click the tree to focus before scrolling. Reverse siblings preserves IDs and selection. This example tests scrolling and updates."))
                .child(div().flex().flex_wrap().gap(px(8.0)).children(self.buttons.iter().map(|(_,button)|button.clone())))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(format!("10,117 loaded · {} displayed · {} selected",tree.visible_ids().len(),tree.selected_ids().len())))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(sample))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child("Unique rows counts distinct IDs rendered since reset; template calls include repeated builds. Sample after each action. These counters measure row construction, not total data traversal or native frame rate."))
                .child(shadcn::Frame::new("tree-scale-frame").look(&self.look).w(px(420.0)).h(px(280.0)).overflow_hidden().border_1().rounded(px(8.0)).child(self.tree.clone()).render(cx))
                .child(self.events.clone())
        })
    }
}
fn items() -> Vec<TreeNode<()>> {
    let mut rows: Vec<_> = (0..100)
        .map(|folder| {
            TreeNode::new(format!("folder-{folder}"), format!("Folder {folder:02}"), ())
                .expanded(true)
                .children((0..100).map(|file| {
                    TreeNode::new(format!("file-{folder}-{file}"), format!("Document {folder:02}/{file:02}"), ())
                }))
        })
        .collect();
    let mut deep = TreeNode::new("deep-file", "Deep document", ());
    for depth in (0..16).rev() {
        deep = TreeNode::new(format!("deep-{depth}"), format!("Level {depth}"), ()).expanded(true).children([deep]);
    }
    rows.push(deep);
    rows
}
fn reverse(nodes: &mut [TreeNode<()>]) {
    nodes.reverse();
    for node in nodes {
        reverse(&mut node.children);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    #[test]
    fn tree_view_scale_fixture_buttons_preserve_state_and_report_bounded_construction() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Scale::new(Arc::new(ShadcnLook::built_in()), cx)
        });
        cx.run_until_parked();
        let click = |cx: &mut gpui::VisualTestContext, id: &'static str| {
            let point = cx.debug_bounds(id).unwrap().center();
            cx.simulate_click(point, Default::default());
            cx.run_until_parked();
        };
        let enabled = |cx: &mut gpui::VisualTestContext, id: &str| {
            cx.update(|window, app| {
                let button = &view.read(app).buttons.iter().find(|(key, _)| *key == id).unwrap().1;
                !button.read(app).render_model(window).state.disabled
            })
        };
        assert!(!enabled(cx, "clear-filter"));
        click(cx, "tree-scale-sample");
        cx.update(|_, app| {
            let sample = view.read(app).sample.unwrap();
            eprintln!("Studio large tree initial: {} unique / {} calls", sample.0, sample.1);
            assert!(sample.0 > 0 && sample.0 < 200);
        });
        click(cx, "tree-scale-middle");
        for _ in 0..4 {
            cx.update(|window, app| window.simulate_next_frame(app));
            cx.run_until_parked();
        }
        assert!(cx.debug_bounds("tree-scale/node/file-50-50").is_some());
        let tree = cx.update(|_, app| view.read(app).tree.clone());
        tree.update(cx, |tree, cx| tree.select_node_by_id("file-50-50", cx));
        click(cx, "tree-scale-reverse");
        click(cx, "tree-scale-filter");
        for id in ["first", "middle", "deep", "folder", "filter"] {
            assert!(!enabled(cx, id), "{id}");
        }
        assert!(enabled(cx, "clear-filter"));
        // Disabled SDK buttons must not silently mutate a hidden folder.
        click(cx, "tree-scale-folder");
        cx.update(|_, app| assert!(tree.read(app).is_expanded(&"folder-50".into())));
        cx.update(|_, app| {
            assert_eq!(tree.read(app).visible_ids().len(), 101);
            assert!(tree.read(app).selected_ids().contains("file-50-50"));
        });
        click(cx, "tree-scale-clear-filter");
        for id in ["first", "middle", "deep", "folder", "filter"] {
            assert!(enabled(cx, id), "{id}");
        }
        assert!(!enabled(cx, "clear-filter"));
        click(cx, "tree-scale-folder");
        assert!(!enabled(cx, "middle"));
        assert!(enabled(cx, "folder"));
        assert!(enabled(cx, "deep"));
        cx.update(|_, app| assert!(!tree.read(app).is_expanded(&"folder-50".into())));
        click(cx, "tree-scale-reset-counts");
        click(cx, "tree-scale-deep");
        for _ in 0..4 {
            cx.update(|window, app| window.simulate_next_frame(app));
            cx.run_until_parked();
        }
        assert!(cx.debug_bounds("tree-scale/node/deep-file").is_some());
        click(cx, "tree-scale-sample");
        cx.update(|_, app| {
            let sample = view.read(app).sample.unwrap();
            assert!(sample.0 > 0 && sample.0 < 200);
            assert!(sample.1 >= sample.0);
        });
        click(cx, "tree-scale-reset");
        cx.update(|_, app| assert_eq!(tree.read(app).visible_ids().len(), 10117));
    }
}
