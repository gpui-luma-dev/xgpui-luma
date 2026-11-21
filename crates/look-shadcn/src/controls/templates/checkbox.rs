use std::sync::Arc;

use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::{CheckboxData, StyledCheckboxTemplate};

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

/// The Shadcn checkbox drawn from `.lss` (`style/checkbox.lss`), following the look's mode and catalog.
pub fn checkbox_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<CheckboxData>> {
    let size = theme.size();
    let look = theme.clone();
    let mut template = StyledCheckboxTemplate::live(Arc::new(move || look.style_theme()));
    if let Some(variant) = checkbox_variant(style) {
        template = template.variant(variant);
    }
    if size != crate::ShadcnSize::Md {
        template = template.attr("size", size.key());
    }
    Arc::new(template)
}

/// The `.lss` variant for a style (choice controls and buttons); primary is the base.
pub(crate) fn checkbox_variant(style: ShadcnButtonStyle) -> Option<&'static str> {
    match style {
        ShadcnButtonStyle::Primary => None,
        ShadcnButtonStyle::Secondary => Some("secondary"),
        ShadcnButtonStyle::Outline => Some("outline"),
        ShadcnButtonStyle::Ghost => Some("ghost"),
        ShadcnButtonStyle::ContentOnly => Some("content_only"),
    }
}
