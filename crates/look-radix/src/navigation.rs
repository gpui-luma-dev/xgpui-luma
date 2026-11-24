//! Shared Radix navigation pieces: the drag preview and the stepper.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::stepper::*;
use crate::Look;

/// The Radix drag preview, drawn from `.lss` (`style/drag_preview.lss`) at `size`.
pub(crate) fn drag_look(look: &Look, size: crate::RadixSize) -> xgpui_luma::infra::drag_drop::DragDropLook {
    let binding = xgpui_luma::style::StyleBinding::new("drag_preview", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    xgpui_luma::infra::drag_drop::styled_drag_drop_look(&binding)
}
/// Live Radix step status and connector colors, with SDK per-size geometry.
/// Drawn from `.lss` (`style/stepper.lss`) at the look's size.
pub fn stepper_theme(look: &Look) -> Arc<dyn StepperTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("stepper", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(xgpui_luma::controls::stepper::StyledStepperTheme::new(binding))
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
/// use gpui::{Context, IntoElement, ParentElement};
/// use xgpui_luma::controls::{control_group::{ControlGroupItem, ControlGroupItemLike}, accordion::{AccordionItem, AccordionTrigger, AccordionContent}};
/// use xgpui_luma_look_radix::*;
/// fn navigation<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     let group = control_group_builder(look, "group", RadixSize::Two)
///         .items([ControlGroupItem::new("day").label("Day")]).horizontal().spawn(cx);
///     let icons = icon_group_builder(look, "icons")
///         .item_template(icon_group_item_template(look, RadixSize::One, |item: &ControlGroupItem| {
///             gpui::div().child(item.label().clone()).into_any_element()
///         }))
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
        .stylesheet(&look.stylesheet_snapshot())
        .motion_defaults(look.motion_defaults().stepper)
        .template(stepper_template(look))
}
