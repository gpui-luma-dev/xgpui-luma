use fixed_decimal::FloatPrecision;
use icu_decimal::DecimalFormatter;
use super::{Decimal, FormatError, icu_error};
use crate::Locale;

/// Standard byte-size symbols. The numeric value is localized; SI/IEC unit
/// symbols stay invariant and are separated with a nonbreaking space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteUnits {
    /// Powers of 1000: B, kB, MB, GB, TB, PB, EB.
    Decimal,
    /// Powers of 1024: B, KiB, MiB, GiB, TiB, PiB, EiB.
    Binary,
}

fn precision(digits: u8) -> Result<(), FormatError> {
    if digits > 18 {
        Err(FormatError::InvalidPrecision(digits))
    } else {
        Ok(())
    }
}

fn float(value: f64) -> Result<Decimal, FormatError> {
    Decimal::try_from_f64(value, FloatPrecision::RoundTrip).map_err(|_| FormatError::InvalidNumber)
}

fn round(number: &mut Decimal, digits: u8) {
    number.round(-i16::from(digits));
    number.trim_start();
    number.trim_end();
}

impl Locale {
    /// Localize an exact decimal, preserving its fractional zeros. Parse decimal
    /// strings or convert integers into `Decimal` to avoid floating-point loss.
    pub fn fmt_number(&self, number: &Decimal) -> Result<String, FormatError> {
        let formatter = DecimalFormatter::try_new((&self.0).into(), Default::default())
            .map_err(|error| icu_error("number formatter", error))?;
        Ok(formatter.format_to_string(number))
    }

    /// Localize a finite float, rounding half to even and trimming trailing
    /// fractional zeros. `max_fraction_digits` must be in 0..=18.
    pub fn fmt_float(&self, value: f64, max_fraction_digits: u8) -> Result<String, FormatError> {
        precision(max_fraction_digits)?;
        let mut number = float(value)?;
        round(&mut number, max_fraction_digits);
        self.fmt_number(&number)
    }

    /// Format a ratio as a percentage: `0.125` → `12.5%` in English. Scaling
    /// precedes rounding; CLDR supplies the sign, spacing, and sign placement.
    /// Accepts finite ratios and a fractional digit limit in 0..=18.
    #[cfg(feature = "format-percent")]
    pub fn fmt_percent(&self, ratio: f64, max_fraction_digits: u8) -> Result<String, FormatError> {
        use icu_experimental::dimension::percent::formatter::PercentFormatter;
        precision(max_fraction_digits)?;
        let mut number = float(ratio)?;
        number.multiply_pow10(2);
        round(&mut number, max_fraction_digits);
        let formatter = PercentFormatter::try_new((&self.0).into(), Default::default())
            .map_err(|error| icu_error("percent formatter", error))?;
        Ok(formatter.format(&number).to_string())
    }

    /// Scale bytes using exact integer arithmetic, round half to even, and
    /// localize the number. Values that round up to the next unit are promoted.
    /// The fractional digit limit is 0..=18; scaling and rounding never pass
    /// through a float, including for values above its exact integer range.
    pub fn fmt_bytes(&self, bytes: u64, units: ByteUnits, max_fraction_digits: u8) -> Result<String, FormatError> {
        precision(max_fraction_digits)?;
        let (base, symbols) = match units {
            ByteUnits::Decimal => (1000_u128, ["B", "kB", "MB", "GB", "TB", "PB", "EB"]),
            ByteUnits::Binary => (1024_u128, ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"]),
        };
        let value = u128::from(bytes);
        // u64::MAX * 10^18 fits in u128, so scaling cannot overflow.
        let scale = 10_u128.pow(u32::from(max_fraction_digits));
        let mut unit = 0;
        let mut divisor = 1;
        while unit < symbols.len() - 1 && value >= divisor * base {
            unit += 1;
            divisor *= base;
        }
        loop {
            let numerator = value * scale;
            let mut rounded = numerator / divisor;
            let remainder = numerator % divisor;
            if remainder * 2 > divisor || (remainder * 2 == divisor && rounded % 2 != 0) {
                rounded += 1;
            }
            if unit < symbols.len() - 1 && rounded >= base * scale {
                unit += 1;
                divisor *= base;
                continue;
            }
            let mut number = Decimal::from(rounded);
            number.multiply_pow10(-i16::from(max_fraction_digits));
            number.trim_end();
            return Ok(format!("{}\u{a0}{}", self.fmt_number(&number)?, symbols[unit]));
        }
    }
}
