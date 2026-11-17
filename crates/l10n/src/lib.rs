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
//! With the `gpui` feature, install with `cx.set_global(localizer)`, translate
//! through `L10nExt`, and switch with `set_locale`. SDK context integration,
//! ICU number/date formatting, typed IDs, and resource validation tooling are
//! subsequent slices of issue #225; this crate currently supplies the engine core.

mod locale;
mod localizer;
#[cfg(feature = "gpui")]
mod integration;

pub use fluent_bundle::{FluentArgs, FluentValue};
pub use locale::{LayoutDirection, Locale};
pub use localizer::{Error, Localizer};
#[cfg(feature = "gpui")]
pub use integration::{L10nExt, set_locale};
