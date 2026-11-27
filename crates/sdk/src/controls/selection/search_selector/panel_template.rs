use crate::infra::template::ModifiedElementTemplate;
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, div, prelude::*, px};

use crate::infra::popup_placement::{ResolvedPopupPlacement, anchored_popup};

use crate::controls::selector_list::SelectorItemsPanelLook;

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
    /// Where the popup opens; `None` draws the panel inline.
    pub popup_placement: Option<ResolvedPopupPlacement>,
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

impl SearchSelectorPanelTemplate for ModifiedElementTemplate<dyn SearchSelectorPanelTemplate, AnyElement> {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let element = self.base.render(model, cx);
        self.apply(element, cx)
    }
}

pub(super) fn panel_template_with_modifier<F>(
    template: Arc<dyn SearchSelectorPanelTemplate>,
    modifier: F,
) -> Arc<dyn SearchSelectorPanelTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedElementTemplate::new(template, Box::new(modifier)))
}

impl SearchSelectorPanelTemplate for DefaultSearchSelectorPanelTemplate {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
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

        if let (Some(bounds), Some(placement)) = (model.popup_bounds, model.popup_placement) {
            let panel = div()
                .id((model.id.clone(), 1usize))
                .w(bounds.size.width)
                .bg(look.background)
                .border(px(look.border_width))
                .border_color(look.border)
                .rounded(px(look.radius))
                .shadow(look.shadow)
                .overflow_hidden()
                .block_mouse_except_scroll()
                .child(panel_content);
            return anchored_popup(&placement, px(look.window_margin), panel).into_any_element();
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
