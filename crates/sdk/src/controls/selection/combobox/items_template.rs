use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{App, SharedString, Stateful, div, prelude::*, px};

use super::behavior::SelectionItem;
use super::item_template::ComboBoxItemTemplate;
use crate::controls::selector_list::{SelectorItemsPanelLook, SelectorPanelClickHandler, SelectorPanelHoverHandler};

pub struct ComboBoxItemsRenderModel<'a> {
    pub menu_id: &'a SharedString,
    pub combobox_id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a ComboBoxItemTemplate<SelectionItem>>,
    pub look: SelectorItemsPanelLook,
}

pub struct ComboBoxItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    /// One callback per visible row, including disabled rows; consume in visible order.
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub type ComboBoxItemsTemplateModifier =
    Box<dyn Fn(Stateful<gpui::Div>, &ComboBoxItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static>;

pub trait ComboBoxItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

impl RenderModel for dyn ComboBoxItemsTemplate {
    type Model<'a> = ComboBoxItemsRenderModel<'a>;
}

pub struct DefaultComboBoxItemsTemplate {
    modifiers: RootModifiers<dyn ComboBoxItemsTemplate>,
}

impl DefaultComboBoxItemsTemplate {
    pub fn new() -> Self {
        Self { modifiers: RootModifiers::default() }
    }

    #[allow(dead_code)]
    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<gpui::Div>, &ComboBoxItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl Default for DefaultComboBoxItemsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_combobox_items_template() -> Arc<dyn ComboBoxItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxItemsTemplate::new())).clone()
}

pub fn combobox_items_template_with_modifier<F>(
    template: Arc<dyn ComboBoxItemsTemplate>,
    modifier: F,
) -> Arc<dyn ComboBoxItemsTemplate>
where
    F: Fn(Stateful<gpui::Div>, &ComboBoxItemsRenderModel<'_>) -> Stateful<gpui::Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl ComboBoxItemsTemplate for DefaultComboBoxItemsTemplate {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let ComboBoxItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let look = &model.look;

        let mut root =
            div().id((model.menu_id.clone(), 0usize)).relative().flex().flex_col().w_full().p(px(look.padding));
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let click = clicks.next();
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let enabled_item = model.enabled && item.enabled;
            let color = if enabled_item {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let selected = model.selected_source_index == Some(source_index);
            let active = enabled_item && model.active_visible_index == Some(visible_index);
            let content = if let Some(item_template) = model.item_template {
                item_template(
                    &super::item_template::ComboBoxItemRenderModel {
                        combobox_id: model.combobox_id,
                        item,
                        source_index,
                        visible_index,
                        selected,
                        active,
                        open: model.open,
                        enabled: enabled_item,
                    },
                    cx,
                )
            } else {
                div().flex_1().child(item.label.clone()).into_any_element()
            };

            let mut row = div()
                .id(("row", visible_index))
                .flex()
                .items_center()
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(content);

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if let Some(click) = click {
                    row = row.on_click(click);
                }
            } else {
                row = crate::style::fade(row, look.item_disabled_opacity);
            }

            root = root.child(row);
        }

        self.modifiers.apply(root, model)
    }
}

impl ComboBoxItemsTemplate for ModifiedTemplate<dyn ComboBoxItemsTemplate> {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply(root, model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_template_with_modifier_wraps_template() {
        let template = default_combobox_items_template();
        let wrapped = combobox_items_template_with_modifier(template.clone(), |element, _| element);

        assert!(!Arc::ptr_eq(&wrapped, &template));
    }
}
