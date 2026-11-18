use std::time::Duration;
use icu_experimental::duration::{
    DurationFormatter, ValidatedDurationFormatterOptions,
    options::{BaseStyle, DurationFormatterOptions, FieldDisplay},
};
use super::{FormatError, icu_error};
use crate::Locale;

/// CLDR duration unit styles; digital uses locale-specific clock punctuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationStyle {
    Long,
    Short,
    Narrow,
    Digital,
}

impl Locale {
    /// Format an elapsed duration with localized units and digits. Decomposes
    /// into days/hours/minutes/seconds/subseconds without inventing months or
    /// years. This is not a wall-clock time; days always mean 24 elapsed hours.
    pub fn fmt_duration(&self, value: Duration, style: DurationStyle) -> Result<String, FormatError> {
        let mut options = DurationFormatterOptions::default();
        options.base = match style {
            DurationStyle::Long => BaseStyle::Long,
            DurationStyle::Short => BaseStyle::Short,
            DurationStyle::Narrow => BaseStyle::Narrow,
            DurationStyle::Digital => BaseStyle::Digital,
        };
        if matches!(style, DurationStyle::Digital) {
            // ICU's automatic visibility can omit subsecond-only digital
            // durations. Explicit clock fields retain the fractional seconds.
            options.hour_visibility = Some(FieldDisplay::Always);
            options.minute_visibility = Some(FieldDisplay::Always);
            options.second_visibility = Some(FieldDisplay::Always);
        } else if value.is_zero() {
            options.second_visibility = Some(FieldDisplay::Always);
        }
        let options = ValidatedDurationFormatterOptions::validate(options)
            .map_err(|error| icu_error("duration options", error))?;
        let formatter = DurationFormatter::try_new((&self.0).into(), options)
            .map_err(|error| icu_error("duration formatter", error))?;
        let seconds = value.as_secs();
        let duration = icu_experimental::duration::Duration {
            days: seconds / 86400,
            hours: seconds / 3600 % 24,
            minutes: seconds / 60 % 60,
            seconds: seconds % 60,
            milliseconds: u64::from(value.subsec_millis()),
            microseconds: u64::from(value.subsec_micros() % 1000),
            nanoseconds: u64::from(value.subsec_nanos() % 1000),
            ..Default::default()
        };
        Ok(formatter.format(&duration).to_string())
    }
}
