use std::sync::Arc;

use gpui::{SharedString, div, prelude::*};
use xgpui_luma::controls::button::ButtonContentContext;
use xgpui_luma::theme::InteractionState;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::icons::render_lucide_icon;

#[derive(Clone, Copy)]
pub(crate) struct ButtonStateSample {
    pub id: &'static str,
    pub header: &'static str,
    pub state: InteractionState,
}

#[derive(Clone, Copy)]
pub(crate) enum ButtonTemplateVariant {
    TextButton,
    IconButton,
}

impl ButtonTemplateVariant {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::TextButton => "text-button",
            Self::IconButton => "icon-button",
        }
    }

    pub(crate) fn round(self) -> bool {
        matches!(self, Self::IconButton)
    }

    pub(crate) fn content(self) -> xgpui_luma::controls::button::ControlPresenter<ButtonContentContext<()>> {
        let label = SharedString::from("Button");
        match self {
            Self::TextButton => Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
            Self::IconButton => Arc::new(move |model, _| {
                let icon_size = model.look.icon_size;
                render_lucide_icon(LucideIcon::Heart, model.look.foreground, icon_size)
            }),
        }
    }
}
