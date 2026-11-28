use std::sync::{Arc, OnceLock};

use crate::controls::button::{ButtonTemplate, default_button_template};

pub trait PagerTheme: Send + Sync {
    /// The pager's style (#433): the binding the template resolves (see [`resolve_pager`](super::resolve_pager)) and
    /// draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("pager")
    }

    /// The page buttons' template for a pager whose resolved root is `_pager`.
    fn button_template(&self, _pager: xgpui_luma_style::PartView<'_>) -> Arc<dyn ButtonTemplate<()>> {
        default_button_template()
    }
}

pub fn default_pager_theme() -> Arc<dyn PagerTheme> {
    static THEME: OnceLock<Arc<dyn PagerTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledPagerTheme::sdk())).clone()
}
