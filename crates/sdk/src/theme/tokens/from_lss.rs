//! The SDK token tables read from `.lss` (issue #314): the one source of the SDK's palette,
//! elevation, metrics and typography values is `theme/palette.lss` and `theme/tokens.lss`.
//!
//! [`LumaThemeMode::from_theme`] reads a theme's `palette.*`, `elevation.*`, `metrics.*` and
//! `typography.*` tokens into the typed tables. The tables reject unknown and missing fields, so a
//! token the `.lss` adds or drops without the Rust type following is an error, not a silent default.

use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use xgpui_luma_style::{Theme, Value};

use super::{LumaThemeMode, ThemeMode};
use crate::color::ColorValue;

/// The token groups the typed tables hold. Other groups (`geometry`, `motion`) are read by `.lss`
/// rules only.
const GROUPS: [&str; 4] = ["palette", "elevation", "metrics", "typography"];

impl LumaThemeMode {
    /// `theme`'s palette, elevation, metrics and typography tokens in its mode, as typed tables.
    /// Errors name the token or field that doesn't fit.
    pub fn from_theme(theme: &Theme) -> Result<Self> {
        let mut root = groups(theme, &GROUPS, &[])?;
        let mut group = |name: &str| root.remove(name).unwrap_or_else(|| toml::Value::Table(toml::Table::new()));
        let palette = group("palette").try_into().context("palette tokens")?;
        let elevation = group("elevation").try_into().context("elevation tokens")?;
        let metrics = group("metrics").try_into().context("metrics tokens")?;
        let typography = group("typography").try_into().context("typography tokens")?;
        Ok(Self::new(palette, metrics, typography, elevation))
    }
}

impl super::MetricTokens {
    /// `theme`'s `metrics.*` tokens for a control with these attribute values (e.g.
    /// `[("size", "sm")]`, so `size sm { … }` definitions apply), as a typed table.
    pub fn from_theme(theme: &Theme, attrs: &[(&str, &str)]) -> Result<Self> {
        let mut root = groups(theme, &["metrics"], attrs)?;
        root.remove("metrics")
            .unwrap_or_else(|| toml::Value::Table(toml::Table::new()))
            .try_into()
            .context("metrics tokens")
    }
}

/// The tokens of `theme` in the named groups, as nested TOML tables keyed by path.
fn groups(theme: &Theme, names: &[&str], attrs: &[(&str, &str)]) -> Result<toml::Table> {
    let mut root = toml::Table::new();
    for path in theme.stylesheet().token_paths() {
        if !path.split('.').next().is_some_and(|group| names.contains(&group)) {
            continue;
        }
        let value = theme.token_with(path, attrs).map_err(|d| anyhow!("{d}"))?;
        insert(&mut root, path, table_value(path, value)?);
    }
    Ok(root)
}

/// The SDK's own tables for `mode`, read once from the SDK's `.lss` files. The files are built in
/// and tested, so failing to read them is a build defect, like [`crate::style::default_theme`].
pub(super) fn sdk(mode: ThemeMode) -> &'static LumaThemeMode {
    static LIGHT: OnceLock<LumaThemeMode> = OnceLock::new();
    static DARK: OnceLock<LumaThemeMode> = OnceLock::new();
    let (cell, theme): (_, fn() -> Theme) = match mode {
        ThemeMode::Light => (&LIGHT, crate::style::default_theme),
        ThemeMode::Dark => (&DARK, crate::style::dark_theme),
    };
    cell.get_or_init(|| {
        LumaThemeMode::from_theme(&theme())
            .unwrap_or_else(|e| panic!("the SDK token files fill the token tables: {e:#}"))
    })
}

/// A token value in the shape its typed table field deserializes from.
fn table_value(path: &str, value: Value) -> Result<toml::Value> {
    let color = |c: xgpui_luma_style::Color| toml::Value::try_from(ColorValue::from(c)).context("a color");
    Ok(match value {
        Value::Number(n) => toml::Value::Float(f64::from(n)),
        Value::Color(c) => color(c)?,
        Value::Str(family) if path.starts_with("typography.font.") => {
            toml::Value::Table(toml::Table::from_iter([("family".into(), toml::Value::String(family.to_string()))]))
        }
        Value::Text(t) => toml::Value::Table(toml::Table::from_iter([
            ("size".into(), toml::Value::Float(f64::from(t.size))),
            ("line_height".into(), toml::Value::Float(f64::from(t.line_height))),
            ("weight".into(), toml::Value::Float(f64::from(t.weight))),
        ])),
        Value::Shadow(layers) => {
            let layers = layers
                .iter()
                .map(|l| {
                    Ok(toml::Value::Table(toml::Table::from_iter([
                        ("color".into(), color(l.color)?),
                        ("inset".into(), toml::Value::Boolean(l.inset)),
                        ("offset_x".into(), toml::Value::Float(f64::from(l.x))),
                        ("offset_y".into(), toml::Value::Float(f64::from(l.y))),
                        ("blur".into(), toml::Value::Float(f64::from(l.blur))),
                        ("spread".into(), toml::Value::Float(f64::from(l.spread))),
                    ])))
                })
                .collect::<Result<Vec<_>>>()?;
            toml::Value::Table(toml::Table::from_iter([("layers".into(), toml::Value::Array(layers))]))
        }
        // `none = []`: no shadow layers.
        Value::List(items) if items.is_empty() && path.starts_with("elevation.") => {
            toml::Value::Table(toml::Table::from_iter([("layers".into(), toml::Value::Array(Vec::new()))]))
        }
        other => return Err(anyhow!("token {path}: {other:?} has no place in the token tables")),
    })
}

