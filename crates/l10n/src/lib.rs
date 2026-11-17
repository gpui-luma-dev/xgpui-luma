//! Fluent localization for Luma applications.
//!
//! Load embedded resources in SDK-first, app-last order; later resources override
//! earlier messages. Disable the default `gpui` feature to use the engine headlessly.
//!
//! ```
//! use xgpui_luma_l10n::{FluentArgs, Locale, Localizer};
//!
//! # fn main() -> Result<(), xgpui_luma_l10n::Error> {
//! let english: Locale = "en".parse()?;
//! let mut localizer = Localizer::new("de-DE".parse()?);
//! localizer.add_resource(&english, "welcome = Welcome, { $name }!")?;
//! let mut args = FluentArgs::new();
//! args.set("name", "Ada");
//! assert_eq!(localizer.tr_args("welcome", &args), "Welcome, \u{2068}Ada\u{2069}!");
//! # Ok(())
//! # }
//! ```
//!
//! With the `gpui` feature, use `install` to publish the localizer, translate
//! through `L10nExt`, and switch with `set_locale`. The optional `sdk` feature
//! also publishes the SDK locale context, built-in strings, and ICU numbers.
//! Optional formatting features add ICU helpers
//! on `Locale`; see `formatting`. The `build-support` feature provides resource checks and typed
//! message constructors generated from the English catalog; see `build_support`.
//!
//! Resolve startup preferences with `LocaleRequest` before creating the localizer.
//! Explicit CLI and saved-setting values are validated strictly. Unusable OS
//! locale entries are skipped, with English as the final default. Resource
//! availability is independent: an untranslated locale retains its direction
//! and formatting identity while message lookup falls back to English.

mod locale;
mod localizer;
mod selection;
mod message;
#[cfg(any(
    feature = "format-numbers",
    feature = "format-duration",
    feature = "format-datetime",
    feature = "format-lists"
))]
pub mod formatting;
#[cfg(feature = "build-support")]
pub mod build_support;
#[cfg(feature = "gpui")]
mod integration;
#[cfg(feature = "sdk")]
pub mod sdk;

pub use fluent_bundle::{FluentArgs, FluentValue};
pub use locale::{LayoutDirection, Locale};
pub use localizer::{Error, Localizer};
pub use selection::{LocaleRequest, LocaleSource, SelectedLocale};
pub use message::Message;
#[cfg(feature = "gpui")]
pub use integration::{L10nExt, install, set_locale};
