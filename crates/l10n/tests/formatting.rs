#![cfg(any(
    feature = "format-numbers",
    feature = "format-duration",
    feature = "format-datetime",
    feature = "format-lists"
))]

use xgpui_luma_l10n::Locale;
fn locale(tag: &str) -> Locale {
    tag.parse().unwrap()
}

#[cfg(feature = "format-numbers")]
mod numbers {
    use super::*;
    use xgpui_luma_l10n::formatting::{ByteUnits, Decimal, FormatError};

    #[test]
    fn preserves_exact_decimals_and_localizes_separators_and_digits() {
        let number: Decimal = "1234567.50".parse().unwrap();
        assert_eq!(locale("en").fmt_number(&number).unwrap(), "1,234,567.50");
        assert_eq!(locale("de").fmt_number(&number).unwrap(), "1.234.567,50");
        assert_eq!(locale("ja").fmt_number(&number).unwrap(), "1,234,567.50");
        assert_eq!(locale("ar-u-nu-arab").fmt_float(1234.5, 2).unwrap(), "١٬٢٣٤٫٥");
        assert_eq!(locale("ar-u-nu-latn").fmt_float(1234.5, 2).unwrap(), "1,234.5");
        assert_eq!(locale("en").fmt_number(&Decimal::from(u64::MAX)).unwrap(), "18,446,744,073,709,551,615");
    }

    #[test]
    fn float_rounding_is_half_even_with_validated_inputs() {
        let en = locale("en");
        for (value, digits, expected) in
            [(2.5, 0, "2"), (3.5, 0, "4"), (-2.5, 0, "-2"), (2.675, 2, "2.68"), (0.125, 2, "0.12")]
        {
            assert_eq!(en.fmt_float(value, digits).unwrap(), expected);
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(en.fmt_float(value, 2), Err(FormatError::InvalidNumber)));
        }
        assert!(matches!(en.fmt_float(1.0, 19), Err(FormatError::InvalidPrecision(19))));
        assert!(!en.fmt_float(f64::MAX, 18).unwrap().is_empty());
        assert_eq!(en.fmt_float(f64::MIN_POSITIVE, 18).unwrap(), "0");
    }

    #[test]
    fn negative_zero_and_negatives_rounding_to_zero_format_without_a_sign() {
        for tag in ["en", "de", "ar-u-nu-arab"] {
            let locale = locale(tag);
            let zero = locale.fmt_float(0.0, 2).unwrap();
            for (value, digits) in [(-0.0, 2), (-0.004, 2), (-0.4, 0), (-1e-300, 18)] {
                assert_eq!(locale.fmt_float(value, digits).unwrap(), zero, "{tag} {value} {digits}");
            }
        }
        assert_eq!(locale("en").fmt_float(-0.006, 2).unwrap(), "-0.01");
    }

    #[test]
    fn byte_sizes_are_exact_at_boundaries_and_promote_after_rounding() {
        let en = locale("en");
        for (bytes, units, digits, expected) in [
            (0, ByteUnits::Binary, 2, "0\u{a0}B"),
            (999, ByteUnits::Decimal, 2, "999\u{a0}B"),
            (1000, ByteUnits::Decimal, 2, "1\u{a0}kB"),
            (1024, ByteUnits::Binary, 2, "1\u{a0}KiB"),
            (1536, ByteUnits::Binary, 2, "1.5\u{a0}KiB"),
            (1250, ByteUnits::Decimal, 1, "1.2\u{a0}kB"),
            (1750, ByteUnits::Decimal, 1, "1.8\u{a0}kB"),
            (999999, ByteUnits::Decimal, 2, "1\u{a0}MB"),
            (1048575, ByteUnits::Binary, 2, "1\u{a0}MiB"),
            (u64::MAX, ByteUnits::Decimal, 18, "18.446744073709551615\u{a0}EB"),
            (u64::MAX, ByteUnits::Binary, 0, "16\u{a0}EiB"),
        ] {
            assert_eq!(en.fmt_bytes(bytes, units, digits).unwrap(), expected);
        }
        assert_eq!(locale("de").fmt_bytes(1536, ByteUnits::Binary, 2).unwrap(), "1,5\u{a0}KiB");
        assert!(matches!(en.fmt_bytes(1, ByteUnits::Decimal, 19), Err(FormatError::InvalidPrecision(19))));
    }

    #[test]
    fn runtime_locale_switches_change_formatting_and_fluent_isolates_results() {
        let mut localizer = xgpui_luma_l10n::Localizer::new(locale("en"));
        assert_eq!(localizer.locale().fmt_float(1234.5, 1).unwrap(), "1,234.5");
        localizer.set_locale(locale("de"));
        let number = localizer.locale().fmt_float(1234.5, 1).unwrap();
        assert_eq!(number, "1.234,5");
        localizer.add_resource(&locale("de"), "value = Wert: { $number }").unwrap();
        let mut args = xgpui_luma_l10n::FluentArgs::new();
        args.set("number", number);
        assert_eq!(localizer.tr_args("value", &args), "Wert: \u{2068}1.234,5\u{2069}");
    }
}

