//! The table read from `.lss` (issue #285): `table.lss`. Looks keep their drag preview (the shared
//! `drag_preview`) and delegate the frame, header and rows to [`StyledTable`].

#![allow(deprecated)] // `look`, `row` and `row_look` build the compatibility looks.

use crate::style::StyleStates;
use crate::style::PartRead;
use xgpui_luma_style::{PartId, PartView};

#[allow(deprecated)]
use super::theme::TableDragLook;
use super::theme::{TableGeometry, TableLook, TableRowLook, TableRowPalette, TableTheme};
#[allow(deprecated)]
use crate::infra::drag_drop::DragDropLook;
use crate::style::StyleBinding;
use crate::theme::{InteractionState, ListRowScale, LumaTextStyle, MetricTokens};

fn typography(part: PartView<'_>) -> LumaTextStyle {
    part.typography(14.0, 20.0)
}

/// The table's drag preview and insertion feedback: the shared drag preview's colors and radius with the
/// table's drag `geometry` (offset, width, padding, strokes).
#[deprecated(note = "use `TableTheme::drag_style` and `drag_geometry` (#433)")]
#[allow(deprecated)]
pub fn table_drag_look(preview: DragDropLook, geometry: TableGeometry) -> TableDragLook {
    TableDragLook {
        geometry,
        background: preview.background,
        foreground: preview.foreground,
        valid_marker: preview.valid_marker,
        invalid_marker: preview.invalid_marker,
        radius: preview.radius,
    }
}

/// A table row's layout from the row scale (#433): its padding and minimum height. Its colors, text and divider
/// are the resolved style's `row` part.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct TableRowLayout {
    pub padding_x: f32,
    pub padding_y: f32,
    /// The row's laid-out height (its padding sits inside it).
    pub min_height: f32,
}

impl TableRowLayout {
    /// The layout `scale` gives.
    pub fn new(scale: &ListRowScale) -> Self {
        Self { padding_x: scale.padding_x, padding_y: scale.padding_y, min_height: scale.min_height }
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

    /// The binding this reads.
    pub fn binding(&self) -> StyleBinding {
        self.binding.clone()
    }

    /// The frame and header's resolved style, enabled or not, focused or not (#433): read its root and `header`
    /// parts, and the table's properties (`row_divider_width`, `column_padding_x`, ...) from its root.
    pub fn resolve(&self, enabled: bool, focused: bool) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve_states(
            StyleStates::new(InteractionState {
                focused: focused && enabled,
                disabled: !enabled,
                ..Default::default()
            }),
            1.0,
        )
    }

    /// The frame and header, enabled or not, focused or not.
    #[deprecated(note = "read `resolve` instead (#433)")]
    pub fn look(&self, enabled: bool, focused: bool) -> TableLook {
        let resolved = self.resolve(enabled, focused);
        let (root, header) = (resolved.root(), resolved.part(PartId::new("header")));
        TableLook {
            row_divider_width: root.number("row_divider_width"),
            column_padding_x: root.number("column_padding_x"),
            column_resize_hit_width: root.number("column_resize_hit_width"),
            column_resize_line_width: root.number("column_resize_line_width"),
            column_resize_vertical_extension: root.number("column_resize_vertical_extension"),
            background: root.color("background"),
            border: root.color("border"),
            border_width: root.number("border_width"),
            header_divider_width: root.number("header_divider_width"),
            header_padding_bottom_scale: root.number("header_padding_bottom_scale"),
            header_background: header.color("background"),
            header_label_color: header.color("foreground"),
            header_typography: typography(header),
            radius: root.number("radius"),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
        }
    }

