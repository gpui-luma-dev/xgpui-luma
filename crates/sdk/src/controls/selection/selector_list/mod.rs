//! Shared selector item-list chrome used by [`super::selector`], combobox, and autocomplete.
//!
//! This is not a spawnable control. Prefer [`super::selector`] (popup dropdown) or
//! [`super::selection_panel`] (listbox-in-a-panel).

mod item_template;
mod items_template;
mod model;

pub use item_template::{SelectorItemRenderModel, SelectorItemTemplate, make_selector_item_template};
pub use items_template::{
    SelectorPopupGeometry, SelectorPopupGeometryHandler, DefaultSelectorItemsTemplate, SelectorItemsPanelLook,
    SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers, SelectorItemsTemplateModifier,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, SelectorPanelMouseDownHandler,
    SelectorPanelScrollWheelHandler, default_selector_items_panel_look, default_selector_items_template,
    selector_items_panel_source, styled_selector_items_panel_look,
};
pub use model::{SelectorItem, SelectorItemLike, SelectorPath, normalize_selector_items};
pub(crate) use items_template::items_template_with_modifier;

/// A list popup's placement next to its trigger, and how many rows fit there (#433).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ListPopupFit {
    pub placement: crate::infra::popup_placement::ResolvedPopupPlacement,
    /// Rows that fit in the space on the chosen side; at least one.
    pub max_rows: usize,
}

/// Place a row list below `trigger` (above when it fits better) and count the
/// rows that fit. `chrome` is the height of anything above the rows, such as a
/// search field; `wanted_rows` is how many rows the popup would show unlimited.
pub(crate) fn fit_list_popup(
    trigger: gpui::Bounds<gpui::Pixels>,
    width: gpui::Pixels,
    look: &SelectorItemsPanelLook,
    chrome: gpui::Pixels,
    wanted_rows: usize,
    viewport: impl Into<crate::infra::popup_placement::PopupSpace>,
) -> ListPopupFit {
    use crate::infra::popup_placement::{PopupPlacementRequest, PopupSide};
    use gpui::px;

    let frame = chrome + px(look.padding * 2.0);
    let row = px(look.item_height);
    let placement = PopupPlacementRequest::new(trigger, PopupSide::Below, viewport)
        .flip(true)
        .gap(px(look.popup_offset_y))
        .content_size(Some(gpui::size(width, frame + row * wanted_rows as f32)))
        .window_margin(px(look.window_margin))
        .min_main_size(frame + row)
        .resolve();
    let max_rows = if look.item_height > 0.0 {
        ((placement.max_size.height - frame) / row).floor().max(1.0) as usize
    } else {
        wanted_rows.max(1)
    };
    ListPopupFit { placement, max_rows }
}

#[cfg(test)]
mod fit_tests {
    use gpui::{Bounds, point, px, size};

    use super::*;

    fn look() -> SelectorItemsPanelLook {
        let mut look = default_selector_items_panel_look(&crate::theme::ThemeTokens::default());
        look.padding = 4.0;
        look.item_height = 20.0;
        look.popup_offset_y = 4.0;
        look.window_margin = 8.0;
        look
    }

    #[test]
    fn list_fits_below_without_limiting_rows() {
        let trigger = Bounds::new(point(px(10.0), px(10.0)), size(px(100.0), px(30.0)));
        let fit = fit_list_popup(trigger, px(100.0), &look(), px(0.0), 5, size(px(400.0), px(400.0)));
        assert_eq!(fit.placement.side, crate::infra::popup_placement::PopupSide::Below);
        assert!(fit.max_rows >= 5);
    }

    #[test]
    fn list_near_the_bottom_flips_above() {
        let trigger = Bounds::new(point(px(10.0), px(340.0)), size(px(100.0), px(30.0)));
        let fit = fit_list_popup(trigger, px(100.0), &look(), px(0.0), 5, size(px(400.0), px(400.0)));
        assert_eq!(fit.placement.side, crate::infra::popup_placement::PopupSide::Above);
    }

    #[test]
    fn long_list_is_limited_to_the_rows_that_fit() {
        // Below: 400 - 8 - (190 + 4) = 198; above: 160 - 4 - 8 = 148. Rows: (198 - 8 - 30) / 20 = 8.
        let trigger = Bounds::new(point(px(10.0), px(160.0)), size(px(100.0), px(30.0)));
        let fit = fit_list_popup(trigger, px(100.0), &look(), px(30.0), 50, size(px(400.0), px(400.0)));
        assert_eq!(fit.placement.side, crate::infra::popup_placement::PopupSide::Below);
        assert_eq!(fit.max_rows, 8);
        assert!(fit.placement.scrolling);
    }

    #[test]
    fn zero_row_height_keeps_the_wanted_rows() {
        let mut look = look();
        look.item_height = 0.0;
        let trigger = Bounds::new(point(px(10.0), px(10.0)), size(px(100.0), px(30.0)));
        let fit = fit_list_popup(trigger, px(100.0), &look, px(0.0), 3, size(px(400.0), px(400.0)));
        assert_eq!(fit.max_rows, 3);
    }
}
