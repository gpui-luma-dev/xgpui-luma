use crate::infra::locale::{LayoutDirection, LocaleContext};
use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, anchored, deferred, div, point, prelude::*, px};

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
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let rtl = LocaleContext::current(cx).direction == LayoutDirection::Rtl;
        if let Some(bounds) = model.popup_bounds {
            let look = model.popup_look;

            return deferred(
                anchored()
                    .snap_to_window_with_margin(px(look.window_margin))
                    .anchor(if rtl {
                        gpui::Anchor::TopRight
                    } else {
                        gpui::Anchor::TopLeft
                    })
                    .position(point(if rtl { bounds.right() } else { bounds.left() }, bounds.bottom()))
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
                            .child(model.list_content),
                    ),
            )
            .with_priority(1)
            .into_any_element();
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
