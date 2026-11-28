//! The Shadcn pager drawn from `.lss` (`style/pager.lss`); page buttons are this look's `.lss` outline button at
//! the small size, sized and spaced by the pager.

use std::sync::Arc;

use gpui::{FontWeight, prelude::*};
use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::pager::{PagerTemplate, PagerTheme, StyledPagerTheme, ThemedPagerTemplate};
use xgpui_luma::style::PartRead;
use xgpui_luma::style::lss::{Overrides, PartId, PartView, Prop};

use crate::controls::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

/// The page buttons for a resolved pager: the small outline button with the pager's height, radius, gap,
/// padding and text, and the current page in semibold.
fn page_buttons(theme: &ShadcnLook, pager: PartView<'_>) -> Arc<dyn ButtonTemplate<()>> {
    let root = PartId::ROOT;
    let typography = pager.typography(11.0, 16.0);
    let overrides = Overrides::new()
        .set(root, Prop::<f32>::new("height"), pager.number("button_size"))
        .set(root, Prop::<f32>::new("radius"), pager.number("radius"))
        .set(root, Prop::<f32>::new("gap"), pager.number("gap"))
        .set(root, Prop::<f32>::new("padding_x"), pager.number("padding_x"))
        .set(root, Prop::<f32>::new("padding_y"), 0.0)
        .set(root, Prop::<f32>::new("font_size"), typography.size)
        .set(root, Prop::<f32>::new("line_height"), typography.line_height);
    let template =
        super::button::styled_button_template::<()>(theme.with_size(crate::ShadcnSize::Sm), ShadcnButtonStyle::Outline)
            .overrides(overrides)
            .with_modifier(|root, model| {
                if matches!(model.role, ButtonFamilyRole::Toggle { selected: true }) {
                    root.font_weight(FontWeight::SEMIBOLD)
                } else {
                    root
                }
            });
    Arc::new(template)
}

pub fn pager_template(theme: ShadcnLook) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(theme)))
}

pub fn pager_theme(theme: ShadcnLook) -> Arc<dyn PagerTheme> {
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "pager",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    Arc::new(StyledPagerTheme::new(binding, Arc::new(move |pager: PartView<'_>| page_buttons(&theme, pager))))
}
