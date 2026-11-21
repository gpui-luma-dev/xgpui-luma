//! Radix toggle drawn from `.lss` (`style/toggle.lss`) on the button-family variants (selected = solid accent).

use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::toggle::ToggleData;

use crate::ButtonSize;
use crate::button::{ButtonVariant, Paint, button_template, styled_button_template};
use crate::look::Look;

pub fn toggle_template(look: &Look) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_for(look, Paint::accent())
}

/// Soft toggle with explicit paint (accent/gray × high-contrast).
pub fn toggle_template_for(look: &Look, paint: Paint) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_for_variant(look, ButtonVariant::Soft, paint)
}

pub fn page_toggle_template(look: &Look) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_for_variant(look, ButtonVariant::Page, Paint::accent())
}

pub(crate) fn toggle_template_for_size(
    look: &Look,
    paint: Paint,
    page: bool,
    size: ButtonSize,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let variant = if page { ButtonVariant::Page } else { ButtonVariant::Soft };
    Arc::new(styled_button_template::<ToggleData>(look, variant, paint, size))
}

fn toggle_template_for_variant(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    button_template::<ToggleData>(look, variant, paint)
}
