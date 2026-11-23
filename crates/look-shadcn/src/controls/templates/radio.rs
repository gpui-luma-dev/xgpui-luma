use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::control_group::{ControlGroupItemLike, ControlGroupTemplate};
use xgpui_luma::controls::radio_button::{RadioButtonData, StyledRadioButtonTemplate};
use xgpui_luma::controls::radio_group::{RadioGroupLayout, radio_group_buttons_template_with};

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

/// The Shadcn radio drawn from `.lss` (`style/radio.lss`), following the look's mode and catalog.
pub fn radio_button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
    Arc::new(styled_radio_button_template(theme, style))
}

/// [`radio_button_template`] before it is shared, for adding modifiers.
pub fn styled_radio_button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> StyledRadioButtonTemplate {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledRadioButtonTemplate::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = super::checkbox::checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    template
}

pub fn radio_group_template<T>(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_buttons_template_with(radio_group_binding(&theme), theme.radio_button_template(style), layout)
}

/// The look's `radio_group` control, following its live theme.
pub(crate) fn radio_group_binding(look: &ShadcnLook) -> xgpui_luma::style::StyleBinding {
    let look = look.clone();
    xgpui_luma::style::StyleBinding::new(
        "radio_group",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || look.style_theme())),
    )
}
