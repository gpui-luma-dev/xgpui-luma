//! Locale contracts used by SDK templates across looks.
//!
//! This module has no localization-engine dependency. Apps or an l10n adapter
//! publish direction, language, translated strings, and number formatters as one
//! snapshot with [`set_locale_context`]. Without a snapshot, existing apps get
//! LTR, no language preference, English strings, and plain number formatting.
//! Control families adopt these contracts separately; setting a context alone
//! does not change controls whose templates have not been migrated yet.
//!
//! ```
//! use gpui::{App, IntoElement, div, px, prelude::*};
//! use xgpui_luma::prelude::{LocaleContext, LogicalStyled};
//! fn localized_row(cx: &App) -> impl IntoElement {
//!     let direction = LocaleContext::current(cx).direction;
//!     div().flex().flex_row_logical(direction).ps(px(8.), direction)
//! }
//! ```

use std::sync::{Arc, OnceLock};
use gpui::{App, Global, ParagraphShapeOptions, SharedString, TextDirection};

mod logical;
pub use logical::LogicalStyled;

/// Horizontal UI direction. Geometry that represents data keeps physical edges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutDirection {
    #[default]
    Ltr,
    Rtl,
}

/// Locale-aware numeric display supplied by an app or localization adapter.
/// Implementations must be thread safe and return a displayable string even for
/// unsupported input. Precision and error fallback policy belong to the adapter.
pub trait NumberFormat: Send + Sync {
    /// Format an unsigned integer without losing precision through `f64`.
    fn integer(&self, value: u64) -> SharedString;
    /// Format a numeric value, including negative or fractional values.
    fn number(&self, value: f64) -> SharedString;
    /// Format a ratio: 0.125 means 12.5 percent. Override for locale-specific
    /// percent symbols, spacing, and placement.
    fn percent(&self, ratio: f64) -> SharedString {
        format!("{}%", self.number(ratio * 100.)).into()
    }
    /// Format a display range. Override for locale-specific patterns.
    fn range(&self, start: f64, end: f64) -> SharedString {
        format!("{}–{}", self.number(start), self.number(end)).into()
    }
}

/// Compatibility formatter: Rust's plain numeric display and English patterns.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlainNumberFormat;
impl NumberFormat for PlainNumberFormat {
    fn integer(&self, value: u64) -> SharedString {
        value.to_string().into()
    }
    fn number(&self, value: f64) -> SharedString {
        value.to_string().into()
    }
}

/// Built-in page indicator, invoked with one-based page and page count.
/// `App` lets adapters consult the active localizer at render time without
/// capturing stale translations. It must not mutate state or perform I/O.
pub type PageOfFormatter = Arc<dyn Fn(usize, usize, &App) -> SharedString + Send + Sync>;

/// Built-in table selection summary, invoked with selected and total row counts.
/// As with [`PageOfFormatter`], adapters may read the active localizer at render time.
pub type SelectionSummaryFormatter = Arc<dyn Fn(usize, usize, &App) -> SharedString + Send + Sync>;

/// Autocomplete diagnostic/status copy. State/event identifiers are data, not UI labels.
/// Adapters localize the framing and status names; the selected label is app content.
pub type AutocompleteStatusFormatter =
    Arc<dyn Fn(&str, Option<&str>, usize, &str, &str, &App) -> (SharedString, SharedString) + Send + Sync>;

/// SDK-owned text. App-provided labels/placeholders take precedence when
/// controls adopt this contract. Dynamic text stays in a formatter so plural
/// selection and word order can be provided by the localization engine.
#[derive(Clone)]
pub struct SdkStrings {
    /// Locale-sensitive autocomplete status and diagnostic framing.
    pub autocomplete_status: AutocompleteStatusFormatter,
    pub cancel: SharedString,
    pub clear: SharedString,
    pub first: SharedString,
    pub previous: SharedString,
    pub next: SharedString,
    pub last: SharedString,
    pub no_matches: SharedString,
    pub select_placeholder: SharedString,
    pub search_placeholder: SharedString,
    pub selection_search: SharedString,
    pub page_of: PageOfFormatter,
    pub rows_per_page: SharedString,
    pub selection_summary: SelectionSummaryFormatter,
}

impl Default for SdkStrings {
    fn default() -> Self {
        Self {
            autocomplete_status: Arc::new(|status, selected, matches, last, keyboard, _| {
                (
                    format!("status: {status}").into(),
                    format!(
                        "selected: {} | matches: {matches} | last: {last} | keyboard: {keyboard}",
                        selected.unwrap_or("none")
                    )
                    .into(),
                )
            }),
            cancel: "Cancel".into(),
            clear: "Clear".into(),
            first: "First".into(),
            previous: "Previous".into(),
            next: "Next".into(),
            last: "Last".into(),
            no_matches: "No matches".into(),
            select_placeholder: "Select…".into(),
            search_placeholder: "Search…".into(),
            selection_search: "Selection search".into(),
            page_of: Arc::new(|page, pages, _| format!("Page {page} of {pages}").into()),
            rows_per_page: "Rows per page".into(),
            selection_summary: Arc::new(|selected, total, _| format!("Selected rows: {selected} of {total}").into()),
        }
    }
}

/// One app-wide locale snapshot consumed by SDK templates across looks.
/// The SDK does not parse or infer language tags; adapters supply a BCP 47 tag.
/// Use [`set_locale_context`] to publish changes and invalidate windows together.
#[derive(Clone)]
pub struct LocaleContext {
    pub direction: LayoutDirection,
    pub language: Option<SharedString>,
    pub strings: Arc<SdkStrings>,
    pub numbers: Arc<dyn NumberFormat>,
}

impl Default for LocaleContext {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::Ltr,
            language: None,
            strings: Arc::new(SdkStrings::default()),
            numbers: Arc::new(PlainNumberFormat),
        }
    }
}
impl Global for LocaleContext {}

impl LocaleContext {
    /// Read the installed snapshot or an immutable shared English/LTR default.
    /// Does not install a global as a side effect and never panics if absent.
    pub fn current(cx: &App) -> &Self {
        static DEFAULT: OnceLock<LocaleContext> = OnceLock::new();
        cx.try_global::<Self>().unwrap_or_else(|| DEFAULT.get_or_init(Self::default))
    }

    /// Shaping options for UI chrome: locale base direction and content language.
    pub fn paragraph_options(&self) -> ParagraphShapeOptions {
        self.options(match self.direction {
            LayoutDirection::Ltr => TextDirection::LeftToRight,
            LayoutDirection::Rtl => TextDirection::RightToLeft,
        })
    }

    /// User/data text infers its own base direction while retaining language.
    pub fn data_paragraph_options(&self) -> ParagraphShapeOptions {
        self.options(TextDirection::Auto)
    }

    fn options(&self, direction: TextDirection) -> ParagraphShapeOptions {
        let mut options = ParagraphShapeOptions::default().with_direction(direction);
        options.language = self.language.clone();
        options
    }
}

/// Publish all locale properties before refreshing windows. Subsequent reads
/// observe the same direction/language/strings/numbers snapshot in both looks.
pub fn set_locale_context(context: LocaleContext, cx: &mut App) {
    cx.set_global(context);
    cx.refresh_windows();
}

#[cfg(test)]
mod tests;

#[cfg(feature = "test-support")]
pub mod conformance;
