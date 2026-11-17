use crate::infra::locale::{LocaleContext, LogicalStyled};
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::model::{ToolbarItem, ToolbarItemKind, ToolbarItemRenderModel, ToolbarRenderModel, fallback_item};
use super::theme::{ToolbarTheme, ToolbarVariant, default_toolbar_theme};
use crate::controls::control_group::{ControlGroupItemLike, ControlGroupRenderModel, ControlGroupTemplate};
use crate::theme::ControlSize;

pub type ToolbarTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ToolbarTemplate: Send + Sync {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ToolbarRenderedItem {
    pub kind: ToolbarItemKind,
    pub element: gpui::AnyElement,
}

pub struct ThemedToolbarTemplate {
    theme: Arc<dyn ToolbarTheme>,
    size: ControlSize,
    modifiers: Vec<ToolbarTemplateModifier>,
}

impl ThemedToolbarTemplate {
    pub fn new(theme: Arc<dyn ToolbarTheme>) -> Self {
        Self { theme, size: ControlSize::Md, modifiers: Vec::new() }
    }

    pub fn with_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ToolbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedToolbarTemplate {
    base: Arc<dyn ToolbarTemplate>,
    modifiers: Vec<ToolbarTemplateModifier>,
}

impl ModifiedToolbarTemplate {
    fn new(base: Arc<dyn ToolbarTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ToolbarTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ToolbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

impl ToolbarTemplate for ThemedToolbarTemplate {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let size = model.size.unwrap_or(self.size);
        let look = self.theme.resolve(model.enabled, size, model.variant);
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .flex_row_logical(LocaleContext::current(cx).direction)
            .items_center()
            .gap(px(look.gap))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(look.radius))
            .bg(look.background);

        if model.variant == ToolbarVariant::Outline {
            root = root.border(px(look.border_width)).border_color(look.border);
        }

        for item in items {
            let child = match item.kind {
                ToolbarItemKind::Separator => div()
                    .w(px(look.separator_width))
                    .h(px(look.separator_height))
                    .flex_none()
                    .bg(look.separator)
                    .into_any_element(),
                ToolbarItemKind::Hosted => item.element,
            };
            root = root.child(child);
        }

        self.apply_modifiers(root, model)
    }
}

impl ToolbarTemplate for ModifiedToolbarTemplate {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, items, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_toolbar_template() -> Arc<dyn ToolbarTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToolbarTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(ThemedToolbarTemplate::new(default_toolbar_theme()))).clone()
}

pub fn toolbar_template_with_theme(theme: Arc<dyn ToolbarTheme>) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(theme))
}

pub(super) fn modified_toolbar_template<F>(template: Arc<dyn ToolbarTemplate>, modifier: F) -> Arc<dyn ToolbarTemplate>
where
    F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedToolbarTemplate::new(template).with_modifier(Box::new(modifier)))
}

pub(crate) fn render_fallback_toolbar_item(model: &ToolbarItemRenderModel) -> gpui::AnyElement {
    fallback_item(model)
}

pub(crate) fn toolbar_control_group_template(
    size: ControlSize,
    variant: ToolbarVariant,
    template: Arc<dyn ToolbarTemplate>,
) -> ControlGroupTemplate<ToolbarItem> {
    Arc::new(move |model, _handlers, window, cx| {
        let toolbar_model = toolbar_render_model(model, size, variant);
        let items = render_toolbar_items(model, window, cx);
        template.render(&toolbar_model, items, window, cx)
    })
}

fn toolbar_render_model<'a>(
    model: &'a ControlGroupRenderModel<'a, ToolbarItem>,
    size: ControlSize,
    variant: ToolbarVariant,
) -> ToolbarRenderModel<'a> {
    ToolbarRenderModel {
        id: model.id,
        items: model
            .items
            .iter()
            .map(|item| ToolbarItemRenderModel {
                id: item.item.id().clone(),
                kind: item.item.kind,
                index: item.index,
                enabled: item.enabled || item.item.is_separator(),
                state: item.state,
            })
            .collect(),
        enabled: model.enabled,
        size: Some(size),
        variant,
    }
}

fn render_toolbar_items(
    model: &ControlGroupRenderModel<'_, ToolbarItem>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<ToolbarRenderedItem> {
    model
        .items
        .iter()
        .map(|item| {
            let render_model = ToolbarItemRenderModel {
                id: item.item.id().clone(),
                kind: item.item.kind,
                index: item.index,
                enabled: item.enabled || item.item.is_separator(),
                state: item.state,
            };
            let element = match item.item.kind {
                ToolbarItemKind::Separator => div().into_any_element(),
                ToolbarItemKind::Hosted => item
                    .item
                    .content
                    .as_ref()
                    .map(|content| content.present(&render_model, window, cx).element)
                    .unwrap_or_else(|| render_fallback_toolbar_item(&render_model)),
            };
            ToolbarRenderedItem { kind: item.item.kind, element }
        })
        .collect()
}
