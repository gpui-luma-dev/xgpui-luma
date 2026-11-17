use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, anchored, deferred, div, point, prelude::*, px};

use crate::controls::selector_list::SelectorItemsPanelLook;
use crate::infra::locale::{LayoutDirection, LocaleContext};

use super::behavior::SelectionItem;
use super::item_template::SearchSelectorItemTemplate;

pub struct SearchSelectorPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub geometry: crate::theme::stylesheet::SearchSelectorGeometry,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a SearchSelectorItemTemplate<SelectionItem>>,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_look: SelectorItemsPanelLook,
    pub search_content: Option<AnyElement>,
    pub list_content: AnyElement,
}

pub trait SearchSelectorPanelTemplate: Send + Sync {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement;
}

pub type SearchSelectorPanelTemplateModifier = Box<dyn Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct DefaultSearchSelectorPanelTemplate;

pub fn default_search_selector_panel_template() -> Arc<dyn SearchSelectorPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorPanelTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorPanelTemplate)).clone()
}

struct ModifiedSearchSelectorPanelTemplate {
    base: Arc<dyn SearchSelectorPanelTemplate>,
    modifiers: Vec<SearchSelectorPanelTemplateModifier>,
}

impl ModifiedSearchSelectorPanelTemplate {
    fn new(base: Arc<dyn SearchSelectorPanelTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl SearchSelectorPanelTemplate for ModifiedSearchSelectorPanelTemplate {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let mut element = self.base.render(model, cx);
        for modifier in &self.modifiers {
            element = modifier(element, cx);
        }
        element
    }
}

pub(super) fn panel_template_with_modifier<F>(
    template: Arc<dyn SearchSelectorPanelTemplate>,
    modifier: F,
) -> Arc<dyn SearchSelectorPanelTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedSearchSelectorPanelTemplate::new(template).with_modifier(modifier))
}

impl SearchSelectorPanelTemplate for DefaultSearchSelectorPanelTemplate {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let direction = LocaleContext::current(cx).direction;
        let look = model.popup_look;

        let panel_content = div()
            .id("panel-content")
            .w_full()
            .when_some(model.search_content, |panel, search| {
                panel
                    .child(div().p(px(model.geometry.search_padding)).child(search))
                    .child(div().h(px(model.geometry.search_separator_height)).bg(look.border))
            })
            .child(model.list_content);

        if let Some(bounds) = model.popup_bounds {
            let (anchor, position) = popup_start(bounds, direction);
            return deferred(
                anchored()
                    .snap_to_window_with_margin(px(look.window_margin))
                    .anchor(anchor)
                    .position(position)
                    .offset(point(px(0.0), px(look.popup_offset_y)))
                    .child(
                        div()
                            .id((model.id.clone(), 1usize))
                            .w(bounds.size.width)
                            .bg(look.background)
                            .border(px(look.border_width))
                            .border_color(look.border)
                            .rounded(px(look.radius))
                            .shadow(look.shadow)
                            .overflow_hidden()
                            .block_mouse_except_scroll()
                            .child(panel_content),
                    ),
            )
            .with_priority(1)
            .into_any_element();
        }

        div()
            .id((model.id.clone(), 0usize))
            .w_full()
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow)
            .overflow_hidden()
            .block_mouse_except_scroll()
            .child(panel_content)
            .into_any_element()
    }
}

fn popup_start(bounds: Bounds<Pixels>, direction: LayoutDirection) -> (gpui::Anchor, gpui::Point<Pixels>) {
    match direction {
        LayoutDirection::Ltr => (gpui::Anchor::TopLeft, point(bounds.left(), bounds.bottom())),
        LayoutDirection::Rtl => (gpui::Anchor::TopRight, point(bounds.right(), bounds.bottom())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_anchors_at_inline_start_without_changing_vertical_position() {
        let bounds = Bounds::new(point(px(40.), px(30.)), gpui::size(px(180.), px(32.)));
        assert_eq!(popup_start(bounds, LayoutDirection::Ltr), (gpui::Anchor::TopLeft, point(px(40.), px(62.))));
        assert_eq!(popup_start(bounds, LayoutDirection::Rtl), (gpui::Anchor::TopRight, point(px(220.), px(62.))));
    }
}
