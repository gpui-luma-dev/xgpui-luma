//! Shared Radix navigation geometry and stepper styling.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::stepper::*;
use xgpui_luma::theme::{ControlSize, ThemeTokens};
use crate::{Look, ScaleFamily, SemanticRole, Tone};

// Default SDK resolvers are used only for layout; adapters replace every color.
pub(crate) fn geometry_tokens(look: &Look) -> ThemeTokens {
    ThemeTokens { metrics: look.metrics(), stylesheet: look.common_stylesheet().into(), ..Default::default() }
}
pub(crate) fn drag_look(look: &Look, size: ControlSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
    let mut drag = xgpui_luma::infra::drag_drop::DragDropLook::from_tokens(&geometry_tokens(look), size);
    drag.background = look.resolve_role(SemanticRole::Surface).hsla();
    drag.foreground = look.resolve_role(SemanticRole::Foreground).hsla();
    drag.valid_marker = look.resolve_role(SemanticRole::Focus).hsla();
    drag.invalid_marker = look.resolve_role(SemanticRole::Destructive).hsla();
    drag.font_family = crate::typography::font_family(look);
    drag
}
struct StepperThemeAdapter {
    look: Look,
}
impl StepperTheme for StepperThemeAdapter {
    fn resolve(&self, enabled: bool, size: ControlSize) -> StepperLook {
        let mut resolved = DefaultStepperTheme::new(geometry_tokens(&self.look)).resolve(enabled, size);
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        let accent = |step| Tone::Accent.step(&self.look, step);
        resolved.complete_bg = if enabled { accent(9) } else { gray(3) };
        resolved.complete_fg = if enabled {
            Tone::Accent.contrast(&self.look)
        } else {
            gray(8)
        };
        resolved.in_progress_bg = if enabled { accent(4) } else { gray(3) };
        resolved.in_progress_fg = if enabled { accent(11) } else { gray(8) };
        resolved.incomplete_bg = gray(2);
        resolved.incomplete_border = gray(6);
        resolved.incomplete_fg = gray(if enabled { 11 } else { 8 });
        resolved.track_active_color = if enabled { accent(9) } else { gray(5) };
        resolved.track_muted_color = gray(5);
        resolved
    }
}
/// Live Radix step status and connector colors, with SDK per-size geometry.
pub fn stepper_theme(look: &Look) -> Arc<dyn StepperTheme> {
    Arc::new(StepperThemeAdapter { look: look.clone() })
}
/// SDK stepper template with Radix status styling.
pub fn stepper_template(look: &Look) -> Arc<dyn StepperTemplate> {
    Arc::new(ThemedStepperTemplate::new(stepper_theme(look)))
}
/// Build groups and navigation through the full SDK builder APIs.
/// Supply an explicit Look, including forks. Themes resolve changes on render;
/// builder-owned icons, motion defaults, and shell geometry are captured here.
/// Notify affected control entities and owners after mutating the Look.
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma::controls::{control_group::ControlGroupItem, accordion::{AccordionItem, AccordionTrigger, AccordionContent}};
/// use xgpui_luma::theme::ControlSize;
/// use xgpui_luma_look_radix::*;
/// fn navigation<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     let group = control_group_builder(look, "group", ControlSize::Md)
///         .items([ControlGroupItem::new("day").label("Day")]).horizontal().spawn(cx);
///     let icons = icon_group_builder(look, "icons", ControlSize::Sm)
///         .items([ControlGroupItem::new("bold").label("B")]).multiple().spawn(cx);
///     let accordion = accordion_builder(look, "accordion").item(AccordionItem::new(
///         "account", AccordionTrigger::new("Account"), AccordionContent::new("Account preferences")
///     )).spawn(cx);
///     let pager = pager_builder(look, "pager").page_count(10).spawn(cx);
///     let stepper = stepper_builder(look, "stepper", 3).current_step(1).spawn(cx);
///     let sidebar = sidebar_builder(look, "sidebar").spawn(cx);
/// }
/// ```
/// Radix stepper builder; direction, labels, content, state overrides, and motion remain SDK options.
pub fn stepper_builder(look: &Look, id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
    StepperBuilder::new(id, step_count)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().stepper)
        .template(stepper_template(look))
}
