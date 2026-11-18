//! Debug-build ICU data probes must respect application logging in every
//! formatting feature combination, not only when `format-duration` is enabled.
#![cfg(any(
    feature = "format-numbers",
    feature = "format-percent",
    feature = "format-datetime",
    feature = "format-lists"
))]

use std::process::Command;
use xgpui_luma_l10n::Locale;

const LOCALES: &[&str] = &[
    "en",
    "de",
    "ja",
    "ar-u-nu-arab",
    "en-XA",
    "ar-XB",
    "th-u-ca-buddhist-nu-latn",
    "ja-u-hc-h12",
    "en-u-hc-h23",
    "sr-Latn",
    "zh-Hant-HK",
    "und",
    "zz",
    "qaa",
    "en-u-ca-japanese",
    "ar-u-ca-islamic",
    "fa",
    "hi-u-nu-deva",
    "ja-u-ca-japanese-hc-h11",
    "en-u-ca-chinese",
    "he-u-ca-hebrew",
    "yue",
];

fn format_everything(locale: &Locale) {
    #[cfg(feature = "format-numbers")]
    {
        let _ = locale.fmt_float(-1234.5, 2);
        let _ = locale.fmt_number(&"12.50".parse().unwrap());
    }
    #[cfg(feature = "format-percent")]
    let _ = locale.fmt_percent(0.125, 1);
    #[cfg(feature = "format-datetime")]
    {
        use xgpui_luma_l10n::formatting::DateStyle;
        for style in [DateStyle::Long, DateStyle::Medium, DateStyle::Short] {
            let _ = locale.fmt_date(2024, 5, 8, style);
        }
        let _ = locale.fmt_time(13, 5, 9);
    }
    #[cfg(feature = "format-lists")]
    {
        use xgpui_luma_l10n::formatting::ListKind;
        for kind in [ListKind::And, ListKind::Or, ListKind::Unit] {
            let _ = locale.fmt_list(&["a", "b", "c"], kind);
        }
    }
}

#[test]
fn formatter_probes_do_not_write_unsolicited_console_errors() {
    const CHILD: &str = "LUMA_PROVIDER_LOGGING_CHILD";
    if std::env::var_os(CHILD).is_some() {
        for tag in LOCALES {
            format_everything(&tag.parse().unwrap());
        }
        return;
    }
    // A fresh process has no logger installed, like the default Lab startup.
    // Capture stderr directly: ICU's debug fallback bypasses the log facade.
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "formatter_probes_do_not_write_unsolicited_console_errors", "--nocapture"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(!stderr.contains("ICU4X data error"), "{stderr}");
}
