//! Text that SDK controls display on their own, when the app has not supplied it.
//!
//! The defaults are the SDK's English text. An app or look that localizes
//! replaces them as one snapshot with [`set_luma_strings`]; controls read the
//! current snapshot when they render, so a locale switch updates open windows.
//! Text an app passes to a control (a placeholder, a label, a formatter) always
//! wins over these defaults.
//!
//! ```
//! use gpui::App;
//! use xgpui_luma::infra::strings::{LumaStrings, set_luma_strings};
//!
//! fn use_french(cx: &mut App) {
//!     set_luma_strings(
//!         LumaStrings {
//!             no_matches: "Aucun résultat".into(),
//!             ..LumaStrings::default()
//!         },
//!         cx,
//!     );
//! }
//! ```

use std::sync::{Arc, OnceLock};

use gpui::{App, Global, SharedString};

/// Text built from two counts, such as "Page 2 of 5". A closure rather than a
/// pattern string so a translation can choose its own word order and plurals.
pub type CountText = Arc<dyn Fn(usize, usize) -> SharedString + Send + Sync + 'static>;

/// The SDK's built-in user-visible text. See the [module docs](self).
#[derive(Clone)]
pub struct LumaStrings {
    /// Combo box and search selector trigger before anything is chosen.
    pub select_placeholder: SharedString,
    /// Search field inside the search selector popup.
    pub selection_search: SharedString,
    /// Autocomplete text field before the user types.
    pub filter_placeholder: SharedString,
    /// Search selector popup when the filter matches no items.
    pub no_matches: SharedString,
    /// Label beside the page-size picker in a paging table.
    pub rows_per_page: SharedString,
    /// Accessible description of a disabled list item.
    pub disabled: SharedString,
    /// Pager indicator, called with the one-based page and the page count.
    pub page_of: CountText,
    /// Paging table summary, called with the selected and total row counts.
    pub rows_selected: CountText,
}

/// The shared English page indicator. Controls compare against this instance
/// to tell the built-in text apart from a formatter the app supplied.
pub(crate) fn english_page_of() -> CountText {
    static PAGE_OF: OnceLock<CountText> = OnceLock::new();
    PAGE_OF.get_or_init(|| Arc::new(|page, pages| format!("Page {page} of {pages}").into())).clone()
}

impl Default for LumaStrings {
    fn default() -> Self {
        Self {
            select_placeholder: "Select…".into(),
            selection_search: "Selection search".into(),
            filter_placeholder: "Type to filter…".into(),
            no_matches: "No matches".into(),
            rows_per_page: "Rows per page".into(),
            disabled: "Disabled".into(),
            page_of: english_page_of(),
            rows_selected: Arc::new(|selected, total| format!("{selected} of {total} row(s) selected.").into()),
        }
    }
}

impl Global for LumaStrings {}

impl LumaStrings {
    /// The installed strings, or the English defaults when none are installed.
    /// Never installs a global and never panics.
    pub fn current(cx: &App) -> &Self {
        static DEFAULT: OnceLock<LumaStrings> = OnceLock::new();
        cx.try_global::<Self>().unwrap_or_else(|| DEFAULT.get_or_init(Self::default))
    }
}

/// Replace the SDK's built-in text and redraw every window.
pub fn set_luma_strings(strings: LumaStrings, cx: &mut App) {
    cx.set_global(strings);
    cx.refresh_windows();
}

#[cfg(all(test, feature = "test-support"))]
pub(crate) mod tests;
