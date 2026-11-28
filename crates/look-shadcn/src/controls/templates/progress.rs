use std::sync::Arc;

use xgpui_luma::controls::progress::{ProgressTheme, ThemedLinearProgressTemplate, ThemedProgressTemplate};

use crate::controls::progress::progress_style;
use crate::look::ShadcnLook;

struct ShadcnProgressTheme {
    theme: ShadcnLook,
}

impl ProgressTheme for ShadcnProgressTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        progress_style(self.theme.mode_tokens().as_ref(), self.theme.size())
    }
}

pub fn progress_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedProgressTemplate::new(progress_theme(theme)))
}

pub fn linear_progress_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedLinearProgressTemplate::new(progress_theme(theme)))
}

pub fn progress_theme(theme: ShadcnLook) -> Arc<dyn ProgressTheme> {
    Arc::new(ShadcnProgressTheme { theme: theme.clone() })
}
