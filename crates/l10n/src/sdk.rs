//! Optional bridge to `xgpui-luma::infra::locale`. The SDK stays independent of
//! Fluent/ICU; enable this crate's `sdk` feature in the application.
//!
//! Call [`crate::install`] instead of setting the GPUI global directly. SDK
//! resources (en/de/ja/ar) are inserted below app resources, even if app overrides
//! were loaded before installation. [`crate::set_locale`] republishes direction,
//! language, strings, and numbers before refreshing windows. After editing
//! resources in an installed localizer, call [`sync`] to republish its SDK view.
//!
//! Direct engine mutation (`Localizer::set_locale`) does not publish globals.
//! The adapter owns the SDK context when syncing; customize built-in text with
//! `sdk-*` Fluent message overrides. Direct control labels still take precedence
//! when control families adopt the SDK contract.
//!
//! Numeric display uses up to six fractional digits, percent display up to two,
//! with ICU half-even rounding. On unsupported/nonfinite input or unavailable
//! ICU data, the infallible SDK hook falls back to `PlainNumberFormat`. Unsigned
//! integers never pass through a float. Range endpoints are localized with the
//! SDK's default en-dash separator. Formatters/results are not cached.
//!
//! ```no_run
//! use xgpui_luma_l10n::{Error, Localizer, install, set_locale};
//! fn configure(cx: &mut gpui::App) -> Result<(), Error> {
//!     let mut localizer = Localizer::new("de".parse()?);
//!     localizer.add_resource(&"de".parse()?, "sdk-next = Weiter zur Vorschau")?;
//!     install(localizer, cx)?;
//!     assert_eq!(xgpui_luma::infra::locale::LocaleContext::current(cx).strings.next,
//!                "Weiter zur Vorschau");
//!     set_locale("ar-u-nu-arab".parse()?, cx);
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use gpui::{App, SharedString};
use xgpui_luma::infra::locale::{
    LocaleContext, SdkStrings, NumberFormat, PlainNumberFormat, LayoutDirection as SdkDirection,
};
use crate::{Error, FluentArgs, LayoutDirection, Locale, Localizer};

pub(crate) const RESOURCES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en/sdk.ftl")),
    ("de", include_str!("../locales/de/sdk.ftl")),
    ("ja", include_str!("../locales/ja/sdk.ftl")),
    ("ar", include_str!("../locales/ar/sdk.ftl")),
];

pub(crate) fn prepare(localizer: &mut Localizer) -> Result<(), Error> {
    if !localizer.sdk_resources_loaded {
        localizer.prepend_defaults(RESOURCES)?;
        localizer.sdk_resources_loaded = true;
    }
    Ok(())
}

struct IcuNumbers(Locale);
impl NumberFormat for IcuNumbers {
    fn integer(&self, value: u64) -> SharedString {
        self.0
            .fmt_number(&value.into())
            .map(Into::into)
            .unwrap_or_else(|_| PlainNumberFormat.integer(value))
    }
    fn number(&self, value: f64) -> SharedString {
        self.0.fmt_float(value, 6).map(Into::into).unwrap_or_else(|_| PlainNumberFormat.number(value))
    }
    fn percent(&self, ratio: f64) -> SharedString {
        self.0.fmt_percent(ratio, 2).map(Into::into).unwrap_or_else(|_| PlainNumberFormat.percent(ratio))
    }
}

fn page_of(page: usize, pages: usize, cx: &App) -> SharedString {
    let Some(localizer) = cx.try_global::<Localizer>() else {
        return format!("Page {page} of {pages}").into();
    };
    let numbers = &LocaleContext::current(cx).numbers;
    let mut args = FluentArgs::new();
    // SDK page variables are formatted display strings, preserving full u64
    // precision and Fluent isolation; they are not numeric plural selectors.
    args.set("page", numbers.integer(page as u64).to_string());
    args.set("pages", numbers.integer(pages as u64).to_string());
    localizer.tr_args("sdk-page-of", &args).into()
}

fn selection_summary(selected: usize, total: usize, cx: &App) -> SharedString {
    let Some(localizer) = cx.try_global::<Localizer>() else {
        return (SdkStrings::default().selection_summary)(selected, total, cx);
    };
    let numbers = &LocaleContext::current(cx).numbers;
    let mut args = FluentArgs::new();
    // Display strings preserve exact integers and remain isolated in mixed text.
    args.set("selected", numbers.integer(selected as u64).to_string());
    args.set("total", numbers.integer(total as u64).to_string());
    localizer.tr_args("sdk-selection-summary", &args).into()
}

pub(crate) fn context(localizer: &Localizer) -> LocaleContext {
    let tr = |id| localizer.tr(id).into();
    let strings = SdkStrings {
        cancel: tr("sdk-cancel"),
        clear: tr("sdk-clear"),
        first: tr("sdk-first"),
        previous: tr("sdk-previous"),
        next: tr("sdk-next"),
        last: tr("sdk-last"),
        no_matches: tr("sdk-no-matches"),
        select_placeholder: tr("sdk-select-placeholder"),
        search_placeholder: tr("sdk-search-placeholder"),
        selection_search: tr("sdk-selection-search"),
        page_of: Arc::new(page_of),
        rows_per_page: tr("sdk-rows-per-page"),
        selection_summary: Arc::new(selection_summary),
    };
    LocaleContext {
        direction: match localizer.locale().direction() {
            LayoutDirection::Ltr => SdkDirection::Ltr,
            LayoutDirection::Rtl => SdkDirection::Rtl,
        },
        language: Some(localizer.locale().language_tag().into()),
        strings: Arc::new(strings),
        numbers: Arc::new(IcuNumbers(localizer.locale().clone())),
    }
}

/// Republish the SDK context after app resource changes or direct engine
/// mutation. Returns false if no localizer is installed, leaving SDK state alone.
pub fn sync(cx: &mut App) -> Result<bool, Error> {
    if cx.try_global::<Localizer>().is_none() {
        return Ok(false);
    }
    prepare(cx.global_mut::<Localizer>())?;
    let context = context(cx.global::<Localizer>());
    cx.set_global(context);
    cx.refresh_windows();
    Ok(true)
}

#[cfg(test)]
mod tests;
