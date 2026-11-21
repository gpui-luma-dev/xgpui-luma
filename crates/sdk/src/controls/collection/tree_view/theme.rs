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
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self::compute_with_stylesheet(metrics, scale_factor, crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }

    /// Derive the row layout from common appearance and shared metric tokens.
    pub fn compute_with_stylesheet(
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().tree_view.authored_geometry();
        let geometry = stylesheet.tree_view.authored_geometry().with_fallback(&baseline);
        Self {
            drag_marker_width: geometry.drag_marker_width.unwrap_or_default(),
            drag_insert_width: geometry.drag_insert_width.unwrap_or_default(),
            drag_opacity: geometry.drag_opacity.unwrap_or_default(),
            scrollbar_width: geometry.scrollbar_width.unwrap_or_default(),
            disabled_opacity: geometry.disabled_opacity.unwrap_or_default(),
            preview_padding_x: geometry.preview_padding_x.unwrap_or_default(),
            preview_padding_y: geometry.preview_padding_y.unwrap_or_default(),
            row_height: snap_to_pixel(
                geometry
                    .row_height
                    .unwrap_or_else(|| metrics.control_height() * geometry.row_height_scale.unwrap_or_default()),
                scale_factor,
            ),
            base_padding_x: snap_to_pixel(geometry.base_padding_x.unwrap_or_else(|| metrics.padding_x()), scale_factor),
            indentation_width: snap_to_pixel(geometry.indentation_width.unwrap_or_default(), scale_factor),
            inner_gap: snap_to_pixel(geometry.inner_gap.unwrap_or_else(|| metrics.gap()), scale_factor),
            radius: geometry.radius.unwrap_or_else(|| metrics.radius()),
            icon_size: snap_to_pixel(geometry.icon_size.unwrap_or_else(|| metrics.icon_size()), scale_factor),
            chevron_size: snap_to_pixel(
                geometry
                    .chevron_size
                    .unwrap_or_else(|| (metrics.icon_size() - geometry.chevron_inset.unwrap_or_default()).max(0.0)),
                scale_factor,
            ),
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

crate::theme::stylesheet::geometry::geometry_family!(
    TreeViewStylesheet,
    TreeViewGeometryStylesheet,
    TreeViewGeometry,
    ResolvedTreeViewGeometry,
    "tree_view",
    [
        row_height,
        row_height_scale,
        base_padding_x,
        indentation_width,
        inner_gap,
        radius,
        icon_size,
        chevron_size,
        chevron_inset,
        preview_padding_x,
        preview_padding_y, drag_marker_width, drag_insert_width, scrollbar_width
    ],
    unit[disabled_opacity, drag_opacity]
);

impl TreeViewStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TreeViewGeometryStylesheet {
                drag_marker_width: Some(1.0),
                drag_insert_width: Some(2.0),
                drag_opacity: Some(0.12),
                scrollbar_width: Some(12.0),
                disabled_opacity: Some(0.4),
                preview_padding_x: Some(12.0),
                preview_padding_y: Some(4.0),
                row_height_scale: Some(0.85),
                indentation_width: Some(16.0),
                chevron_inset: Some(2.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
