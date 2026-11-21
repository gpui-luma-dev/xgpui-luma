use std::sync::Arc;

use xgpui_luma::controls::button::{ButtonData, ButtonTemplate, StyledButtonTemplate};
use xgpui_luma::controls::toggle::ToggleData;

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

/// The Shadcn button drawn from `.lss` (`style/button.lss`), following the look's mode and catalog.
pub fn button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(styled_button_template::<()>(theme, style))
}

/// The Shadcn toggle drawn from `.lss` (`style/toggle.lss`): its colors blend with the toggle's progress.
pub fn toggle_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<ToggleData>> {
    Arc::new(styled_button_template::<ToggleData>(theme, style))
}

pub(crate) fn styled_button_template<D: ButtonData>(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> StyledButtonTemplate<D> {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledButtonTemplate::<D>::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = super::checkbox::checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    template
}

/// Toggle chrome for control-group items (`ButtonTemplate<bool>`), drawn from `.lss`; the item's role
/// carries its selection.
pub fn toggle_item_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(styled_button_template::<bool>(theme, style))
}

/// Animated toggle chrome for control-group items carrying [`ToggleData`], drawn from `.lss`.
pub fn animated_toggle_item_template(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    Arc::new(styled_button_template::<ToggleData>(theme, style))
}
