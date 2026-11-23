use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::scales::snap_to_pixel;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeViewScale {
    pub drag_marker_width: f32,
    pub drag_insert_width: f32,
    pub drag_opacity: f32,
    pub scrollbar_width: f32,
    /// Disabled-row opacity and drag-preview padding in pixels.
    pub disabled_opacity: f32,
    pub preview_padding_x: f32,
    pub preview_padding_y: f32,
    pub row_height: f32,
    pub base_padding_x: f32,
    pub indentation_width: f32,
    pub inner_gap: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl TreeViewScale {
    /// The row layout for a theme that supplies only metrics: fixed sizes from the SDK's
    /// `tree_view.lss`, the rest from `metrics`, so the theme keeps its own spacing.
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        let sdk = |property: &str| crate::style::sdk_number("tree_view", property);
        // `.lss` rows are a share of the control height, and the chevron sits inset in the icon size.
        let row_share = sdk("row_height_scale");
        let chevron_inset = sdk("chevron_inset");
        Self {
            drag_marker_width: sdk("drag_marker_width"),
            drag_insert_width: sdk("drag_insert_width"),
            drag_opacity: sdk("drag_opacity"),
            scrollbar_width: sdk("scrollbar_width"),
            disabled_opacity: sdk("disabled_opacity"),
            preview_padding_x: sdk("preview_padding_x"),
            preview_padding_y: sdk("preview_padding_y"),
            row_height: snap_to_pixel(metrics.control_height() * row_share, scale_factor),
            base_padding_x: snap_to_pixel(metrics.padding_x(), scale_factor),
            indentation_width: snap_to_pixel(sdk("indentation_width"), scale_factor),
            inner_gap: snap_to_pixel(metrics.gap(), scale_factor),
            radius: metrics.radius(),
            icon_size: snap_to_pixel(metrics.icon_size(), scale_factor),
            chevron_size: snap_to_pixel((metrics.icon_size() - chevron_inset).max(0.0), scale_factor),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeViewPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait TreeViewTheme: Send + Sync {
    /// Elevation for the detached drag preview, independent of row chrome.
    fn drag_preview_shadow(&self) -> Vec<gpui::BoxShadow> {
        ThemeTokens::default().elevation.drag_preview.to_box_shadows()
    }

    /// Resolve default file/folder glyphs; node-supplied icons take precedence.
    fn icon(&self, role: crate::theme::stylesheet::IconRole) -> crate::infra::icon::IconSource {
        crate::theme::stylesheet::CommonStylesheet::sdk_default().icons.resolve(role)
    }
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette;
    /// Optional inset row outline, painted without changing row geometry.
    fn row_outline(&self, _state: InteractionState) -> Option<Hsla> {
        None
    }
    fn metrics(&self) -> MetricTokens;

    /// Resolve row layout; themes may use the same common schema as SDK defaults.
    fn resolve_scale(&self, scale_factor: f32) -> TreeViewScale {
        TreeViewScale::compute(&self.metrics(), scale_factor)
    }
}

pub fn default_tree_view_theme() -> Arc<dyn TreeViewTheme> {
    static THEME: OnceLock<Arc<dyn TreeViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTreeViewTheme::sdk())).clone()
}
