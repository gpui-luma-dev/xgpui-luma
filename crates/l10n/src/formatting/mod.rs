//! ICU4X formatting with compiled locale data. Enable `formatting` for all
//! helpers, or select `format-numbers` (also bytes), `format-percent`,
//! `format-duration`, `format-datetime`, and `format-lists` individually.
//!
//! Helpers live on `Locale`: use `localizer.locale().fmt_number(...)` so runtime
//! locale changes also change formatting. Full Unicode locale preferences are
//! forwarded to ICU, including `-u-nu-arab`/`-u-nu-latn`, `-u-ca`, and `-u-hc`.
//! Formatters are constructed per call; results are backend-independent strings.
//! Pass them as Fluent arguments for bidi isolation. Sentence plurals remain in
//! Fluent; ICU chooses grammatical unit forms internally for durations.
//! Keep raw numeric plural-selector arguments separate from preformatted display
//! strings; these helpers do not replace Fluent's own number formatting.
//!
//! Percent and duration APIs wrap a pinned experimental ICU component without
//! exposing its types. All helpers return errors rather than panicking.
//!
//! ```
//! # #[cfg(feature = "format-numbers")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use xgpui_luma_l10n::{Locale, formatting::{ByteUnits, Decimal}};
//! let locale: Locale = "de".parse()?;
//! let number: Decimal = "1234.50".parse()?;
//! assert_eq!(locale.fmt_number(&number)?, "1.234,50");
//! assert_eq!(locale.fmt_bytes(1536, ByteUnits::Binary, 2)?, "1,5\u{a0}KiB");
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "format-numbers"))]
//! # fn main() {}
//! ```

#[cfg(feature = "format-numbers")]
mod numbers;
#[cfg(feature = "format-datetime")]
mod datetime;
#[cfg(feature = "format-lists")]
mod lists;
#[cfg(feature = "format-duration")]
mod duration;

#[cfg(feature = "format-numbers")]
pub use icu_decimal::input::Decimal;
#[cfg(feature = "format-numbers")]
pub use numbers::ByteUnits;
#[cfg(feature = "format-datetime")]
pub use datetime::DateStyle;
#[cfg(feature = "format-lists")]
pub use lists::ListKind;
#[cfg(feature = "format-duration")]
pub use duration::DurationStyle;

/// Invalid input or unavailable ICU formatter data.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("{operation}: {details}")]
    Icu { operation: &'static str, details: String },
    #[error("number must be finite and representable as a decimal")]
    InvalidNumber,
    #[error("fractional digit limit must be at most 18 (got {0})")]
    InvalidPrecision(u8),
    #[error("invalid ISO date: {0}")]
    InvalidDate(String),
    #[error("invalid wall-clock time: {0}")]
    InvalidTime(String),
}

fn icu_error(operation: &'static str, error: impl std::fmt::Display) -> FormatError {
    FormatError::Icu { operation, details: error.to_string() }
}
