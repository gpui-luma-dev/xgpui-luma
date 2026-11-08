//! Radix accordion trigger, content, and drag styling over the SDK template.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::accordion::*;
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};
use crate::{Look, ScaleFamily, SemanticRole};
struct AccordionThemeAdapter {
    look: Look,
}
impl AccordionTheme for AccordionThemeAdapter {
    fn resolve_trigger(&self, state: InteractionState, size: ControlSize) -> AccordionPalette {
        let mut palette =
            DefaultAccordionTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve_trigger(state, size);
        palette.background = match state.layer() {
            InteractionLayer::Hovered => Some(self.look.resolve_step(ScaleFamily::Gray, 3).hsla()),
            InteractionLayer::Pressed => Some(self.look.resolve_step(ScaleFamily::Gray, 4).hsla()),
            _ => None,
        };
        palette.foreground = self
            .look
            .resolve_role(if state.disabled {
                SemanticRole::MutedForeground
            } else {
                SemanticRole::Foreground
            })
            .hsla();
        palette.icon_color = palette.foreground;
        palette.chevron_color = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        palette.border_color = self
            .look
            .resolve_role(if state.focused && !state.disabled {
                SemanticRole::Focus
            } else {
                SemanticRole::Border
            })
            .hsla();
        palette.font_family = crate::typography::font_family(&self.look);
        palette
    }
    fn resolve_content(&self, _: bool) -> AccordionContentPalette {
        AccordionContentPalette {
            background: None,
            foreground: self.look.resolve_role(SemanticRole::Foreground).hsla(),
        }
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
    fn resolve_scale(&self, size: ControlSize, scale_factor: f32) -> AccordionScale {
        AccordionScale::compute_with_stylesheet(
            size,
            &self.look.metrics(),
            scale_factor,
            &self.look.common_stylesheet(),
        )
    }
    fn drag_handle_icon(&self) -> xgpui_luma::infra::icon::IconSource {
        self.look.common_stylesheet().icons.resolve(xgpui_luma::theme::stylesheet::IconRole::DragHandle)
    }
    fn drag_tooltip_theme(&self) -> Arc<dyn xgpui_luma::controls::tooltip::TooltipTheme> {
        crate::tooltip_theme(&self.look)
    }
    fn resolve_drag(&self, size: ControlSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
        crate::navigation::drag_look(&self.look, size)
    }
}
/// Retains the supplied Look and resolves palette, geometry, and drag chrome on render.
pub fn accordion_theme(look: &Look) -> Arc<dyn AccordionTheme> {
    Arc::new(AccordionThemeAdapter { look: look.clone() })
}
/// SDK accordion template with Radix trigger/content styling.
pub fn accordion_template(look: &Look) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(look)))
}
/// Themed SDK builder retaining expansion, focus, reordering, and callback APIs.
pub fn accordion_builder(look: &Look, id: impl Into<SharedString>) -> AccordionBuilder {
    AccordionBuilder::new(id)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().accordion)
        .template(accordion_template(look))
}
