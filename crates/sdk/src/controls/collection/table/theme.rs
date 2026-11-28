#![allow(deprecated)] // TableLook, TableRowLook and TableRowPalette are this module's compatibility types.

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionState, ListRowScale, LumaTextStyle, MetricTokens};

/// The frame and header's values, copied out of the table's style (#433). A compatibility type: the template
/// draws from the resolved style ([`StyledTable::resolve`](super::StyledTable::resolve)).
#[deprecated(note = "read the resolved style from `TableTheme::style()` instead (#433)")]
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

/// A row's colors, copied out of the table's style (#433). A compatibility type.
#[deprecated(note = "read the resolved style from `TableTheme::style()` instead (#433)")]
#[derive(Clone, Debug)]
pub struct TableRowPalette {
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub label_typography: LumaTextStyle,
}

/// A row's colors and layout, copied out of the table's style (#433). A compatibility type: rows draw from the
/// resolved style ([`StyledTable::resolve_row`](super::StyledTable::resolve_row)) and a
/// [`TableRowLayout`](super::TableRowLayout).
#[deprecated(note = "read the resolved style from `TableTheme::style()` and a `TableRowLayout` instead (#433)")]
#[derive(Clone, Copy, Debug)]
pub struct TableRowLook {
    pub column_emphasis_weight: gpui::FontWeight,
    /// Inter-row divider thickness in pixels.
    pub row_divider_width: f32,
    /// Horizontal text-column inset in pixels.
    pub column_padding_x: f32,
    /// Opacity of default muted cells.
    pub column_muted_opacity: f32,
    /// The row's opacity: the `row` part's resolved `opacity`, which a look sets under `:disabled`.
    pub opacity: f32,
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

#[derive(Clone, Debug)]
pub struct TableDragLook {
    /// Preview and insertion feedback geometry, from `table.lss`.
    pub geometry: TableGeometry,
    pub background: Hsla,
    pub foreground: Hsla,
    pub valid_marker: Hsla,
    pub invalid_marker: Hsla,
    pub radius: f32,
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

    /// Defaults to existing theme surfaces; themes can supply semantic accents.
    fn resolve_drag(&self) -> TableDragLook {
        use crate::style::PartRead;
        use xgpui_luma_style::PartId;
        let styled = super::styled::StyledTable::new(self.style());
        let frame = styled.resolve(true, true);
        let invalid = super::styled::StyledTable::new(self.row_style())
            .resolve_row(false, InteractionState { invalid: true, ..Default::default() });
        let (root, header) = (frame.root(), frame.part(PartId::new("header")));
        TableDragLook {
            geometry: super::styled::StyledTable::sdk().drag_geometry(),
            background: header.color("background"),
            foreground: header.color("foreground"),
            valid_marker: root.color("border"),
            invalid_marker: invalid.part(PartId::new("row")).color("foreground"),
            radius: root.number("radius"),
        }
    }

    /// The frame and header built from [`Self::style`], for code that still reads a [`TableLook`].
    #[deprecated(note = "read the resolved style from `style()` instead (#433)")]
    fn resolve_look(&self, enabled: bool, focused: bool) -> TableLook {
        super::styled::StyledTable::new(self.style()).look(enabled, focused)
    }

    /// A row's colors built from [`Self::style`], for code that still reads a [`TableRowPalette`].
    #[deprecated(note = "read the resolved style from `style()` instead (#433)")]
    fn resolve_row(&self, selected: bool, state: InteractionState) -> TableRowPalette {
        super::styled::StyledTable::new(self.row_style()).row(selected, state)
    }

    /// A row built from [`Self::style`], for code that still reads a [`TableRowLook`].
    #[deprecated(note = "read the resolved style from `style()` and a `TableRowLayout` instead (#433)")]
    fn resolve_row_look(&self, selected: bool, state: InteractionState, scale: &ListRowScale) -> TableRowLook {
        super::styled::StyledTable::new(self.row_style()).row_look(selected, state, scale)
    }
}

pub fn default_table_theme() -> Arc<dyn TableTheme> {
    static THEME: OnceLock<Arc<dyn TableTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledTableTheme::sdk())).clone()
}

impl TableRowLook {
    /// Combines row colors with the SDK's `table.lss` row layout and label baseline.
    #[deprecated(note = "rows draw from the resolved style (#433)")]
    pub fn from_palette(palette: &TableRowPalette, scale: &ListRowScale) -> Self {
        let sdk = super::styled::StyledTable::sdk().row_look(false, InteractionState::default(), scale);
        Self {
            column_emphasis_weight: crate::theme::LumaTypography::default().text.title.weight,
            background: palette.background,
            label_color: palette.label_color,
            divider: palette.divider,
            label_typography: palette.label_typography,
            label_baseline_shift: crate::theme::scales::label_baseline_shift(),
            ..sdk
        }
    }
}
