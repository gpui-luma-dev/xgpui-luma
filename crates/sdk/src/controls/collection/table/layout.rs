use super::model::TableScrollMode;
use super::styled::TableRowLayout;
use crate::style::PartRead;

/// GPUI row elements use `min_h(min_height)` with `py(padding_y)` on the same node, so the
/// laid-out row height is [`TableRowLayout::min_height`], not min_height + padding.
///
/// Per-row layout height for [`TableRowLayout::min_height`] rows (see module comment).
pub fn default_row_height(layout: &TableRowLayout) -> f32 {
    layout.min_height
}

pub fn visible_row_height(layout: &TableRowLayout, override_height: Option<f32>) -> f32 {
    override_height.unwrap_or_else(|| default_row_height(layout))
}

/// Matches [`DefaultTableShellTemplate::paint_shell`] header slot padding, from the frame's resolved style
/// (#433).
pub fn header_height(style: &xgpui_luma_style::Resolved) -> f32 {
    let (root, header) = (style.root(), style.part(xgpui_luma_style::PartId::new("header")));
    let padding_y = root.number("padding_y");
    header.typography(14.0, 20.0).line_height
        + padding_y
        + (padding_y * root.number("header_padding_bottom_scale"))
        + root.number("header_divider_width")
}

/// Total scroll-body height for `visible_rows` data rows, including inter-row dividers.
pub fn body_rows_height(visible_rows: usize, row_height: f32, divider_width: f32) -> f32 {
    let row_count = visible_rows as f32;
    let dividers = visible_rows.saturating_sub(1) as f32 * divider_width;
    row_count * row_height + dividers
}

/// How many full rows fit when row dividers are painted *outside* the row box
/// (see [`body_rows_height`]). Prefer [`rows_for_viewport_height_border_box`] when
/// dividers are `border_t` on the row (inside `.h(...)`).
///
/// Returns `None` for unknown heights or invalid divider geometry.
pub fn rows_for_viewport_height(viewport_height: f32, row_height: f32, divider_width: f32) -> Option<usize> {
    if viewport_height <= 0.0
        || !viewport_height.is_finite()
        || row_height <= 0.0
        || !row_height.is_finite()
        || !divider_width.is_finite()
        || divider_width < 0.0
    {
        return None;
    }
    let stride = row_height + divider_width;
    if !stride.is_finite() || !(viewport_height + divider_width).is_finite() {
        return None;
    }
    let rows = ((viewport_height + divider_width) / stride).floor() as usize;
    Some(rows.max(1))
}

/// How many full rows fit when dividers are inside the row height (GPUI `border_t` on
/// a row with an explicit `.h(...)`).
pub fn rows_for_viewport_height_border_box(viewport_height: f32, row_height: f32) -> Option<usize> {
    if viewport_height <= 0.0 || !viewport_height.is_finite() || row_height <= 0.0 || !row_height.is_finite() {
        return None;
    }
    Some(((viewport_height / row_height).floor() as usize).max(1))
}

/// Per-row height that makes `row_count` rows fill `viewport_height` exactly.
///
/// Assumes row dividers are border-box (painted inside each row's height), matching
/// fill-height list rows that paint the resolved divider inside `.h(height)`.
pub fn distributed_row_height(viewport_height: f32, row_count: usize) -> Option<f32> {
    if row_count == 0 || viewport_height <= 0.0 || !viewport_height.is_finite() {
        return None;
    }
    Some(viewport_height / row_count as f32)
}

/// The shell's height for `visible_rows`, from the frame's resolved style (#433).
pub fn compute_shell_height(
    visible_rows: usize,
    row_height: f32,
    style: &xgpui_luma_style::Resolved,
    has_header: bool,
) -> f32 {
    let root = style.root();
    let mut total_height = body_rows_height(visible_rows, row_height, root.number("row_divider_width"));

    if has_header {
        total_height += header_height(style);
    }

    total_height + (2.0 * root.number("border_width"))
}

/// Resolves the row count used for fixed shell/body height.
///
/// When [`fill_height`](super::model::TableBuilder::fill_height) is set, only an explicit
/// `visible_rows` value produces a fixed height; otherwise the shell fills its parent (`h_full`).
/// Paged lists without fill-height still size from `page_size` when `visible_rows` is unset.
pub fn effective_visible_rows(
    visible_rows: Option<usize>,
    scroll_mode: TableScrollMode,
    fill_height: bool,
) -> Option<usize> {
    if fill_height {
        return visible_rows;
    }
    visible_rows.or(match scroll_mode {
        TableScrollMode::Paged { page_size } => Some(page_size),
        _ => None,
    })
}

