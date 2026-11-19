use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::control_group::{ControlGroupItemLike, ControlGroupTemplate};
use xgpui_luma::controls::radio_button::{RadioButtonData, StyledRadioButtonTemplate};
use xgpui_luma::controls::radio_group::{RadioGroupLayout, radio_group_buttons_template};

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
    radio_group_buttons_template(theme.radio_button_template(style), layout)
}
