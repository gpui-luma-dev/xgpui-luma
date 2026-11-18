//! The look's English for text the SDK leaves empty. Apps replace any of it
//! through the same builder inputs (placeholder, no_matches, text, ...).

use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::split_button::SplitButtonText;

pub(crate) const SELECT_PLACEHOLDER: &str = "Select…";
pub(crate) const SELECTION_SEARCH: &str = "Selection search";
pub(crate) const FILTER_PLACEHOLDER: &str = "Type to filter…";
pub(crate) const NO_MATCHES: &str = "No matches";
pub(crate) const DRAG_TO_REORDER: &str = "Drag to reorder";

pub(crate) fn page_of(page: usize, pages: usize) -> SharedString {
    format!("Page {page} of {pages}").into()
}

pub(crate) fn split_button() -> SplitButtonText {
    SplitButtonText {
        usage_hint: "Enter or Space activates the primary action; Down or Up Arrow opens the menu; Escape dismisses"
            .into(),
        primary_unavailable: "Primary action unavailable; press Down Arrow for menu options".into(),
        menu_label: Some(Arc::new(|label| format!("{label} menu").into())),
    }
}
