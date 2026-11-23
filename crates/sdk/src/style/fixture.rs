//! A colorful test theme (issue #314): the SDK's control files over test-only token files.
//!
//! `tests/fixtures/colorful/{tokens,palette}.lss` stand in for `theme/tokens.lss` and
//! `theme/palette.lss`. Every token there has its own value (each color its own hue, light and dark
//! different), so a control property that resolves to a token's value proves it reads that token.
//! Tests compare against [`token`], never against literals, so changing a shipped `.lss` value
//! can't break them.

use std::sync::OnceLock;

use xgpui_luma_style::{ControlId, NoNativeTokens, Source, StyleKey, Stylesheet, Theme, Value};

const TOKEN_FILES: [&str; 2] = ["theme/tokens.lss", "theme/palette.lss"];

/// The SDK's control files with the fixture token files in place of the shipped ones.
pub(crate) fn sources() -> Vec<Source> {
    let mut sources = vec![
        Source::new("fixture/tokens.lss", include_str!("../../tests/fixtures/colorful/tokens.lss")),
        Source::new("fixture/palette.lss", include_str!("../../tests/fixtures/colorful/palette.lss")),
    ];
    let is_token_file = |s: &Source| TOKEN_FILES.iter().any(|t| *s.name == *format!("sdk/{t}"));
    let shipped = super::sources();
    assert_eq!(
        shipped.iter().filter(|s| is_token_file(s)).count(),
        TOKEN_FILES.len(),
        "the fixture replaces every token file"
    );
    sources.extend(shipped.into_iter().filter(|s| !is_token_file(s)));
    sources
}

pub(crate) fn stylesheet() -> Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Stylesheet::builder(super::config())
                .layer("sdk", sources())
                .compile()
                .unwrap_or_else(|d| panic!("the fixture compiles:\n{d}"))
        })
        .clone()
}

/// The fixture theme in light mode.
pub(crate) fn light() -> Theme {
    mode("light")
}

/// The fixture theme in dark mode.
pub(crate) fn dark() -> Theme {
    mode("dark")
}

/// Both modes, for tests that hold in each.
pub(crate) fn themes() -> [Theme; 2] {
    [light(), dark()]
}

fn mode(name: &str) -> Theme {
    Theme::bind(&stylesheet(), std::sync::Arc::new(NoNativeTokens), name)
        .unwrap_or_else(|d| panic!("the fixture binds in {name}:\n{d}"))
}

/// A token's value in `theme`, read through a one-property probe control. `kind` is the property
/// kind to declare (`color`, `length`, `number`, `shadow`, `text`, …).
pub(crate) fn token(theme: &Theme, path: &str, kind: &str) -> Value {
    let sheet = theme
        .stylesheet()
        .with_source("sdk", Source::new("probe.lss", format!("probe {{\n  value: {kind} = {path}\n}}")))
        .unwrap_or_else(|d| panic!("probe for {path} compiles:\n{d}"));
    let probe = theme.with_stylesheet(&sheet).expect("the probe binds in the theme's mode");
    probe
        .resolve(&StyleKey::new(ControlId::new("probe")))
        .root()
        .value("value")
        .cloned()
        .unwrap_or_else(|| panic!("{path} resolves"))
}

/// Asserts that `control`'s `part.prop` in `states` resolves to `path`'s value in `theme`. Each
/// fixture token is unique, so this holds only when the property reads that token. `part` is a part
/// name, or `"root"` for the control's own properties.
#[track_caller]
pub(crate) fn assert_reads(theme: &Theme, control: &str, states: &[&str], part: &str, prop: &str, path: &str) {
    let key = theme.key_named(control, &[], states).unwrap_or_else(|d| panic!("{control} {states:?}: {d}"));
    let resolved = theme.resolve(&key);
    let view = if part == "root" {
        resolved.root()
    } else {
        let id = theme.stylesheet().part(key.control(), part).unwrap_or_else(|| panic!("{control} has part {part}"));
        resolved.part(id)
    };
    let got = view.value(prop).unwrap_or_else(|| panic!("{control}.{part}.{prop} {states:?} has a value"));
    let kind = got.kind().map_or("number", xgpui_luma_style::Kind::name);
    let want = token(theme, path, kind);
    assert_eq!(*got, want, "{control}.{part}.{prop} {states:?} in {} reads {path}", theme.mode());
}
