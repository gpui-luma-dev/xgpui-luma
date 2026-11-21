//! Radix accordion trigger, content, and drag styling over the SDK template.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::accordion::*;
use xgpui_luma::theme::{InteractionState, MetricTokens};
use crate::Look;
/// The Radix accordion: trigger, content and layout from `.lss` (`style/accordion.lss`); the drag preview,
/// handle icon and tooltip from the look.
struct AccordionThemeAdapter {
    look: Look,
    styled: StyledAccordion,
}
impl AccordionTheme for AccordionThemeAdapter {
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette {
        self.styled.trigger(state)
    }
    fn resolve_content(&self, _: bool) -> AccordionContentPalette {
        self.styled.content()
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics_for(self.look.size())
    }
    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        self.styled.scale(scale_factor)
    }
    fn drag_handle_icon(&self) -> xgpui_luma::infra::icon::IconSource {
        self.look.common_stylesheet().icons.resolve(xgpui_luma::theme::stylesheet::IconRole::DragHandle)
    }
    fn drag_tooltip_theme(&self) -> Arc<dyn xgpui_luma::controls::tooltip::TooltipTheme> {
        crate::tooltip_theme(&self.look)
    }
    fn resolve_drag(&self) -> xgpui_luma::infra::drag_drop::DragDropLook {
        let size = self.look.size();
        crate::navigation::drag_look(&self.look, size)
    }
}
/// Retains the supplied Look and resolves palette, geometry, and drag chrome on render.
pub fn accordion_theme(look: &Look) -> Arc<dyn AccordionTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("accordion", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(AccordionThemeAdapter { look: look.clone(), styled: StyledAccordion::new(binding) })
}
/// SDK accordion template with Radix trigger/content styling.
pub fn accordion_template(look: &Look) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(look)))
}
/// Themed SDK builder retaining expansion, focus, reordering, and callback APIs.
pub fn accordion_builder(look: &Look, id: impl Into<SharedString>) -> AccordionBuilder {
    AccordionBuilder::new(id)
        .stylesheet(&look.sized_stylesheet(look.size()))
        .motion_defaults(look.motion_defaults().accordion)
        .template(accordion_template(look))
}