    /// A body row's resolved style, selected or not, in `state` (#433): read its `row` part.
    pub fn resolve_row(&self, selected: bool, state: InteractionState) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let states = StyleStates::new(state).selected(selected);
        self.binding.resolve_states(states, 1.0)
    }

    /// The row drag preview's geometry.
    pub fn drag_geometry(&self) -> TableGeometry {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        TableGeometry {
            drag_offset: root.number("drag_offset"),
            drag_width: root.number("drag_width"),
            drag_padding_x: root.number("drag_padding_x"),
            drag_padding_y: root.number("drag_padding_y"),
            drag_border_width: root.number("drag_border_width"),
            drag_marker_width: root.number("drag_marker_width"),
        }
    }

    /// A body row's colors, selected or not, in `state`.
    #[deprecated(note = "read `resolve_row` instead (#433)")]
    pub fn row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        let resolved = self.resolve_row(selected, state);
        let row = resolved.part(PartId::new("row"));
        TableRowPalette {
            background: row.color("background"),
            label_color: row.color("foreground"),
            divider: row.color("border"),
            label_typography: typography(row),
        }
    }

    /// A body row's colors and layout, with padding and height from `scale`.
    #[deprecated(note = "read `resolve_row` and a `TableRowLayout` instead (#433)")]
    pub fn row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        let resolved = self.resolve_row(selected, state);
        let (root, row) = (resolved.root(), resolved.part(PartId::new("row")));
        TableRowLook {
            column_emphasis_weight: gpui::FontWeight(root.number("column_emphasis_weight").max(100.0)),
            row_divider_width: root.number("row_divider_width"),
            column_padding_x: root.number("column_padding_x"),
            column_muted_opacity: root.number("column_muted_opacity"),
            opacity: row.opacity(),
            background: row.color("background"),
            label_color: row.color("foreground"),
            divider: row.color("border"),
            label_typography: typography(row),
            radius: row.number("radius"),
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            min_height: scale.min_height,
            label_baseline_shift: root.number("label_baseline_shift"),
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
    fn style(&self) -> StyleBinding {
        self.0.binding.clone()
    }

    fn drag_geometry(&self) -> TableGeometry {
        self.0.drag_geometry()
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

/// The table draws from the resolved style (#433): every value it reads matches what the old looks held, for the
/// frame and header in each state and a row in each state.
#[cfg(test)]
mod parity_tests {
    use super::*;

    #[test]
    fn the_frame_and_rows_draw_what_the_looks_held() {
        let styled = StyledTable::sdk();
        for (enabled, focused) in [(true, false), (true, true), (false, false)] {
            let look = styled.look(enabled, focused);
            let resolved = styled.resolve(enabled, focused);
            let (root, header) = (resolved.root(), resolved.part(PartId::new("header")));
            assert_eq!(
                (
                    root.color("background"),
                    root.color("border"),
                    header.color("background"),
                    header.color("foreground")
                ),
                (look.background, look.border, look.header_background, look.header_label_color)
            );
            assert_eq!(
                (root.number("border_width"), root.number("radius"), root.number("padding_y"), typography(header)),
                (look.border_width, look.radius, look.padding_y, look.header_typography)
            );
            assert_eq!(
                (root.number("column_resize_hit_width"), root.number("header_divider_width")),
                (look.column_resize_hit_width, look.header_divider_width)
            );
        }
        let scale = ListRowScale::compute(&MetricTokens::default(), 1.0);
        let layout = TableRowLayout::new(&scale);
        for selected in [false, true] {
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
                InteractionState { focused: true, ..Default::default() },
                InteractionState { disabled: true, ..Default::default() },
            ] {
                let look = styled.row_look(selected, state, &scale);
                let resolved = styled.resolve_row(selected, state);
                let (root, row) = (resolved.root(), resolved.part(PartId::new("row")));
                assert_eq!(
                    (row.color("background"), row.color("foreground"), row.color("border"), typography(row)),
                    (look.background, look.label_color, look.divider, look.label_typography),
                    "{selected} {state:?}"
                );
                assert_eq!(
                    (row.opacity(), root.number("row_divider_width"), root.number("label_baseline_shift")),
                    (look.opacity, look.row_divider_width, look.label_baseline_shift)
                );
                assert_eq!(
                    (layout.padding_x, layout.padding_y, layout.min_height),
                    (look.padding_x, look.padding_y, look.min_height)
                );
            }
        }
    }
}
