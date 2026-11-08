use gpui::px;
use crate::theme::AppTheme;
use xgpui_luma_plot::{PiePresentation, TextPresentation, LegendPresentation, EmptyStatePresentation};
use crate::theme;

pub(crate) fn style(look: &AppTheme) -> PiePresentation {
    let chrome = theme::chrome(look);
    let typography = theme::typography();
    let mut text = TextPresentation::new(chrome.body_text);
    text.font_family = typography.font.sans.family.into();
    text.font_size = px(12.0);
    text.line_height = px(18.0);
    let mut title = text.clone();
    title.font_size = px(28.0);
    title.line_height = px(34.0);
    let mut caption = text.clone();
    caption.color = chrome.muted_text;
    let mut style = PiePresentation {
        palette: super::palette::BLUE_SCALE.map(Into::into).to_vec(),
        label_text: text,
        center_title: title,
        center_caption: caption,
        legend: LegendPresentation::new(chrome.body_text),
        empty_state: Some(EmptyStatePresentation::new("No positive finite values", chrome.muted_text)),
        ..PiePresentation::default()
    };
    style.legend.visible = false;
    style
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pie_text_resolves_the_current_theme_mode() {
        let look = AppTheme::new();
        look.set_mode(xgpui_luma::theme::ThemeMode::Light);
        let light = style(&look);
        look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
        let dark = style(&look);
        assert_ne!(light.label_text.color, dark.label_text.color);
        assert_ne!(light.center_title.color, dark.center_title.color);
    }
}
