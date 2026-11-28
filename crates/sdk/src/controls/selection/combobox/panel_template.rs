use crate::infra::template::ModifiedElementTemplate;
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, div, prelude::*, px};

use crate::infra::popup_placement::{ResolvedPopupPlacement, anchored_popup};

use super::behavior::SelectionItem;
use super::item_template::ComboBoxItemTemplate;

pub struct ComboBoxPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a ComboBoxItemTemplate<SelectionItem>>,
    pub popup_bounds: Option<Bounds<Pixels>>,
    /// Where the popup opens; `None` draws the panel inline.
    pub popup_placement: Option<ResolvedPopupPlacement>,
    /// The popup panel's style (a `floating_menu` binding, #433).
    pub popup_menu: crate::style::StyleBinding,
    pub list_content: AnyElement,
}

pub trait ComboBoxPanelTemplate: Send + Sync {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement;
}

pub type ComboBoxPanelTemplateModifier = Box<dyn Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct DefaultComboBoxPanelTemplate;

pub fn default_combobox_panel_template() -> Arc<dyn ComboBoxPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxPanelTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxPanelTemplate)).clone()
}

impl ComboBoxPanelTemplate for ModifiedElementTemplate<dyn ComboBoxPanelTemplate, AnyElement> {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let element = self.base.render(model, cx);
        self.apply(element, cx)
    }
}

pub(super) fn panel_template_with_modifier<F>(
    template: Arc<dyn ComboBoxPanelTemplate>,
    modifier: F,
) -> Arc<dyn ComboBoxPanelTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedElementTemplate::new(template, Box::new(modifier)))
}

impl ComboBoxPanelTemplate for DefaultComboBoxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
        let style = crate::controls::floating_menu::MenuStyle::resolve(&model.popup_menu);
        let window_margin = crate::controls::selector_list::selector_panel_placement(&model.popup_menu).window_margin;
        if let (Some(bounds), Some(placement)) = (model.popup_bounds, model.popup_placement) {
            let panel = style
                .surface(div().id((model.id.clone(), 1usize)))
                .w(bounds.size.width)
                .overflow_hidden()
                .block_mouse_except_scroll()
                .child(model.list_content);
            return anchored_popup(&placement, px(window_margin), panel).into_any_element();
        }

        style
            .surface(div().id((model.id.clone(), 0usize)))
            .w_full()
            .overflow_hidden()
            .block_mouse_except_scroll()
            .child(model.list_content)
            .into_any_element()
    }
}
