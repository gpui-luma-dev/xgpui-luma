use std::sync::Arc;

use xgpui_luma::controls::control_group::{
    ControlGroupItemLike, ControlGroupTemplate, ControlGroupTheme, StyledControlGroupTheme,
    control_group_template_with_theme,
};

use crate::look::ShadcnLook;

/// The Shadcn control-group frame drawn from `.lss` (`style/control_group.lss`); rows are this look's ghost
/// toggle at the look's size.
pub fn control_group_theme(theme: ShadcnLook) -> Arc<dyn ControlGroupTheme> {
    let style = theme.clone();
    let metrics = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "control_group",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let rows = super::button::styled_button_template::<()>(theme, crate::ShadcnButtonStyle::Ghost);
    Arc::new(StyledControlGroupTheme::new(
        binding,
        rows,
        Arc::new(move || metrics.mode_tokens().metrics_for(metrics.size())),
    ))
}

pub fn control_group_template<T>(theme: ShadcnLook) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    control_group_template_with_theme(control_group_theme(theme))
}
