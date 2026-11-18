use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::button::{ButtonTemplate, default_button_template};
use crate::theme::{LumaTextStyle, ThemeTokens};

use super::PagerStyle;

#[derive(Clone, Debug)]
pub struct PagerLook {
    /// Gap-jump glyph size in pixels.
    pub gap_icon_size: f32,
    /// Page-size border width in pixels.
    pub border_width: f32,
    /// Gap between the page-size label and trigger, in pixels.
    pub page_size_label_gap: f32,
    /// Vertical gap below the page-size trigger, in pixels.
    pub popup_offset_y: f32,
    /// Page-size popup gap in pixels.
    pub popup_gap: f32,
    /// Page-size popup padding in pixels.
    pub popup_padding: f32,
    /// Page-size popup item padding y in pixels.
    pub popup_item_padding_y: f32,
    /// Amount subtracted from the popup radius for each item, in pixels.
    pub popup_item_radius_inset: f32,

    pub panel_background: Hsla,
    pub border: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub hover_background: Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub typography: LumaTextStyle,
    pub button_size: f32,
    pub button_min_width: f32,
    pub control_height: f32,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub group_gap: f32,
    pub disabled_opacity: f32,
}

pub trait PagerTheme: Send + Sync {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook;

    fn button_template(&self, _pager_look: &PagerLook) -> Arc<dyn ButtonTemplate<()>> {
        default_button_template()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPagerTheme {
    tokens: ThemeTokens,
}

pub fn default_pager_theme() -> Arc<dyn PagerTheme> {
    static THEME: OnceLock<Arc<dyn PagerTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPagerTheme::default())).clone()
}

impl DefaultPagerTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PagerTheme for DefaultPagerTheme {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook {
        let palette = self.tokens.stylesheet.recipes.palette("pager", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = self.tokens.typography.text.caption;
        let compact = matches!(style, PagerStyle::Minimal);
        let control_height = metrics.control_height();
        // SDK pixel defaults; looks with their own spacing supply their own pager theme.
        let compact_gap = 8.0;
        let regular_gap = 16.0;
        let item_gap = PAGER_SPACING_UNIT_PX;
        let key = style.stylesheet_key();
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().pager.authored_geometry_for(key);
        let recipe = self.tokens.stylesheet.pager.authored_geometry_for(key).with_fallback(&baseline);
        let padding_x =
            (metrics.padding_x() - PAGER_SPACING_UNIT_PX * recipe.padding_x_inset_scale.unwrap_or_default()).max(0.0);
        let padding_y = metrics.padding_y() + metrics.gap() * recipe.padding_y_gap_scale.unwrap_or_default();

        let mut look = PagerLook {
            gap_icon_size: recipe.gap_icon_size.unwrap_or_default(),
            border_width: recipe.border_width.unwrap_or_default(),
            page_size_label_gap: recipe.page_size_label_gap.unwrap_or_default(),
            popup_offset_y: recipe.popup_offset_y.unwrap_or_default(),
            popup_gap: recipe.popup_gap.unwrap_or_default(),
            popup_padding: recipe.popup_padding.unwrap_or_default(),
            popup_item_padding_y: recipe.popup_item_padding_y.unwrap_or_default(),
            popup_item_radius_inset: recipe.popup_item_radius_inset.unwrap_or_default(),

            panel_background: if enabled {
                palette.surface.panel.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            body_text: if enabled {
                palette.app.foreground
            } else {
                palette.state.disabled.foreground
            },
            muted_text: if enabled {
                palette.app.muted_foreground
            } else {
                palette.state.disabled.foreground
            },
            selected_background: palette.state.selected.background,
            selected_foreground: palette.state.selected.foreground,
            hover_background: palette.state.hover.background,
            shadow: vec![],
            typography,
            button_size: control_height,
            button_min_width: control_height
                + PAGER_SPACING_UNIT_PX * recipe.button_extra_width_scale.unwrap_or_default(),
            control_height,
            radius: metrics.radius(),
            padding_x,
            padding_y,
            gap: item_gap,
            group_gap: if compact { compact_gap } else { regular_gap },
            disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
        };
        crate::theme::stylesheet::apply_common_geometry!(self.tokens.stylesheet.pager, key = key, PagerGeometry, look,
            [button_size => button_size, button_min_width => button_min_width, control_height => control_height,
             padding_x => padding_x, padding_y => padding_y, gap => gap, group_gap => group_gap, radius => radius,
             font_size => typography.size, line_height => typography.line_height, disabled_opacity => disabled_opacity],
            defaults = Default::default());
        look
    }
}

/// Pixel unit the pager's scale-based geometry multiplies.
const PAGER_SPACING_UNIT_PX: f32 = 4.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ControlMetricTokens, MetricTokens};

    #[test]
    fn default_pager_fallback_uses_metric_tokens() {
        let mut tokens = ThemeTokens { metrics: MetricTokens::default(), ..Default::default() };
        tokens.metrics.control = ControlMetricTokens::new(40.0, 18.0, 7.0, 10.0, 9.0, 14.0);

        let theme = DefaultPagerTheme::new(tokens);
        let regular = theme.resolve(true, PagerStyle::Numeric);
        let compact = theme.resolve(true, PagerStyle::Minimal);

        assert_eq!(regular.button_size, 40.0);
        assert_eq!(regular.button_min_width, 44.0);
        assert_eq!(regular.control_height, 40.0);
        assert_eq!(regular.radius, 9.0);
        assert_eq!(regular.padding_x, 16.0);
        assert_eq!(regular.padding_y, 12.0);
        assert_eq!(regular.gap, 4.0);
        assert_eq!(regular.group_gap, 16.0);
        assert_eq!(compact.button_min_width, 40.0);
        assert_eq!(compact.group_gap, 8.0);
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    PagerStylesheet,
    PagerGeometryStylesheet,
    PagerGeometry,
    ResolvedPagerGeometry,
    "pager",
    [
        gap_icon_size,
        button_size,
        button_min_width,
        control_height,
        padding_x,
        padding_y,
        gap,
        group_gap,
        font_size,
        line_height,
        radius,
        padding_x_inset_scale,
        padding_y_gap_scale,
        button_extra_width_scale,
        border_width,
        page_size_label_gap,
        popup_offset_y,
        popup_gap,
        popup_padding,
        popup_item_padding_y,
        popup_item_radius_inset
    ],
    unit[disabled_opacity]
);

impl PagerStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PagerGeometryStylesheet {
                gap_icon_size: Some(16.0),
                border_width: Some(1.0),
                page_size_label_gap: Some(8.0),
                popup_offset_y: Some(4.0),
                popup_gap: Some(2.0),
                popup_padding: Some(4.0),
                popup_item_padding_y: Some(4.0),
                popup_item_radius_inset: Some(2.0),
                padding_x_inset_scale: Some(1.0 / 2.0),
                padding_y_gap_scale: Some(1.0 / 2.0),
                button_extra_width_scale: Some(1.0),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: crate::theme::ControlSizes::from([(
                "minimal".into(),
                PagerGeometryStylesheet { button_extra_width_scale: Some(0.0), ..Default::default() },
            )]),
        }
    }
}
