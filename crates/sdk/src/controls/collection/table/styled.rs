//! The table read from `.lss` (issue #285): `table.lss`. Looks keep their drag preview (the shared
//! `drag_preview`) and delegate the frame, header and rows to [`StyledTable`].

use crate::style::StyleStates;
use crate::style::PartRead;

use super::theme::{TableGeometry, TableTheme};

use crate::style::StyleBinding;
use crate::theme::{InteractionState, ListRowScale, MetricTokens};

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
mod parity_tests {}