/// Put `value` at `path` (dot-separated) in `root`, creating tables on the way.
fn insert(root: &mut toml::Table, path: &str, value: toml::Value) {
    let mut table = root;
    let mut segments = path.split('.').peekable();
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            table.insert(segment.to_string(), value);
            return;
        }
        let entry = table.entry(segment.to_string()).or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if !entry.is_table() {
            *entry = toml::Value::Table(toml::Table::new());
        }
        let toml::Value::Table(next) = entry else { return };
        table = next;
    }
}

#[cfg(test)]
mod tests {
    use xgpui_luma_style::Source;

    use super::*;
    use crate::style::fixture;

    /// The tables hold the theme's own token values, in each mode: read against the colorful
    /// fixture, where every token is distinct.
    #[test]
    fn tables_hold_the_theme_tokens_in_each_mode() {
        for theme in fixture::themes() {
            let tables = LumaThemeMode::from_theme(&theme).unwrap_or_else(|e| panic!("{}: {e:#}", theme.mode()));
            let color =
                |path: &str| ColorValue::from(fixture::token(&theme, path, "color").as_color().expect("a color"));
            let number = |path: &str| fixture::token(&theme, path, "number").as_number().expect("a number");
            assert_eq!(tables.palette.form.input.invalid_border, color("palette.form.input.invalid_border"));
            assert_eq!(tables.palette.data.accent_3, color("palette.data.accent_3"));
            assert_eq!(tables.elevation.menu.layers.len(), 2);
            assert_eq!(tables.elevation.menu.layers[0].blur, {
                let Value::Shadow(layers) = fixture::token(&theme, "elevation.menu", "shadow") else {
                    panic!()
                };
                layers[0].blur
            });
            assert!(tables.elevation.none.layers.is_empty());
            assert_eq!(tables.metrics.control.control_height, number("metrics.control.control_height"));
            assert_eq!(tables.metrics.focus.offset, number("metrics.focus.offset"));
            assert_eq!(tables.typography.font.mono.family, "Fixture Mono");
            assert_eq!(tables.typography.text.role.h3.size, number("typography.text.role.h3.size"));
        }
        let (light, dark) = (fixture::light(), fixture::dark());
        let background = |theme| LumaThemeMode::from_theme(theme).expect("tables").palette.app.background;
        assert_ne!(background(&light), background(&dark), "dark reads the mode dark values");
    }

    /// A token the Rust tables don't have is an error that names it, not silently dropped.
    #[test]
    fn an_extra_token_is_an_error() {
        let theme = fixture::light();
        let sheet = theme
            .stylesheet()
            .with_source("sdk", Source::new("extra.lss", "tokens { palette { app { glow = srgb(1, 1, 0) } } }"))
            .expect("compiles");
        let error = LumaThemeMode::from_theme(&theme.with_stylesheet(&sheet).expect("binds")).expect_err("rejected");
        assert!(format!("{error:#}").contains("glow"), "{error:#}");
    }

    /// A token the Rust tables need but the theme lacks is an error that names it.
    #[test]
    fn a_missing_token_is_an_error() {
        let config = crate::style::config();
        let tokens = include_str!("../../../tests/fixtures/colorful/palette.lss")
            .lines()
            .filter(|line| !line.trim_start().starts_with("accent_5"))
            .collect::<Vec<_>>()
            .join("\n");
        let sheet = xgpui_luma_style::Stylesheet::builder(config)
            .layer(
                "sdk",
                [
                    Source::new("tokens.lss", include_str!("../../../tests/fixtures/colorful/tokens.lss")),
                    Source::new("palette.lss", tokens),
                ],
            )
            .compile()
            .expect("compiles");
        let theme = Theme::bind(&sheet, std::sync::Arc::new(xgpui_luma_style::NoNativeTokens), "light").expect("binds");
        let error = LumaThemeMode::from_theme(&theme).expect_err("rejected");
        assert!(format!("{error:#}").contains("accent_5"), "{error:#}");
    }

    /// Size-scoped metrics apply for a control of that size; others keep the unscoped value.
    #[test]
    fn metrics_read_size_scoped_values() {
        let theme = fixture::light();
        let sheet = theme
            .stylesheet()
            .with_source("sdk", Source::new("sized.lss", "tokens { metrics { control { size sm { gap = 4 } } } }"))
            .expect("compiles");
        let theme = theme.with_stylesheet(&sheet).expect("binds");
        let base = crate::theme::MetricTokens::from_theme(&theme, &[]).expect("metrics");
        let small = crate::theme::MetricTokens::from_theme(&theme, &[("size", "sm")]).expect("metrics");
        assert_eq!(small.control.gap, 4.0);
        assert_eq!(base.control.gap, fixture::token(&theme, "metrics.control.gap", "number").as_number().unwrap());
        assert_eq!(small.control.padding_x, base.control.padding_x, "unscoped tokens are shared");
    }

    /// The SDK's own tables are the SDK theme's tokens.
    #[test]
    fn sdk_tables_come_from_the_sdk_theme() {
        let from_theme = LumaThemeMode::from_theme(&crate::style::dark_theme()).expect("tables");
        assert_eq!(sdk(ThemeMode::Dark).palette, from_theme.palette);
        assert_eq!(crate::theme::MetricTokens::default(), from_theme.metrics);
        assert_eq!(crate::theme::LumaTypography::default(), from_theme.typography);
    }
}
