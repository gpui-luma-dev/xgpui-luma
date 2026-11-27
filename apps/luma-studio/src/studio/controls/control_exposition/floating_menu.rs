use std::sync::Arc;

use gpui::{App, ClickEvent, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler, render_floating_menu};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::infra::state::MenuPath;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct FloatingMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<FloatingMenuExpositionLeftPane>,
}

struct FloatingMenuExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
}

impl FloatingMenuExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for FloatingMenuExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let menu_look = self.look.floating_menu_theme().style();
            let preview = div()
                .w_full()
                .flex()
                .flex_wrap()
                .items_start()
                .justify_center()
                .gap(px(16.0))
                .child(render_menu_sample("Standard", &menu_look, &default_items(), None))
                .child(render_menu_sample("Hover / active item", &menu_look, &default_items(), Some(MenuPath::Root(1))))
                .child(render_menu_sample("Disabled item", &menu_look, &disabled_items(), None))
                .child(render_menu_sample("Submenu affordance", &menu_look, &submenu_items(), None));

            div()
                .id("controls-doc-floating-menu-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl FloatingMenuControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("floating-menu").expect("floating-menu catalog entry");
        let left_pane = cx.new(|_| FloatingMenuExpositionLeftPane { look: look.clone(), entry });

        Self { look, entry, left_pane }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        cx.notify();
    }
}

impl Render for FloatingMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-floating-menu-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.left_pane.clone())
        })
    }
}

fn render_menu_sample(
    label: &'static str,
    style: &xgpui_luma::style::StyleBinding,
    items: &[MenuItem],
    active_path: Option<MenuPath>,
) -> gpui::AnyElement {
    let id = SharedString::from(format!("controls-doc-floating-menu-{}", sample_id(label)));
    let root_count = items.len();
    let chrome = xgpui_luma::controls::floating_menu::styled_floating_menu_look(style).background;

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(render_floating_menu(
            &id,
            items,
            None,
            active_path,
            style.clone(),
            noop_hovers(root_count),
            noop_clicks(root_count),
        ))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(chrome).opacity(0.72).child(label))
        .into_any_element()
}

fn default_items() -> [MenuItem; 3] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
    ]
}

fn disabled_items() -> [MenuItem; 3] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download).enabled(false),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2),
    ]
}

fn submenu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("inspect").label("Inspect"),
    ]
}

fn sample_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn noop_hovers(count: usize) -> Vec<FloatingMenuHoverHandler> {
    (0..count)
        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as FloatingMenuHoverHandler)
        .collect()
}

fn noop_clicks(count: usize) -> Vec<FloatingMenuClickHandler> {
    (0..count)
        .map(|_| Box::new(|_: &ClickEvent, _: &mut Window, _: &mut App| {}) as FloatingMenuClickHandler)
        .collect()
}
