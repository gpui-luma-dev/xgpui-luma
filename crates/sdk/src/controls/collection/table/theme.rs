use crate::theme::stylesheet::CommonStylesheet;

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionState, ListRowScale, LumaTextStyle, MetricTokens};

#[derive(Clone, Debug)]
pub struct TableLook {
    /// Inter-row divider thickness in pixels.
    pub row_divider_width: f32,
    /// Horizontal text-column inset shared by header and body, in pixels.
    pub column_padding_x: f32,
    /// Column resize hit target width in pixels.
    pub column_resize_hit_width: f32,
    /// Painted column resize stroke thickness in pixels.
    pub column_resize_line_width: f32,
    /// Hit target extension above and below the header in pixels.
    pub column_resize_vertical_extension: f32,
    pub background: Hsla,
    pub border: Hsla,
    /// Outer frame thickness in pixels.
    pub border_width: f32,
    /// Header separator thickness in pixels.
    pub header_divider_width: f32,
    /// Header bottom padding divided by resolved vertical padding.
    pub header_padding_bottom_scale: f32,
    pub header_background: Hsla,
    pub header_label_color: Hsla,
    pub header_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl TableLook {
    /// Returns the shared radius for all painted Table surface layers.
    ///
    /// The shell border is inset by GPUI during paint, but independently
    /// subtracting that width from child layers produces divergent corner
    /// paths at large theme radii. Layer roots must use the same resolved
    /// radius; the shell's clip/border handles the inset.
    pub fn inner_radius(&self, _border_width: f32) -> f32 {
        self.radius.max(0.0)
    }
}

#[derive(Clone, Debug)]
pub struct TableRowPalette {
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub label_typography: LumaTextStyle,
}

#[derive(Clone, Copy, Debug)]
pub struct TableRowLook {
    pub column_emphasis_weight: gpui::FontWeight,
    /// Inter-row divider thickness in pixels.
    pub row_divider_width: f32,
    /// Horizontal text-column inset in pixels.
    pub column_padding_x: f32,
    /// Opacity of default muted cells.
    pub column_muted_opacity: f32,
    /// Whole-row opacity applied only when disabled.
    pub disabled_opacity: f32,
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub min_height: f32,
    pub label_baseline_shift: f32,
}

/// Theme-owned visuals for the compact drag preview and insertion feedback.
#[derive(Clone, Debug)]
pub struct TableDragLook {
    /// Captured, size-resolved preview and insertion feedback geometry.
    pub geometry: crate::theme::stylesheet::TableGeometry,
    pub background: Hsla,
    pub foreground: Hsla,
    pub valid_marker: Hsla,
    pub invalid_marker: Hsla,
    pub radius: f32,
}

pub trait TableTheme: Send + Sync {
    /// Defaults to existing theme surfaces; themes can supply semantic accents.
    fn resolve_drag(&self) -> TableDragLook {
        let look = self.resolve_look(true, true);
        let invalid = self.resolve_row(false, InteractionState { invalid: true, ..Default::default() });
        TableDragLook {
            geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .table
                .resolve_geometry(Default::default()),
            background: look.header_background,
            foreground: look.header_label_color,
            valid_marker: look.border,
            invalid_marker: invalid.label_color,
            radius: look.radius,
        }
    }

    fn resolve_look(&self, enabled: bool, focused: bool) -> TableLook;
    fn resolve_row(&self, selected: bool, state: InteractionState) -> TableRowPalette;
    fn metrics(&self) -> MetricTokens;

    fn resolve_row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        TableRowLook::from_palette(
            &self.resolve_row(selected, state),
            scale,
            crate::theme::stylesheet::CommonStylesheet::sdk_default(),
        )
    }
}

pub fn default_table_theme() -> Arc<dyn TableTheme> {
    static THEME: OnceLock<Arc<dyn TableTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTableTheme::sdk())).clone()
}

impl TableRowLook {
    /// Combines row colors and metrics with shared, size-specific row styling.
    pub fn from_palette(
        palette: &TableRowPalette,
        scale: &ListRowScale,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().table.authored_geometry();
        let recipe = stylesheet.table.authored_geometry().with_fallback(&baseline);
        Self {
            column_emphasis_weight: stylesheet.tokens.typography.text.title.weight,
            column_padding_x: recipe.column_padding_x.unwrap_or_default(),
            column_muted_opacity: recipe.column_muted_opacity.unwrap_or_default(),
            row_divider_width: recipe.row_divider_width.unwrap_or_default(),
            disabled_opacity: recipe.row_disabled_opacity.unwrap_or_default(),
            background: palette.background,
            label_color: palette.label_color,
            divider: palette.divider,
            label_typography: palette.label_typography,
            radius: recipe.row_radius.unwrap_or_default(),
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            min_height: scale.min_height,
            label_baseline_shift: crate::theme::scales::label_baseline_shift(stylesheet),
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    TableStylesheet,
    TableGeometryStylesheet,
    TableGeometry,
    ResolvedTableGeometry,
    "table",
    [
        row_divider_width,
        padding_x,
        padding_y,
        padding_y_scale,
        row_radius,
        border_width,
        header_divider_width,
        header_padding_bottom_scale,
        drag_offset,
        drag_width,
        drag_padding_x,
        drag_padding_y,
        drag_border_width,
        drag_marker_width,
        column_padding_x,
        column_resize_hit_width,
        column_resize_line_width,
        column_resize_vertical_extension
    ],
    unit[row_hover_opacity, row_disabled_opacity, column_muted_opacity]
);

impl TableStylesheet {
    /// Fills absent table fields from SDK defaults without copying unrelated families.
    pub fn with_sdk_defaults(&self) -> Self {
        self.clone().with_baseline(&CommonStylesheet::sdk_default().table)
    }
}

impl TableStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TableGeometryStylesheet {
                row_divider_width: Some(1.0),
                column_padding_x: Some(12.0),
                column_muted_opacity: Some(0.65),
                column_resize_hit_width: Some(10.0),
                column_resize_line_width: Some(1.0),
                column_resize_vertical_extension: Some(4.0),
                drag_offset: Some(12.0),
                drag_width: Some(220.0),
                drag_padding_x: Some(12.0),
                drag_padding_y: Some(8.0),
                drag_border_width: Some(1.0),
                drag_marker_width: Some(2.0),
                border_width: Some(1.0),
                header_divider_width: Some(1.0),
                header_padding_bottom_scale: Some(3.0 / 4.0),
                padding_x: Some(0.0),
                padding_y_scale: Some(1.0 / 2.0),
                row_radius: Some(0.0),
                row_hover_opacity: Some(0.4),
                row_disabled_opacity: Some(0.56),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
