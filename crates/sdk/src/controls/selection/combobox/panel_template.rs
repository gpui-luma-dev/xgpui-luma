use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, div, prelude::*, px};

use crate::infra::popup_placement::{ResolvedPopupPlacement, anchored_popup};

use crate::controls::selector_list::SelectorItemsPanelLook;

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
    pub popup_look: SelectorItemsPanelLook,
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

struct ModifiedComboBoxPanelTemplate {
    base: Arc<dyn ComboBoxPanelTemplate>,
    modifiers: Vec<ComboBoxPanelTemplateModifier>,
}

impl ModifiedComboBoxPanelTemplate {
    fn new(base: Arc<dyn ComboBoxPanelTemplate>) -> Self {
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

impl ComboBoxPanelTemplate for ModifiedComboBoxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let mut element = self.base.render(model, cx);
        for modifier in &self.modifiers {
            element = modifier(element, cx);
        }
        element
    }
}

pub(super) fn panel_template_with_modifier<F>(
    template: Arc<dyn ComboBoxPanelTemplate>,
    modifier: F,
) -> Arc<dyn ComboBoxPanelTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedComboBoxPanelTemplate::new(template).with_modifier(modifier))
}

impl ComboBoxPanelTemplate for DefaultComboBoxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
        if let (Some(bounds), Some(placement)) = (model.popup_bounds, model.popup_placement) {
            let look = model.popup_look;
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
                .child(model.list_content);
            return anchored_popup(&placement, px(look.window_margin), panel).into_any_element();
        }

        let look = model.popup_look;
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
            .child(model.list_content)
            .into_any_element()
    }
}
