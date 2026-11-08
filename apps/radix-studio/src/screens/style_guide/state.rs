//! Persistent guide controls and subscriptions, created on the first visit.
use xgpui_luma::controls::tabs::TabsEventExt;
use std::sync::Arc;
use gpui::{AnyElement, App, Context, Entity, Window, prelude::*};
use xgpui_luma::controls::{
    tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode},
    tree_view::TreeView,
};
use xgpui_luma_look_radix::Look;
use super::{PreviewTabs, TabsExamples};

pub struct State {
    tabs: PreviewTabs,
    tree: TreeView<()>,
    disabled_tree: TreeView<()>,
}

impl State {
    pub fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let mut navigation = |name| preview_navigation(name, look, cx);
        let tabs = PreviewTabs {
            avatars: navigation("avatars"),
            badges: navigation("badges"),
            buttons: navigation("buttons"),
            checkboxes: navigation("checkboxes"),
            radios: navigation("radios"),
            switches: navigation("switches"),
            textfields: navigation("textfields"),
            textareas: navigation("textareas"),
            sliders: navigation("sliders"),
            progress: navigation("progress"),
            autocomplete: navigation("autocomplete"),
            combobox: navigation("combobox"),
            search_selector: navigation("search-selector"),
            selection_panel: navigation("selection-panel"),
            scrollbar: navigation("scrollbar"),
            table: navigation("table"),
            split_button: navigation("split-button"),
            examples: TabsExamples::spawn(look, cx),
            tooltips: super::tooltips::TooltipExamples::new(look, cx),
            collections: super::collections::Previews::new(look, cx),
            navigation: super::navigation_previews::Previews::new(look, cx),
            panels: super::panel_previews::Previews::new(look, cx),
        };
        let tree = super::super::shared::tree_view::spawn("guide-tree", look, true, cx);
        let disabled_tree = super::super::shared::tree_view::spawn("guide-tree-disabled", look, false, cx);
        for tree in [&tree, &disabled_tree] {
            tree.update(cx, |tree, cx| {
                tree.set_wheel_scroll_policy(xgpui_luma::interaction::WheelScrollPolicy::PassThrough, cx);
            });
        }
        Self { tabs, tree, disabled_tree }
    }

    pub fn render(&self, look: &Arc<Look>, window: &mut Window, cx: &mut App) -> AnyElement {
        super::page(look, self.tabs.clone(), self.tree.clone(), self.disabled_tree.clone(), window, cx)
    }
}

pub(super) fn preview_navigation<M: 'static>(name: &str, look: &Look, cx: &mut Context<M>) -> Entity<Tabs> {
    let tabs = xgpui_luma_look_radix::Tabs::new(format!("radix-studio-{name}-preview-tabs"))
        .look(look)
        .items([
            TabsItem::new("template-preview").label("Template Preview"),
            TabsItem::new("colors").label("Colors"),
            TabsItem::new("all-sizes").label(match name {
                "pager" => "Layouts",
                "dock-splitter" | "split-view" => "Geometry",
                "sidebar" => "Presentations",
                _ => "All Sizes",
            }),
        ])
        .active("template-preview")
        .width_mode(TabsWidthMode::Intrinsic)
        .with_template_modifier(|root, _| root.w_full())
        .spawn(cx);
    tabs.clone().on_event(cx, |_, event: &TabsEvent, cx| {
        if matches!(event, TabsEvent::Change { .. }) {
            cx.notify();
        }
    });
    tabs
}
