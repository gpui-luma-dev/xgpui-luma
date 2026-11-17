use icu_datetime::{
    DateTimeFormatter, NoCalendarFormatter,
    fieldsets::{YMD, T},
    input::{Date, Time},
};
use super::{FormatError, icu_error};
use crate::Locale;

/// CLDR date field length (year, month, day).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateStyle {
    Short,
    Medium,
    Long,
}

impl Locale {
    /// Format an ISO date in the locale's calendar (`-u-ca` is honored). No
    /// timezone conversion is performed; the caller supplies the intended date.
    pub fn fmt_date(&self, year: i32, month: u8, day: u8, style: DateStyle) -> Result<String, FormatError> {
        let date = Date::try_new_iso(year, month, day).map_err(|error| FormatError::InvalidDate(error.to_string()))?;
        let fields = match style {
            DateStyle::Short => YMD::short(),
            DateStyle::Medium => YMD::medium(),
            DateStyle::Long => YMD::long(),
        };
        let formatter =
            DateTimeFormatter::try_new((&self.0).into(), fields).map_err(|error| icu_error("date formatter", error))?;
        Ok(formatter.format(&date).to_string())
    }

    /// Format wall-clock hours/minutes/seconds using the locale's hour cycle
    /// (`-u-hc` is honored). This does not interpret timestamps or timezones.
    pub fn fmt_time(&self, hour: u8, minute: u8, second: u8) -> Result<String, FormatError> {
        let time =
            Time::try_new(hour, minute, second, 0).map_err(|error| FormatError::InvalidTime(error.to_string()))?;
        let formatter = NoCalendarFormatter::try_new((&self.0).into(), T::medium())
            .map_err(|error| icu_error("time formatter", error))?;
        Ok(formatter.format(&time).to_string())
    }
}
