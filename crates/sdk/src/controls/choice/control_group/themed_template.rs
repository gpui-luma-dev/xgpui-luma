use crate::style::PartRead;
use xgpui_luma_style::PartId;
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
        let style = self.inner.theme.resolve_list_style(model.enabled);
        let list = style.root();
        let item_disabled_opacity = self.inner.theme.resolve_list_style(false).part(PartId::new("item")).opacity();

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .overflow_hidden()
            .rounded(px(list.number("radius")))
            .bg(list.color("background"))
            .border(px(list.number("border_width")))
            .border_color(list.color("border"))
            .px(px(list.number("padding_x")))
            .py(px(list.number("padding_y")));

        root = match chrome.layout {
            ControlGroupLayout::Horizontal => root.items_center().gap(px(list.number("gap"))),
            ControlGroupLayout::Vertical => root.flex_col().items_start().gap(px(list.number("vertical_gap"))),
        };

        root = root.children(render_control_group_items(model, handlers, window, cx, item_disabled_opacity));
        self.inner.apply_modifiers(root, &chrome)
    }
}

pub fn themed_control_group_template() -> ThemedControlGroupTemplate {
    ThemedControlGroupTemplate::new(default_control_group_theme())
}
