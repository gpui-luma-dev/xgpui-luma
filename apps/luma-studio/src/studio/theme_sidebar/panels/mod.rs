pub(crate) mod colors;
mod other;
mod typography;

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{relative, App, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window};
use xgpui_luma::controls::accordion::AccordionControl;
use xgpui_luma::controls::context_menu::{ContextMenu, MenuItem};
use xgpui_luma::controls::slider::Slider;
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma_look_shadcn::{ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;

pub(super) use colors::ColorsPanel;
pub(super) use other::OtherPanel;
pub(super) use typography::TypographyPanel;

pub(super) struct PanelContextMenuHost {
    context_menu: Entity<ContextMenu>,
}

impl PanelContextMenuHost {
    pub(super) fn new<F, E>(
        id: impl Into<SharedString>,
        target_content: F,
        items: impl IntoIterator<Item = MenuItem>,
        look: Arc<ShadcnLook>,
        cx: &mut Context<Self>,
    ) -> Self
    where
        F: Fn(&mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        let context_menu = shadcn::ContextMenu::new(id)
            .look(look.as_ref())
            .target_content(target_content)
            .items(items)
            .spawn(cx);

        Self { context_menu }
    }

    pub(super) fn context_menu(&self) -> Entity<ContextMenu> {
        self.context_menu.clone()
    }
}

impl Render for PanelContextMenuHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::div().w_full().min_h(relative(1.0)).child(self.context_menu.clone())
    }
}

pub(super) fn colors_context_menu_items() -> [MenuItem; 4] {
    [
        MenuItem::new("expand-all").label("Expand All"),
        MenuItem::new("collapse-all").label("Collapse All"),
        MenuItem::new("reset").label("Reset"),
        MenuItem::new("toggle-mode").label("Toggle Light/Dark"),
    ]
}

pub(super) fn other_context_menu_items() -> [MenuItem; 1] {
    [MenuItem::new("reset").label("Reset")]
}

pub(super) fn category_item_id(prefix: &str, category: &str) -> String {
    format!("{prefix}-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

pub(super) fn expanded_category_ids<'a>(
    accordion: &Entity<AccordionControl>,
    categories: impl IntoIterator<Item = &'a str>,
    prefix: &str,
    cx: &App,
) -> HashSet<String> {
    let accordion = accordion.read(cx);
    categories
        .into_iter()
        .filter_map(|category| {
            let id = category_item_id(prefix, category);
            accordion.is_expanded(&id.clone().into()).then_some(id)
        })
        .collect()
}

pub(super) fn spawn_compact_textfield<T: 'static>(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: impl Into<SharedString>,
    cx: &mut Context<T>,
) -> TextField {
    colors::apply_token_field_style(look, shadcn::TextField::new(format!("luma-studio-{id}-field")).look(look.as_ref()))
        .value(value)
        .full_width(true)
        .spawn(cx)
}

pub(super) fn spawn_slider<T: 'static>(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    cx: &mut Context<T>,
) -> Slider {
    shadcn::Slider::new(format!("luma-studio-{id}-slider"))
        .look(look.as_ref())
        .range(min..max)
        .step(step)
        .value(value)
        .spawn(cx)
}
