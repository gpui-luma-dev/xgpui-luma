//! The table read from `.lss` (issue #285): `table.lss`. Looks keep their drag preview (the shared
//! `drag_preview`) and delegate the frame, header and rows to [`StyledTable`].

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{TableDragLook, TableLook, TableRowLook, TableRowPalette, TableTheme};
use crate::infra::drag_drop::DragDropLook;
use crate::style::StyleBinding;
use crate::theme::{InteractionState, ListRowScale, LumaTextStyle, MetricTokens};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

fn typography(part: PartView<'_>) -> LumaTextStyle {
    LumaTextStyle {
        size: part.font_size().unwrap_or(14.0),
        line_height: part.line_height().unwrap_or(20.0),
        weight: gpui::FontWeight(number(part, "font_weight").max(100.0)),
    }
}

/// The table's drag preview and insertion feedback: the shared drag preview's colors and radius with the
/// table's drag `geometry` (offset, width, padding, strokes).
pub fn table_drag_look(preview: DragDropLook, geometry: crate::theme::stylesheet::TableGeometry) -> TableDragLook {
    TableDragLook {
        geometry,
        background: preview.background,
        foreground: preview.foreground,
        valid_marker: preview.valid_marker,
        invalid_marker: preview.invalid_marker,
        radius: preview.radius,
    }
}

/// The table's `.lss` values for one binding (the `table` control).
#[derive(Clone)]
pub struct StyledTable {
    binding: StyleBinding,
}

impl StyledTable {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("table") }
    }

    /// The frame and header, enabled or not, focused or not.
    pub fn look(&self, enabled: bool, focused: bool) -> TableLook {
        let resolved = self.binding.resolve(&[("disabled", !enabled), ("focused", focused && enabled)], 1.0);
        let (root, header) = (resolved.root(), resolved.part(PartId::new("header")));
        TableLook {
            row_divider_width: number(root, "row_divider_width"),
            column_padding_x: number(root, "column_padding_x"),
            column_resize_hit_width: number(root, "column_resize_hit_width"),
            column_resize_line_width: number(root, "column_resize_line_width"),
            column_resize_vertical_extension: number(root, "column_resize_vertical_extension"),
            background: color(root, "background"),
            border: color(root, "border"),
            border_width: number(root, "border_width"),
            header_divider_width: number(root, "header_divider_width"),
            header_padding_bottom_scale: number(root, "header_padding_bottom_scale"),
            header_background: color(header, "background"),
            header_label_color: color(header, "foreground"),
            header_typography: typography(header),
            radius: number(root, "radius"),
            padding_x: number(root, "padding_x"),
            padding_y: number(root, "padding_y"),
        }
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let states = [
            ("selected", selected),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("focused", state.focused),
            ("disabled", state.disabled),
        ];
        self.binding.resolve(&states, 1.0)
    }

    /// A body row's colors, selected or not, in `state`.
    pub fn row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        let resolved = self.resolve_row(selected, state);
        let row = resolved.part(PartId::new("row"));
        TableRowPalette {
            background: color(row, "background"),
            label_color: color(row, "foreground"),
            divider: color(row, "border"),
            label_typography: typography(row),
        }
    }

    /// A body row's colors and layout, with padding and height from `scale`.
    pub fn row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        let resolved = self.resolve_row(selected, state);
        let (root, row) = (resolved.root(), resolved.part(PartId::new("row")));
        TableRowLook {
            column_emphasis_weight: gpui::FontWeight(number(root, "column_emphasis_weight").max(100.0)),
            row_divider_width: number(root, "row_divider_width"),
            column_padding_x: number(root, "column_padding_x"),
            column_muted_opacity: number(root, "column_muted_opacity"),
            disabled_opacity: number(root, "row_disabled_opacity"),
            background: color(row, "background"),
            label_color: color(row, "foreground"),
            divider: color(row, "border"),
            label_typography: typography(row),
            radius: number(row, "radius"),
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            min_height: scale.min_height,
            label_baseline_shift: number(root, "label_baseline_shift"),
        }
    }
}

/// The SDK's table theme: `table.lss`, with the SDK's drag preview.
pub struct StyledTableTheme(StyledTable);

impl StyledTableTheme {
    pub fn sdk() -> Self {
        Self(StyledTable::sdk())
    }
}

impl TableTheme for StyledTableTheme {
    fn resolve_drag(&self) -> TableDragLook {
        table_drag_look(
            crate::infra::drag_drop::styled_drag_drop_look(&StyleBinding::sdk("drag_preview")),
            crate::theme::stylesheet::CommonStylesheet::sdk_default().table.resolve_geometry(Default::default()),
        )
    }

    fn resolve_look(&self, enabled: bool, focused: bool) -> TableLook {
        self.0.look(enabled, focused)
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        self.0.row(selected, state)
    }

    fn resolve_row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        self.0.row_look(selected, state, scale)
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}
