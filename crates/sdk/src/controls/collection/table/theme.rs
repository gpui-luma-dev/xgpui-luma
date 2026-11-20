use crate::theme::stylesheet::CommonStylesheet;

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionLayer, InteractionState, ListRowScale, LumaTextStyle, MetricTokens, ThemeTokens};

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

#[derive(Clone, Debug, Default)]
pub struct DefaultTableTheme {
    tokens: ThemeTokens,
}

pub fn default_table_theme() -> Arc<dyn TableTheme> {
    static THEME: OnceLock<Arc<dyn TableTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTableTheme::sdk())).clone()
}

impl DefaultTableTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TableTheme for DefaultTableTheme {
    fn resolve_drag(&self) -> TableDragLook {
        let palette = self.tokens.stylesheet.recipes.palette("table", &self.tokens.palette);
        TableDragLook {
            geometry: self.tokens.stylesheet.table.with_sdk_defaults().resolve_geometry(Default::default()),
            background: palette.surface.floating.background,
            foreground: palette.surface.floating.foreground,
            valid_marker: palette.focus.ring,
            invalid_marker: palette.form.input.invalid_border,
            radius: self.tokens.metrics.radius(),
        }
    }

    fn resolve_look(&self, enabled: bool, _focused: bool) -> TableLook {
        let palette = self.tokens.stylesheet.recipes.palette("table", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().table.authored_geometry();
        let recipe = self.tokens.stylesheet.table.authored_geometry().with_fallback(&baseline);
        let mut look = TableLook {
            row_divider_width: recipe.row_divider_width.unwrap_or_default(),
            column_padding_x: recipe.column_padding_x.unwrap_or_default(),
            column_resize_hit_width: recipe.column_resize_hit_width.unwrap_or_default(),
            column_resize_line_width: recipe.column_resize_line_width.unwrap_or_default(),
            column_resize_vertical_extension: recipe.column_resize_vertical_extension.unwrap_or_default(),

            background: if enabled {
                palette.form.input.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            border_width: recipe.border_width.unwrap_or_default(),
            header_divider_width: recipe.header_divider_width.unwrap_or_default(),
            header_padding_bottom_scale: recipe.header_padding_bottom_scale.unwrap_or_default(),
            header_background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            header_label_color: if enabled {
                palette.app.muted_foreground
            } else {
                palette.state.disabled.foreground
            },
            header_typography: self.tokens.typography.text.caption,
            radius: metrics.radius(),
            padding_x: recipe.padding_x.unwrap_or_default(),
            padding_y: metrics.padding_y() * recipe.padding_y_scale.unwrap_or_default(),
        };
        crate::theme::stylesheet::apply_common_geometry!(self.tokens.stylesheet.table, TableGeometry, look,
            [padding_x => padding_x, padding_y => padding_y], defaults = Default::default());
        look
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        let palette = self.tokens.stylesheet.recipes.palette("table", &self.tokens.palette);
        let typography = &self.tokens.typography;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };
        let layer = state.layer();

        let selected_background = palette.surface.subtle.background;
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().table.authored_geometry();
        let recipe = self.tokens.stylesheet.table.authored_geometry().with_fallback(&baseline);
        let hover_background = Hsla { a: recipe.row_hover_opacity.unwrap_or_default(), ..palette.data.accent_1 };

        let background = if state.disabled && !state.focused {
            transparent
        } else if selected || state.focused {
            selected_background
        } else {
            match layer {
                InteractionLayer::Disabled => transparent,
                InteractionLayer::Pressed => palette.state.pressed.background,
                InteractionLayer::Hovered => hover_background,
                InteractionLayer::Default => transparent,
            }
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else if selected {
            palette.surface.subtle.foreground
        } else {
            palette.app.foreground
        };

        let mut label_typography = typography.text.label;
        crate::controls::textfield::apply_control_typography(&mut label_typography, typography);

        TableRowPalette { background, label_color, divider: palette.form.input.border, label_typography }
    }

    fn resolve_row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        let mut look = TableRowLook::from_palette(&self.resolve_row(selected, state), scale, &self.tokens.stylesheet);
        look.column_emphasis_weight = self.tokens.typography.text.title.weight;
        look
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
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
