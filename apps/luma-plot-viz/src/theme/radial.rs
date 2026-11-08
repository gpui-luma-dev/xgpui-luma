use crate::theme::AppTheme;
use xgpui_luma_plot::RadialPresentation;
use crate::theme;
pub(crate) fn style(look: &AppTheme) -> RadialPresentation {
    let pie = theme::pie::style(look);
    RadialPresentation {
        palette: pie.palette,
        label_text: pie.label_text,
        center_title: pie.center_title,
        center_caption: pie.center_caption,
        empty_state: Some(xgpui_luma_plot::EmptyStatePresentation::new(
            "No positive radial values",
            theme::chrome(look).muted_text,
        )),
        track: Some(theme::chrome(look).border.into()),
        ..Default::default()
    }
}
