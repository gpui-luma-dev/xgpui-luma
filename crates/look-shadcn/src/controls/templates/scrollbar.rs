use std::sync::Arc;

use xgpui_luma::controls::scrollbar::{ScrollbarTemplate, ScrollbarTheme, StyledScrollbarTheme, ThemedScrollbarTemplate};
use xgpui_luma::theme::MetricTokens;

use crate::look::ShadcnLook;

/// The Shadcn scrollbar from `.lss` (`style/scrollbar.lss`) with the look's live control metrics.
pub fn scrollbar_theme(theme: ShadcnLook) -> Arc<dyn ScrollbarTheme> {
    let styled = StyledScrollbarTheme::new(theme.style_binding("scrollbar"), MetricTokens::default());
    Arc::new(ShadcnScrollbarTheme { theme, styled })
}

struct ShadcnScrollbarTheme {
    theme: ShadcnLook,
    styled: StyledScrollbarTheme,
}

impl ScrollbarTheme for ShadcnScrollbarTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        self.styled.style()
    }

    fn metrics(&self) -> MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn scrollbar_template(theme: ShadcnLook) -> Arc<dyn ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(scrollbar_theme(theme)))
}
