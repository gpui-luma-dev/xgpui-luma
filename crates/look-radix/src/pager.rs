//! Radix pagination using shared button styling and SDK pager geometry.
use std::sync::Arc;
use gpui::SharedString;
use xgpui_luma::controls::pager::*;
use crate::{ButtonVariant, Look, Paint};
/// Live Radix pagination chrome, including page-size popup colors.
/// Drawn from `.lss` (`style/pager.lss`) at the look's size; page buttons are the ghost button.
pub fn pager_theme(look: &Look) -> Arc<dyn PagerTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("pager", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let buttons = look.clone();
    Arc::new(StyledPagerTheme::new(
        binding,
        Arc::new(move |_: &PagerLook| crate::button_template(&buttons, ButtonVariant::Ghost, Paint::accent())),
    ))
}
/// SDK pager template using Radix button and popup chrome.
pub fn pager_template(look: &Look) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(look)))
}
/// Themed SDK builder retaining pagination, formatter, callback, and template APIs.
pub fn pager_builder(look: &Look, id: impl Into<SharedString>) -> PagerBuilder {
    PagerBuilder::new(id).stylesheet(&look.stylesheet_snapshot()).theme(pager_theme(look))
}
