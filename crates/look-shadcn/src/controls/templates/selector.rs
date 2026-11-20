use std::sync::Arc;

use xgpui_luma::controls::selector::{SelectorItemLike, SelectorTheme, StyledSelectorTheme, ThemedSelectorTemplate};
use xgpui_luma::style::ThemeSource;
use xgpui_luma::controls::selector_list::default_selector_items_template;

use crate::look::ShadcnLook;

/// The Shadcn selector trigger drawn from `.lss` (`style/selector.lss`, over this look's outline and
/// ghost buttons) at the look's size; the popup panel keeps the look's menu values.
pub fn selector_theme(theme: ShadcnLook) -> Arc<dyn SelectorTheme> {
    Arc::new(styled_selector_theme(theme))
}

fn styled_selector_theme(theme: ShadcnLook) -> StyledSelectorTheme {
    let size = theme.size();
    let style = theme.clone();
    let panel = theme.clone();
    StyledSelectorTheme::new(
        ThemeSource::Live(Arc::new(move || style.style_theme())),
        Arc::new(move || {
            crate::controls::selector_items_panel::selector_items_panel_look(
                panel.mode_tokens().as_ref(),
                panel.mode(),
                size,
            )
        }),
    )
    .with(|binding| {
        if size != crate::ShadcnSize::Md {
            binding.attr("size", size.key())
        } else {
            binding
        }
    })
}

pub fn selector_template<T>(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::selector::SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(styled_selector_theme(theme)),
        default_selector_items_template::<T>(),
    ))
}
