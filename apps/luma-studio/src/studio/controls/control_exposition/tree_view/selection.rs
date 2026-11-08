//! Selection and range input in Controls → TreeView.
use xgpui_luma::controls::tree_view::TreeViewEventExt;
use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::selector::SelectorEventExt;
use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::sync::Arc;
use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::{
    button::{Button},
    checkbox::{Checkbox},
    selector::{Selector, SelectorEvent, SelectorItem},
    textfield::{TextField, TextFieldEvent},
    tree_view::{TreeNode, TreeView, TreeViewEvent, TreeViewSelectionMode, TreeViewSelectionPolicy},
};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, prelude::*};
use super::ControlEventStream;

pub(super) struct Selection {
    look: Arc<ShadcnLook>,
    tree: TreeView<()>,
    mode: Entity<Selector>,
    controls_mode: Option<TreeViewSelectionMode>,
    sort: Entity<Selector>,
    search: TextField,
    toggle_off: Checkbox,
    follows_active: Checkbox,
    buttons: Vec<Entity<Button>>,
    events: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}
impl Selection {
    pub(super) fn tree(&self) -> TreeView<()> {
        self.tree.clone()
    }

    pub(super) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let leaf = |id, label| TreeNode::new(id, label, ());
        let tree = shadcn::TreeView::new("tree-selection")
            .look(&look)
            .items([
                leaf("docs", "Documents").expanded(true).children([
                    leaf("guide", "Guide"),
                    leaf("draft", "Draft"),
                    leaf("locked", "Locked (disabled)").enabled(false),
                    leaf("readme", "Readme"),
                ]),
                leaf("media", "Media").expanded(true).children([leaf("video", "Video"), leaf("photo", "Photo")]),
                leaf("notes", "Notes"),
            ])
            .selection_mode(TreeViewSelectionMode::Extended)
            .expand_on_row_click(false)
            .wrap_navigation(false)
            .require_focus_for_scroll(true)
            .spawn(cx);
        let mode = shadcn::Selector::new("tree-selection-mode")
            .look(&look)
            .label("Mode")
            .items([
                SelectorItem::new("extended").label("Extended"),
                SelectorItem::new("single").label("Single"),
                SelectorItem::new("multiple").label("Multiple"),
                SelectorItem::new("none").label("None"),
            ])
            .selected_id("extended")
            .spawn(cx);
        let sort = shadcn::Selector::new("tree-selection-sort")
            .look(&look)
            .label("Order")
            .items([
                SelectorItem::new("source").label("Original"),
                SelectorItem::new("az").label("A–Z"),
                SelectorItem::new("za").label("Z–A"),
            ])
            .selected_id("source")
            .spawn(cx);
        let search = shadcn::TextField::new("tree-selection-search")
            .look(&look)
            .full_width(true)
            .placeholder("Filter names…")
            .spawn(cx);
        let toggle_off = shadcn::Checkbox::new("tree-selection-toggle")
            .look(&look)
            .label("Plain click can unselect")
            .spawn(cx);
        let follows_active = shadcn::Checkbox::new("tree-selection-follow")
            .look(&look)
            .label("Select on navigation (Single)")
            .spawn(cx);
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "tree-selection-events",
                "Selection, active row, activation, and policy changes appear here.",
            )
        });
        let subscriptions = {
            let retained_subscriptions = Vec::new();
            tree.clone().on_event(cx, |this: &mut Self, event, cx| {
                let line = match event {
                    TreeViewEvent::SelectionChanged { selected_ids } => {
                        let mut ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
                        ids.sort();
                        Some(format!(
                            "Selected: {}",
                            if ids.is_empty() {
                                "none".to_owned()
                            } else {
                                ids.join(", ")
                            }
                        ))
                    }
                    TreeViewEvent::ActiveNodeChanged { node_id } => {
                        Some(format!("Active: {}", node_id.as_deref().unwrap_or("none")))
                    }
                    TreeViewEvent::NodeActivated { node_id, .. } => Some(format!("Activated: {node_id}")),
                    TreeViewEvent::SelectionPolicyChanged { policy } => Some(format!(
                        "Policy: {:?}; unselect {}; follow {}",
                        policy.mode, policy.toggle_off, policy.selection_follows_active
                    )),
                    _ => None,
                };
                if let Some(line) = line {
                    this.events.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
                cx.notify();
            });
            mode.clone().on_event(cx, |this: &mut Self, event, cx| {
                if let SelectorEvent::Change { item_id, .. } = event {
                    this.change_policy(cx, |policy| {
                        policy.mode = match item_id.as_ref() {
                            "single" => TreeViewSelectionMode::Single,
                            "multiple" => TreeViewSelectionMode::Multiple,
                            "none" => TreeViewSelectionMode::None,
                            _ => TreeViewSelectionMode::Extended,
                        }
                    });
                }
            });
            sort.clone().on_event(cx, |this: &mut Self, event, cx| {
                if let SelectorEvent::Change { item_id, .. } = event {
                    this.tree.update(cx, |tree, cx| match item_id.as_ref() {
                        "az" => tree.set_sort(|a, b| a.label.cmp(&b.label), cx),
                        "za" => tree.set_sort(|a, b| b.label.cmp(&a.label), cx),
                        _ => tree.clear_sort(cx),
                    });
                    cx.notify();
                }
            });
            search.clone().on_event(cx, |this: &mut Self, event, cx| {
                if let TextFieldEvent::Change { value } = event {
                    let query = value.trim().to_lowercase();
                    this.tree.update(cx, |tree, cx| {
                        if query.is_empty() {
                            tree.clear_filter(cx);
                        } else {
                            tree.set_filter(move |node| node.label.to_lowercase().contains(&query), cx);
                        }
                    });
                    cx.notify();
                }
            });
            toggle_off.clone().on_value_changed(cx, |this: &mut Self, checked, cx| {
                this.change_policy(cx, |policy| policy.toggle_off = checked);
            });
            follows_active.clone().on_value_changed(cx, |this: &mut Self, checked, cx| {
                this.change_policy(cx, |policy| policy.selection_follows_active = checked);
            });
            retained_subscriptions
        };
        let buttons = [("all", "Select visible"), ("clear", "Clear")]
            .into_iter()
            .map(|(id, label)| {
                shadcn::Button::new(format!("tree-selection-{id}"))
                    .look(&look)
                    .secondary()
                    .label(label)
                    .with_template_modifier(move |root, _| root.debug_selector(move || format!("tree-selection-{id}")))
                    .spawn(cx)
                    .on_click(cx, move |this: &mut Self, cx| {
                        this.tree.update(cx, |tree, cx| {
                            if id == "all" {
                                tree.select_all(cx);
                            } else {
                                tree.clear_selection(cx);
                            }
                        });
                    })
            })
            .collect();
        Self {
            look,
            tree,
            mode,
            controls_mode: None,
            sort,
            search,
            toggle_off,
            follows_active,
            buttons,
            events,
            _subscriptions: subscriptions,
        }
    }
    fn sync_actions(&mut self, cx: &mut Context<Self>) {
        let tree = self.tree.read(cx);
        let mode = tree.selection_policy().mode;
        let visible = tree.visible_ids();
        fn has_unselected(
            nodes: &[TreeNode<()>],
            visible: &[gpui::SharedString],
            selected: &std::collections::HashSet<gpui::SharedString>,
        ) -> bool {
            nodes.iter().any(|node| {
                (node.enabled && visible.contains(&node.id) && !selected.contains(&node.id))
                    || has_unselected(&node.children, visible, selected)
            })
        }
        let all = matches!(mode, TreeViewSelectionMode::Multiple | TreeViewSelectionMode::Extended)
            && has_unselected(tree.items(), &visible, tree.selected_ids());
        let clear = !tree.selected_ids().is_empty();
        for (button, enabled) in self.buttons.iter().zip([all, clear]) {
            button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        }
        if self.controls_mode != Some(mode) {
            self.toggle_off.update(cx, |control, cx| {
                control.set_enabled(matches!(mode, TreeViewSelectionMode::Single | TreeViewSelectionMode::Extended), cx)
            });
            self.follows_active
                .update(cx, |control, cx| control.set_enabled(mode == TreeViewSelectionMode::Single, cx));
            self.controls_mode = Some(mode);
        }
    }
    fn change_policy(&mut self, cx: &mut Context<Self>, change: impl FnOnce(&mut TreeViewSelectionPolicy)) {
        self.tree.update(cx, |tree, cx| {
            let mut policy = tree.selection_policy();
            change(&mut policy);
            tree.set_selection_policy(policy, cx);
        });
    }
    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tree.update(cx, |tree, cx| {
            tree.set_template(look.tree_view_template(), cx);
            tree.set_scrollbar_template(look.scrollbar_template(), cx);
        });
        self.mode.update(cx, |_, cx| cx.notify());
        self.sort.update(cx, |_, cx| cx.notify());
        self.search.update(cx, |_, cx| cx.notify());
        self.toggle_off.update(cx, |_, cx| cx.notify());
        self.follows_active.update(cx, |_, cx| cx.notify());
        for button in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        self.events.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for Selection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_actions(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let tree = self.tree.read(cx);
            let visible = tree.visible_ids();
            let hidden = tree.selected_ids().iter().filter(|id| !visible.contains(id)).count();
            let status = format!(
                "{} selected · {hidden} hidden · Active: {} · Anchor: {}",
                tree.selected_ids().len(),
                tree.active_node_id().map_or("none", |id| id.as_ref()),
                tree.range_anchor_id().map_or("none", |id| id.as_ref())
            );
            div().w_full().flex().flex_col().gap(px(12.0))
                .child(div().text_color(chrome.title_text).child("Selection and ranges"))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child("Extended: Ctrl/Cmd-click toggles; Shift-click or Shift+Up/Down selects a range. Arrows move focus, Space selects, Enter activates. Ctrl/Cmd+A selects visible rows; add Shift to clear. Select visible applies to Multiple and Extended."))
                .child(div().flex().flex_wrap().gap(px(8.0)).child(self.mode.clone()).child(self.sort.clone()).children(self.buttons.iter().cloned()))
                .child(div().flex().flex_wrap().gap(px(8.0)).child(self.toggle_off.clone()).child(self.follows_active.clone()))
                .child(self.search.clone())
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(status))
                .child(shadcn::Frame::new("tree-selection-frame").look(&self.look).w(px(360.0)).h(px(280.0)).overflow_hidden().border_1().rounded(px(8.0)).child(self.tree.clone()).render(cx))
                .child(self.events.clone())
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use xgpui_luma::controls::checkbox::CheckboxEvent;
    use super::*;
    use gpui::{Modifiers, TestAppContext};
    #[test]
    fn tree_view_selection_fixture_ranges_filtering_and_buttons() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            Selection::new(Arc::new(ShadcnLook::built_in()), cx)
        });
        cx.run_until_parked();
        cx.update(|window, app| {
            assert!(view.read(app).buttons[1].read(app).render_model(window).state.disabled);
            assert!(!view.read(app).buttons[0].read(app).render_model(window).state.disabled);
        });
        let row = cx.debug_bounds("tree-selection/node/guide").unwrap().center();
        cx.simulate_click(row, Modifiers::default());
        cx.simulate_keystrokes("shift-down shift-down");
        let tree = cx.update(|_, app| view.read(app).tree.clone());
        cx.update(|_, app| assert_eq!(tree.read(app).selected_ids().len(), 3));
        cx.update(|_, app| {
            view.read(app)
                .search
                .clone()
                .update(app, |_, cx| cx.emit(TextFieldEvent::Change { value: "Photo".into() }))
        });
        cx.run_until_parked();
        assert!(cx.debug_bounds("tree-selection/node/guide").is_none());
        let button = cx.debug_bounds("tree-selection-all").unwrap().center();
        cx.simulate_click(button, Modifiers::default());
        cx.update(|window, app| {
            assert_eq!(tree.read(app).selected_ids().len(), 5);
            assert!(view.read(app).buttons[0].read(app).render_model(window).state.disabled);
            assert!(!view.read(app).buttons[1].read(app).render_model(window).state.disabled);
        });
        let button = cx.debug_bounds("tree-selection-clear").unwrap().center();
        cx.simulate_click(button, Modifiers::default());
        cx.update(|_, app| assert!(tree.read(app).selected_ids().is_empty()));
        cx.update(|_, app| {
            view.read(app).mode.clone().update(app, |_, cx| {
                cx.emit(SelectorEvent::Change { item_id: "single".into(), label: "Single".into() })
            })
        });
        cx.update(|_, app| {
            view.read(app)
                .toggle_off
                .clone()
                .update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }))
        });
        cx.run_until_parked();
        cx.update(|_, app| {
            let policy = tree.read(app).selection_policy();
            assert_eq!(policy.mode, TreeViewSelectionMode::Single);
            assert!(policy.toggle_off);
        });
        cx.update(|window, app| assert!(view.read(app).buttons[0].read(app).render_model(window).state.disabled));
    }
}
