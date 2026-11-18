//! Debug-build ICU probes must respect application logging rather than print to stderr.
#![cfg(feature = "format-duration")]

use std::{
    process::Command,
    time::Duration,
    sync::atomic::{AtomicUsize, Ordering},
};
use xgpui_luma_l10n::{Locale, formatting::DurationStyle};

struct Diagnostics(AtomicUsize);
impl log::Log for Diagnostics {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &log::Record<'_>) {
        if record.target().starts_with("icu_provider") {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn flush(&self) {}
}
static DIAGNOSTICS: Diagnostics = Diagnostics(AtomicUsize::new(0));

#[test]
fn duration_probes_do_not_write_unsolicited_console_errors() {
    const CHILD: &str = "LUMA_DURATION_LOGGING_CHILD";
    if std::env::var_os(CHILD).is_some() {
        for tag in ["en", "de", "ja", "th-u-nu-thai"] {
            let locale: Locale = tag.parse().unwrap();
            for style in [DurationStyle::Long, DurationStyle::Short, DurationStyle::Narrow, DurationStyle::Digital] {
                for value in [Duration::ZERO, Duration::from_secs(3661), Duration::from_nanos(123456789)] {
                    assert!(!locale.fmt_duration(value, style).unwrap().is_empty());
                }
            }
        }
        // Applications that opt into warnings still receive ICU diagnostics.
        log::set_logger(&DIAGNOSTICS).unwrap();
        log::set_max_level(log::LevelFilter::Warn);
        let locale: Locale = "en".parse().unwrap();
        locale.fmt_duration(Duration::from_secs(3661), DurationStyle::Long).unwrap();
        assert!(DIAGNOSTICS.0.load(Ordering::Relaxed) > 0);
        return;
    }
    // A fresh process has no logger installed, like the default Lab startup.
    // Capture stderr directly: ICU's debug fallback bypasses the log facade.
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "duration_probes_do_not_write_unsolicited_console_errors", "--nocapture"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(!stderr.contains("ICU4X data error"), "{stderr}");
}
