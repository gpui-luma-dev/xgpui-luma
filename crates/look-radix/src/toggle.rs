//! Radix toggle uses button-family variants (selected = solid accent).

use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::toggle::{ToggleData, apply_toggle_progress_chrome};

use crate::button::{ButtonVariant, Paint, button_family_theme_with, button_family_theme_for_size};
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
    size: crate::ButtonSize,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let variant = if page { ButtonVariant::Page } else { ButtonVariant::Soft };
    toggle_template_with_family(button_family_theme_for_size(look, variant, paint, size))
}

fn toggle_template_for_variant(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_with_family(button_family_theme_with(look, variant, paint))
}

fn toggle_template_with_family(
    family: Arc<dyn xgpui_luma::controls::button_family::ButtonFamilyTheme>,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let theme_for_mod = Arc::clone(&family);
    Arc::new(DefaultButtonTemplate::<ToggleData>::new(family).with_modifier(move |element, model| {
        if model.look.is_some() {
            return element;
        }
        let off = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: false }, model.size, model.state);
        let on = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: true }, model.size, model.state);
        apply_toggle_progress_chrome(element, &off, &on, model.data.progress)
    }))
}
