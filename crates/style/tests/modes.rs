//! Theme modes declared in the config, including modes based on another (issue #294).

mod common;

use std::sync::Arc;

use common::*;
use xgpui_luma_style::{Config, ControlId, Source, StyleKey, Stylesheet, Theme, TokenCx, TokenSource, Value};

/// The SDK's config plus a look's `hi_contrast` mode based on light.
fn config() -> Arc<Config> {
    Arc::new(luma_config().extend().mode_based_on("hi_contrast", "light").build().expect("valid"))
}

/// A native root that reports the scheme it was asked in: 1 for light, 2 for dark.
struct SchemeProbe;

impl TokenSource for SchemeProbe {
    fn roots(&self) -> &[&str] {
        &["native"]
    }

    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value> {
        (path == "native.scheme").then(|| Value::Number(if cx.scheme() == "dark" { 2.0 } else { 1.0 }))
    }
}

fn sheet() -> Stylesheet {
    let tokens = "tokens {
  modal {
    own    = 0
    shared = 10
    mode light       { own = 1  from_light = 11 }
    mode dark        { own = 2  from_light = 22 }
    mode hi_contrast { own = 3 }
  }
}";
    let control = "probe {
  own: number = modal.own
  shared: number = modal.shared
  from_light: number = modal.from_light
  native_scheme: number = native.scheme
}";
    Stylesheet::builder(config())
        .layer("sdk", [Source::new("tokens.lss", tokens), Source::new("probe.lss", control)])
        .compile()
        .unwrap_or_else(|d| panic!("{d}"))
}

fn values(mode: &str) -> [f32; 4] {
    let theme = Theme::bind(&sheet(), Arc::new(SchemeProbe), mode).unwrap_or_else(|d| panic!("{mode}: {d}"));
    let resolved = theme.resolve(&StyleKey::new(ControlId::new("probe")));
    let value = |name: &str| resolved.root().value(name).and_then(Value::as_number).unwrap_or(f32::NAN);
    [value("own"), value("shared"), value("from_light"), value("native_scheme")]
}

#[test]
fn a_based_on_mode_falls_back_to_its_base_then_the_shared_values() {
    // [own, shared, from_light, native scheme]
    assert_eq!(values("light"), [1.0, 10.0, 11.0, 1.0]);
    assert_eq!(values("dark"), [2.0, 10.0, 22.0, 2.0]);
    assert_eq!(values("hi_contrast"), [3.0, 10.0, 11.0, 1.0], "own value, light's, shared; light scheme");
}

#[test]
fn a_bound_theme_reports_its_mode_and_scheme() {
    let theme = Theme::bind(&sheet(), Arc::new(SchemeProbe), "hi_contrast").expect("binds");
    assert_eq!((theme.mode(), theme.scheme()), ("hi_contrast", "light"));
    let dark = theme.with_mode("dark").expect("dark");
    assert_eq!((dark.mode(), dark.scheme()), ("dark", "dark"));
}

/// `Theme::token` reads a token as a rule would: the mode's own value, its base's, then the shared one.
#[test]
fn token_reads_the_mode_value() {
    let theme = |mode: &str| Theme::bind(&sheet(), Arc::new(SchemeProbe), mode).expect("binds");
    let number = |theme: &Theme, path: &str| theme.token(path).expect("a token").as_number();
    for (mode, own, from_light) in [("light", 1.0, 11.0), ("dark", 2.0, 22.0), ("hi_contrast", 3.0, 11.0)] {
        let theme = theme(mode);
        assert_eq!(number(&theme, "modal.own"), Some(own), "{mode}");
        assert_eq!(number(&theme, "modal.shared"), Some(10.0), "{mode}");
        assert_eq!(number(&theme, "modal.from_light"), Some(from_light), "{mode} falls back to its base");
    }
    let edited = theme("light").with_token_values(&[("modal.own", Value::Number(7.0))]).expect("edits");
    assert_eq!(number(&edited, "modal.own"), Some(7.0), "live edits apply");
    assert_eq!(sheet().token_paths(), ["modal.from_light", "modal.own", "modal.shared"]);
    let unknown = theme("light").token("modal.missing").expect_err("unknown");
    assert!(unknown.message.contains("modal.missing"), "{unknown}");
}
