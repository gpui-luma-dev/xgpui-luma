use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, prelude::*, px};

use super::model::{ControlGroupChromeModel, ControlGroupItemLike, ControlGroupLayout, ControlGroupRenderModel};
use super::template::{ControlGroupTemplateHandlers, render_control_group_items};
use super::theme::{ControlGroupTheme, default_control_group_theme};
use crate::infra::template::{ControlTemplate, TemplateWithModifiers};

pub struct ThemedControlGroupTemplate {
    inner: ControlTemplate<dyn ControlGroupTheme, ControlGroupChromeModel>,
}

impl ThemedControlGroupTemplate {
    pub fn new(theme: Arc<dyn ControlGroupTheme>) -> Self {
        Self { inner: ControlTemplate::new(theme) }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.inner = self.inner.with_modifier(modifier);
        self
    }

    pub fn render<T>(
        &self,
        model: &ControlGroupRenderModel<'_, T>,
        handlers: ControlGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>
    where
        T: ControlGroupItemLike + 'static,
    {
        let chrome = ControlGroupChromeModel::from(model);
        let list = self.inner.theme.resolve_list(model.enabled);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .overflow_hidden()
            .rounded(px(list.radius))
            .bg(list.background)
            .border(px(list.border_width))
            .border_color(list.border)
            .px(px(list.padding_x))
            .py(px(list.padding_y));

        root = match chrome.layout {
            ControlGroupLayout::Horizontal => root.items_center().gap(px(list.gap)),
            ControlGroupLayout::Vertical => root.flex_col().items_start().gap(px(list.vertical_gap)),
        };

        root = root.children(render_control_group_items(model, handlers, window, cx, list.disabled_opacity));
        self.inner.apply_modifiers(root, &chrome)
    }
}

pub fn themed_control_group_template() -> ThemedControlGroupTemplate {
    ThemedControlGroupTemplate::new(default_control_group_theme())
}
