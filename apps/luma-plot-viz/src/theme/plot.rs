use gpui::{Hsla, px, linear_gradient, linear_color_stop};
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    AreaPresentation, AxisPresentation, ChartMetrics, EmptyStatePresentation, GridPresentation, LinePresentation,
    MarkerPresentation, CartesianStyle, CartesianStyleProvider, SeriesId, SeriesStyle, TextPresentation,
};

/// One live SDK adapter for all plot appearance. Series assignments are explicit
/// host choices; this adapter has no knowledge of dataset names or sample values.
#[derive(Clone)]
pub(crate) struct SdkCartesianStyle {
    look: AppTheme,
    blue: bool,
    series: Vec<(SeriesId, usize)>,
}
impl SdkCartesianStyle {
    pub(crate) fn new(look: &AppTheme) -> Self {
        Self { look: look.clone(), blue: false, series: Vec::new() }
    }
    /// Screenshot-style blue palette, resolved for the current light/dark mode.
    pub(crate) fn blue(mut self) -> Self {
        self.blue = true;
        self
    }
    pub(crate) fn series(mut self, id: impl Into<SeriesId>, color: usize) -> Self {
        self.series.push((id.into(), color));
        self
    }

    fn colors(&self) -> [Hsla; 2] {
        if self.blue {
            super::palette::blue(self.look.mode())
        } else {
            super::palette::categorical(self.look.mode())
        }
    }
    fn series_style(&self, color: Hsla) -> SeriesStyle {
        let mut line = LinePresentation::new(color);
        line.markers = Some(MarkerPresentation::new(color, px(4.0)));
        let area = AreaPresentation::new(linear_gradient(
            180.0,
            linear_color_stop(color.opacity(if self.blue { 0.45 } else { 0.55 }), 0.0),
            linear_color_stop(color.opacity(0.02), 1.0),
        ))
        .outline(LinePresentation::new(color));
        SeriesStyle { line: Some(line), area: Some(area), bar: Some(xgpui_luma_plot::BarPresentation::new(color)) }
    }
}
impl CartesianStyleProvider for SdkCartesianStyle {
    fn resolve(&self) -> CartesianStyle {
        let spacing = self.look.metrics().spacing;
        let muted = self.look.tokens().palette.app.muted_foreground;
        let typography = crate::theme::typography();
        let caption = typography.text.caption;
        let text = TextPresentation {
            color: muted,
            font_family: typography.font.sans.family.into(),
            font_size: px(caption.size),
            line_height: px(caption.line_height),
        };
        let mut axis = AxisPresentation::new(muted).tick_labels(text.clone());
        axis.title = text;
        axis.gutter = px(spacing.s6 + spacing.s2);
        axis.tick_length = px(spacing.s1);
        axis.label_gap = px(spacing.s1);
        let colors = self.colors();
        let defaults = self.series_style(colors[0]);
        let mut style = CartesianStyle::new(axis, defaults.line.expect("adapter supplies a line"));
        style.area = defaults.area.expect("adapter supplies an area");
        style.grid = GridPresentation::new(self.look.tokens().palette.border.default);
        style.metrics = ChartMetrics {
            plot_padding: px(spacing.s1),
            legend_gap: px(spacing.s3),
            legend_item_gap: px(spacing.s1),
            legend_swatch_width: px(spacing.s5),
            legend_swatch_height: px(spacing.s0.max(2.0)),
        };
        style.empty_state = Some(EmptyStatePresentation::new("No finite samples selected", muted));
        for (id, color) in &self.series {
            style.series.insert(id.clone(), self.series_style(colors[color % colors.len()]));
        }
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn retained_sdk_provider_resolves_current_mode_and_explicit_series_assignments() {
        let look = AppTheme::new();
        look.set_mode(ThemeMode::Light);
        let provider = SdkCartesianStyle::new(&look).blue().series("desktop", 0).series("mobile", 1);
        let light = provider.resolve();
        look.set_mode(ThemeMode::Dark);
        let dark = provider.resolve();
        assert_ne!(light.axes[0].presentation.stroke.color, dark.axes[0].presentation.stroke.color);
        assert_ne!(
            light.series[&"desktop".into()].line.unwrap().stroke.color,
            dark.series[&"desktop".into()].line.unwrap().stroke.color
        );
        assert_ne!(
            dark.series[&"desktop".into()].line.unwrap().stroke.color,
            dark.series[&"mobile".into()].line.unwrap().stroke.color
        );
        assert!(dark.series[&"desktop".into()].area.unwrap().outline.is_some());
        assert_eq!(dark.series.len(), 2);
        assert_eq!(SdkCartesianStyle::new(&look).resolve().series.len(), 0);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let style = SdkCartesianStyle::new(&look).series("a", 0).series("b", 1).resolve();
            for id in ["a", "b"] {
                let color = style.series[&id.into()].line.unwrap().stroke.color;
                assert!(color.s > 0.5, "chart series must stay colored over monochrome UI");
                assert_ne!(color, look.tokens().palette.data.accent_1);
                assert_ne!(color, look.tokens().palette.data.accent_2);
            }
            assert_eq!(style.axes[0].presentation.stroke.color.s, 0.0);
            assert_eq!(style.grid.horizontal.unwrap().color.s, 0.0);
            assert_ne!(
                style.series[&"a".into()].line.unwrap().stroke.color,
                style.series[&"b".into()].line.unwrap().stroke.color
            );
        }
    }
}
