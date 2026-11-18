//! English for the bare SDK controls in the workshop. The SDK ships no text of
//! its own; these match what the looks supply, and are where the workshop's
//! language switching will plug in.

use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::split_button::SplitButtonText;

pub(super) const SELECT_PLACEHOLDER: &str = "Select…";
pub(super) const SELECTION_SEARCH: &str = "Selection search";
pub(super) const FILTER_PLACEHOLDER: &str = "Type to filter…";
pub(super) const NO_MATCHES: &str = "No matches";
pub(super) const ROWS_PER_PAGE: &str = "Rows per page";

pub(super) fn page_of(page: usize, pages: usize) -> SharedString {
    format!("Page {page} of {pages}").into()
}

pub(super) fn rows_selected(selected: usize, total: usize) -> SharedString {
    format!("{selected} of {total} row(s) selected.").into()
}

pub(super) fn split_button() -> SplitButtonText {
    SplitButtonText {
        usage_hint: "Enter or Space activates the primary action; Down or Up Arrow opens the menu; Escape dismisses"
            .into(),
        primary_unavailable: "Primary action unavailable; press Down Arrow for menu options".into(),
        menu_label: Some(Arc::new(|label| format!("{label} menu").into())),
    }
}