#[cfg(feature = "format-percent")]
mod percent {
    use super::*;
    use xgpui_luma_l10n::formatting::FormatError;

    #[test]
    fn percentage_patterns_and_scaling_follow_the_locale() {
        assert_eq!(locale("en").fmt_percent(0.125, 1).unwrap(), "12.5%");
        assert_eq!(locale("de").fmt_percent(0.125, 1).unwrap(), "12,5\u{a0}%");
        assert_eq!(locale("tr").fmt_percent(0.125, 1).unwrap(), "%12,5");
        assert_eq!(locale("en").fmt_percent(0.1234, 2).unwrap(), "12.34%");
        assert_eq!(locale("en").fmt_percent(0.99999, 1).unwrap(), "100%");
        assert_eq!(locale("en").fmt_percent(-0.125, 1).unwrap(), "-12.5%");
        assert_eq!(locale("en").fmt_percent(0.0, 2).unwrap(), "0%");
        assert!(locale("ar-u-nu-arab").fmt_percent(0.125, 1).unwrap().contains("١٢٫٥"));
        for (ratio, digits) in [(-0.0, 1), (-0.00004, 2), (-0.004, 0)] {
            assert_eq!(locale("en").fmt_percent(ratio, digits).unwrap(), "0%", "{ratio} {digits}");
            assert_eq!(locale("de").fmt_percent(ratio, digits).unwrap(), "0\u{a0}%", "{ratio} {digits}");
        }
        assert!(matches!(locale("en").fmt_percent(f64::NAN, 1), Err(FormatError::InvalidNumber)));
        assert!(matches!(locale("en").fmt_percent(1.0, 19), Err(FormatError::InvalidPrecision(19))));
    }
}

#[cfg(feature = "format-datetime")]
mod datetime {
    use super::*;
    use xgpui_luma_l10n::formatting::{DateStyle, FormatError};

    #[test]
    fn dates_use_locale_order_and_requested_calendar() {
        assert_eq!(locale("en").fmt_date(2024, 5, 8, DateStyle::Medium).unwrap(), "May 8, 2024");
        assert_eq!(locale("de").fmt_date(2024, 5, 8, DateStyle::Medium).unwrap(), "08.05.2024");
        assert_eq!(locale("ja").fmt_date(2024, 5, 8, DateStyle::Medium).unwrap(), "2024/05/08");
        let buddhist = locale("th-u-ca-buddhist-nu-latn").fmt_date(2024, 5, 8, DateStyle::Long).unwrap();
        assert!(buddhist.contains("2567"), "{buddhist}");
        let arabic = locale("ar-u-nu-arab").fmt_date(2024, 5, 8, DateStyle::Short).unwrap();
        assert!(arabic.contains("٢٠٢٤"), "{arabic}");
        assert!(locale("en").fmt_date(2024, 2, 29, DateStyle::Long).is_ok());
        for (year, month, day) in [(2023, 2, 29), (2024, 13, 1), (2024, 1, 0)] {
            assert!(matches!(
                locale("en").fmt_date(year, month, day, DateStyle::Short),
                Err(FormatError::InvalidDate(_))
            ));
        }
    }

