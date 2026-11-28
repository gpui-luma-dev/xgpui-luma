use std::sync::{Arc, OnceLock};

use crate::theme::{InteractionState, MetricTokens};

/// Theme-owned visuals for the compact drag preview and insertion feedback.
/// The row drag preview's geometry, from `table.lss` (its `drag_*` properties).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TableGeometry {
    /// The preview's offset from the pointer, both axes.
    pub drag_offset: f32,
    pub drag_width: f32,
    pub drag_padding_x: f32,
    pub drag_padding_y: f32,
    pub drag_border_width: f32,
    /// The insertion marker's width.
    pub drag_marker_width: f32,
}

pub trait TableTheme: Send + Sync {
    /// The table's style (#433): the `table` binding the template resolves for the frame, header and rows.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("table")
    }

    /// The body rows' style: [`Self::style`] unless a theme draws its rows apart from its frame (a design surface
    /// pinning states on the rows only).
    fn row_style(&self) -> crate::style::StyleBinding {
        self.style()
    }

    /// The row scale's metrics: row padding and height.
    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }

    /// The drag preview and insertion markers (#433): the SDK's `drag_preview` with the table's header colors, edge
    /// and corner set over it; themes can supply semantic accents.
    fn drag_style(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        use crate::style::PartRead;
        use xgpui_luma_style::PartId;
        let styled = super::styled::StyledTable::new(self.style());
        let frame = styled.resolve(true, true);
        let invalid = super::styled::StyledTable::new(self.row_style())
            .resolve_row(false, InteractionState { invalid: true, ..Default::default() });
        let (root, header) = (frame.root(), frame.part(PartId::new("header")));
        crate::infra::drag_drop::drag_preview_over(
            header.color("background"),
            header.color("foreground"),
            root.color("border"),
            invalid.part(PartId::new("row")).color("foreground"),
            root.number("radius"),
            header.typography(12.5, 18.0),
            None,
        )
    }

    /// The preview and insertion feedback geometry: offset, width, padding and strokes.
    fn drag_geometry(&self) -> TableGeometry {
        super::styled::StyledTable::sdk().drag_geometry()
    }
}

pub fn default_table_theme() -> Arc<dyn TableTheme> {
    static THEME: OnceLock<Arc<dyn TableTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTableTheme::sdk())).clone()
}