pub fn page_count(item_count: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    if item_count == 0 {
        1
    } else {
        item_count.div_ceil(page_size)
    }
}

pub fn page_start(current_page: usize, page_size: usize) -> usize {
    current_page.saturating_mul(page_size.max(1))
}

pub fn visible_item_count(item_count: usize, current_page: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    let start = page_start(current_page, page_size);
    if start >= item_count {
        0
    } else {
        (start + page_size).min(item_count) - start
    }
}

pub fn clamp_page(current_page: usize, item_count: usize, page_size: usize) -> usize {
    page_count(item_count, page_size).saturating_sub(1).min(current_page)
}

/// Page index after a page-size change that keeps the current context in view.
///
/// When `keep_in_view` is a row on the previous page (typically the selected / active row),
/// the new page is chosen so that row stays visible. Otherwise picks the new page with
/// maximum overlap against the previous page's item range (ties break toward the closer
/// window start so growing the viewport does not snap to page 0).
pub fn page_after_size_change(
    current_page: usize,
    old_page_size: usize,
    new_page_size: usize,
    item_count: usize,
    keep_in_view: Option<usize>,
) -> usize {
    let old_page_size = old_page_size.max(1);
    let new_page_size = new_page_size.max(1);
    if item_count == 0 {
        return 0;
    }

    let old_start = page_start(current_page, old_page_size);
    let old_end = old_start + visible_item_count(item_count, current_page, old_page_size);
    if old_end <= old_start {
        return clamp_page(0, item_count, new_page_size);
    }

    if let Some(index) = keep_in_view
        && index < item_count
        && index >= old_start
        && index < old_end
    {
        return clamp_page(index / new_page_size, item_count, new_page_size);
    }

    let pages = page_count(item_count, new_page_size);
    let mut best_page = 0;
    let mut best_overlap = 0usize;
    let mut best_distance = usize::MAX;

    for page in 0..pages {
        let start = page_start(page, new_page_size);
        let end = (start + new_page_size).min(item_count);
        let overlap = old_end.min(end).saturating_sub(old_start.max(start));
        let distance = start.abs_diff(old_start);
        if overlap > best_overlap || (overlap == best_overlap && distance < best_distance) {
            best_overlap = overlap;
            best_distance = distance;
            best_page = page;
        }
    }

    best_page
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::StyleBinding;
    use xgpui_luma_style::{Overrides, PartId, Prop};

    /// The SDK frame with a 14px header line, 4px padding, a 0.75 bottom scale and a 1px divider.
    fn sample_frame() -> std::sync::Arc<xgpui_luma_style::Resolved> {
        let root = PartId::ROOT;
        let overrides = Overrides::new()
            .set(root, Prop::<f32>::new("padding_y"), 4.0)
            .set(root, Prop::<f32>::new("header_padding_bottom_scale"), 0.75)
            .set(root, Prop::<f32>::new("header_divider_width"), 1.0)
            .set(PartId::new("header"), Prop::<f32>::new("line_height"), 14.0);
        super::super::StyledTable::new(StyleBinding::sdk("table").overrides(overrides)).resolve(true, false)
    }

    #[test]
    fn default_row_height_matches_gpui_min_h_layout() {
        let row = TableRowLayout { padding_x: 8.0, padding_y: 8.0, min_height: 36.0 };
        assert_eq!(default_row_height(&row), 36.0);
    }

    #[test]
    fn body_rows_height_includes_dividers() {
        assert_eq!(body_rows_height(5, 36.0, 1.0), 5.0 * 36.0 + 4.0);
        assert_eq!(body_rows_height(5, 36.0, 0.0), 180.0);
        assert_eq!(body_rows_height(5, 36.0, 3.0), 192.0);
    }

    #[test]
    fn header_height_matches_shell_header_slot() {
        assert_eq!(header_height(&sample_frame()), 14.0 + 4.0 + 3.0 + 1.0);
    }

    #[test]
    fn page_count_handles_empty_items() {
        assert_eq!(page_count(0, 10), 1);
        assert_eq!(page_count(25, 10), 3);
    }

    #[test]
    fn visible_item_count_on_last_page() {
        assert_eq!(visible_item_count(25, 2, 10), 5);
        assert_eq!(visible_item_count(25, 0, 10), 10);
    }

    #[test]
    fn clamp_page_stays_in_bounds() {
        assert_eq!(clamp_page(99, 25, 10), 2);
    }

    #[test]
    fn page_after_size_change_maximizes_overlap_with_old_page() {
        // Shrink: page 2 @ 20 → [40, 60); @ 10 → page 4 starts at 40.
        assert_eq!(page_after_size_change(2, 20, 10, 100, None), 4);
        assert_eq!(page_start(4, 10), 40);

        // Grow: page 1 @ 20 → [20, 40); @ 30 → prefer page 1 (start 30) over page 0.
        assert_eq!(page_after_size_change(1, 20, 30, 100, None), 1);
        assert_eq!(page_start(1, 30), 30);

        // Grow: page 2 @ 10 → [20, 30); @ 25 → page 1 (start 25) over page 0.
        assert_eq!(page_after_size_change(2, 10, 25, 100, None), 1);
    }

    #[test]
    fn page_after_size_change_keeps_selected_row_on_current_page_in_view() {
        // Page 2 @ 20 → [40, 60); selected 55; shrink to 10 → page containing 55.
        assert_eq!(page_after_size_change(2, 20, 10, 100, Some(55)), 5);
        assert_eq!(page_start(5, 10), 50);

        // Selected row on another page must not override overlap (stale selection on page 0).
        assert_eq!(page_after_size_change(2, 20, 10, 100, Some(1)), 4);
    }

    #[test]
    fn effective_visible_rows_paged_uses_page_size() {
        assert_eq!(effective_visible_rows(None, TableScrollMode::Paged { page_size: 25 }, false), Some(25));
    }

    #[test]
    fn effective_visible_rows_fill_height_skips_page_size() {
        assert_eq!(effective_visible_rows(None, TableScrollMode::Paged { page_size: 25 }, true), None);
        assert_eq!(effective_visible_rows(Some(10), TableScrollMode::Paged { page_size: 25 }, true), Some(10));
    }

    #[test]
    fn rows_for_viewport_height_accounts_for_dividers() {
        assert_eq!(rows_for_viewport_height(0.0, 36.0, 1.0), None);
        for divider in [-1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(rows_for_viewport_height(184.0, 36.0, divider), None);
        }
        assert_eq!(rows_for_viewport_height(f32::MAX, 36.0, f32::MAX), None);
        // 5 rows at 36px + 4 dividers = 184
        assert_eq!(rows_for_viewport_height(184.0, 36.0, 1.0), Some(5));
        assert_eq!(rows_for_viewport_height(183.0, 36.0, 1.0), Some(4));
        assert_eq!(rows_for_viewport_height(1.0, 36.0, 1.0), Some(1));
    }

    #[test]
    fn rows_for_viewport_height_border_box_ignores_divider_stride() {
        assert_eq!(rows_for_viewport_height_border_box(180.0, 36.0), Some(5));
        assert_eq!(rows_for_viewport_height_border_box(179.0, 36.0), Some(4));
        assert_eq!(rows_for_viewport_height_border_box(0.0, 36.0), None);
    }

    #[test]
    fn distributed_row_height_fills_viewport() {
        // Border-box: n rows share the viewport evenly (dividers inside row height).
        assert!((distributed_row_height(200.0, 5).unwrap() - 40.0).abs() < f32::EPSILON);
        assert!((distributed_row_height(200.0, 5).unwrap() * 5.0 - 200.0).abs() < f32::EPSILON);
        assert_eq!(distributed_row_height(100.0, 0), None);
    }
}

#[cfg(test)]
mod stroke_layout_tests {
    #[test]
    fn shell_height_uses_resolved_strokes_and_header_padding() {
        use crate::style::{PartRead, StyleBinding};
        use xgpui_luma_style::{Overrides, PartId, Prop};
        let frame = |border_width: f32| {
            let root = PartId::ROOT;
            let overrides = Overrides::new()
                .set(root, Prop::<f32>::new("border_width"), border_width)
                .set(root, Prop::<f32>::new("header_divider_width"), 3.0)
                .set(root, Prop::<f32>::new("padding_y"), 8.0)
                .set(root, Prop::<f32>::new("header_padding_bottom_scale"), 0.5);
            crate::controls::table::StyledTable::new(StyleBinding::sdk("table").overrides(overrides))
                .resolve(true, false)
        };
        let style = frame(0.0);
        let header = style.part(PartId::new("header")).typography(14.0, 20.0).line_height + 8.0 + 4.0 + 3.0;
        assert_eq!(super::compute_shell_height(1, 30.0, &style, true), 30.0 + header);
        assert_eq!(super::compute_shell_height(1, 30.0, &frame(2.0), false), 34.0);
    }
}