    #[test]
    fn times_honor_hour_cycle_and_validate_clock_fields() {
        assert_eq!(locale("de").fmt_time(15, 4, 5).unwrap(), "15:04:05");
        assert_eq!(locale("en-u-hc-h23").fmt_time(15, 4, 5).unwrap(), "15:04:05");
        assert_eq!(locale("en-u-hc-h12").fmt_time(15, 4, 5).unwrap(), "3:04:05\u{202f}PM");
        assert_eq!(locale("en-u-hc-h23").fmt_time(0, 0, 0).unwrap(), "00:00:00");
        for (hour, minute, second) in [(24, 0, 0), (0, 60, 0), (0, 0, 61)] {
            assert!(matches!(locale("en").fmt_time(hour, minute, second), Err(FormatError::InvalidTime(_))));
        }
    }
}

#[cfg(feature = "format-lists")]
mod lists {
    use super::*;
    use xgpui_luma_l10n::formatting::ListKind;

    #[test]
    fn list_patterns_include_contextual_conjunctions_and_boundary_cases() {
        assert_eq!(locale("en").fmt_list(&["A", "B", "C"], ListKind::And).unwrap(), "A, B, and C");
        assert_eq!(locale("de").fmt_list(&["A", "B", "C"], ListKind::And).unwrap(), "A, B und C");
        assert_eq!(locale("ja").fmt_list(&["A", "B", "C"], ListKind::And).unwrap(), "A、B、C");
        assert_eq!(
            locale("es").fmt_list(&["España", "Suiza", "Italia"], ListKind::And).unwrap(),
            "España, Suiza e Italia"
        );
        assert_eq!(locale("en").fmt_list(&["A", "B"], ListKind::Or).unwrap(), "A or B");
        assert_eq!(locale("en").fmt_list(&["1 h", "2 m"], ListKind::Unit).unwrap(), "1 h, 2 m");
        assert_eq!(locale("en").fmt_list::<&str>(&[], ListKind::And).unwrap(), "");
        assert_eq!(locale("en").fmt_list(&[String::from("שלום")], ListKind::And).unwrap(), "שלום");
    }
}

#[cfg(feature = "format-duration")]
mod duration {
    use super::*;
    use std::time::Duration;
    use xgpui_luma_l10n::formatting::DurationStyle;

    #[test]
    fn elapsed_units_and_zero_have_localized_grammatical_forms() {
        let value = Duration::from_secs(3661);
        assert_eq!(locale("en").fmt_duration(value, DurationStyle::Long).unwrap(), "1 hour, 1 minute, 1 second");
        assert_eq!(locale("de").fmt_duration(value, DurationStyle::Long).unwrap(), "1 Stunde, 1 Minute und 1 Sekunde");
        assert_eq!(locale("ja").fmt_duration(Duration::ZERO, DurationStyle::Long).unwrap(), "0 秒");
        assert_eq!(locale("en").fmt_duration(Duration::from_secs(2), DurationStyle::Long).unwrap(), "2 seconds");
        assert_eq!(locale("en").fmt_duration(Duration::from_secs(86400), DurationStyle::Long).unwrap(), "1 day");
        assert_eq!(locale("ar-u-nu-arab").fmt_duration(Duration::ZERO, DurationStyle::Long).unwrap(), "٠ ثانية");
    }

    #[test]
    fn styles_preserve_subseconds_and_support_the_duration_range() {
        for style in [DurationStyle::Long, DurationStyle::Short, DurationStyle::Narrow, DurationStyle::Digital] {
            assert!(!locale("en").fmt_duration(Duration::ZERO, style).unwrap().is_empty());
            assert!(!locale("en").fmt_duration(Duration::MAX, style).unwrap().is_empty());
        }
        let value = locale("en").fmt_duration(Duration::from_nanos(123456789), DurationStyle::Digital).unwrap();
        assert!(value.contains("123456789"), "{value}");
        assert!(
            locale("en")
                .fmt_duration(Duration::from_nanos(1), DurationStyle::Long)
                .unwrap()
                .contains("1 nanosecond")
        );
    }
}
