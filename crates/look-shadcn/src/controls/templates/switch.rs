use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::switch::{StyledSwitchTemplate, SwitchData};

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

/// The Shadcn switch drawn from `.lss` (`style/switch.lss`), following the look's mode and catalog.
pub fn switch_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<SwitchData>> {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledSwitchTemplate::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = super::checkbox::checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    Arc::new(template)
}
