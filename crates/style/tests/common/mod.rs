//! Shared setup that mirrors what the SDK and the looks would do: one global config, the style
//! files each crate ships, and each look's native token source.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use xgpui_luma_style::{
    Color, Config, FunctionSig, Kind, NoNativeTokens, ShadowLayer, SnapPolicy, Source, StyleKey, Stylesheet, Theme,
    TokenCx, TokenSource, Value,
};

// ---- the SDK's configuration ---------------------------------------------------------------

/// What a Radix look would build: the SDK config plus Radix's own axes.
pub fn radix_config() -> Arc<Config> {
    Arc::new(
        luma_config()
            .extend()
            .attributes(&["tone", "radius"])
            .states(&["high_contrast"])
            .build()
            .expect("valid"),
    )
}

/// What `xgpui_luma::style::config()` returns: SDK states, generic attributes and the SDK's modes.
pub fn luma_config() -> Arc<Config> {
    Arc::new(
        Config::builder()
            .layers(&["sdk", "look", "app"])
            .states(&[
                "hover", "pressed", "focused", "disabled", "checked", "selected", "active", "expanded", "open",
                "invalid",
            ])
            .exclusive(&["disabled", "pressed", "hover"])
            .attributes(&["variant", "size"])
            .modes(&["light", "dark"])
            .snap(SnapPolicy::Lengths)
            .build()
            .expect("valid config"),
    )
}

// ---- the files each crate ships ------------------------------------------------------------

macro_rules! fixture {
    ($path:literal) => {
        Source::new($path, include_str!(concat!("../fixtures/", $path)))
    };
}

/// What `xgpui_luma::style::sources()` would return.
pub fn sdk_sources() -> Vec<Source> {
    vec![
        fixture!("sdk/tokens.lss"),
        fixture!("sdk/checkbox.lss"),
        fixture!("sdk/button.lss"),
        fixture!("sdk/popup_menu.lss"),
    ]
}

/// What `shadcn::style::sources()` would return.
pub fn shadcn_sources() -> Vec<Source> {
    vec![
        fixture!("shadcn/tokens.lss"),
        fixture!("shadcn/checkbox.lss"),
        fixture!("shadcn/button.lss"),
        fixture!("shadcn/popup_menu.lss"),
    ]
}

/// What `radix::style::sources()` would return.
pub fn radix_sources() -> Vec<Source> {
    vec![fixture!("radix/tokens.lss"), fixture!("radix/checkbox.lss")]
}

pub fn sdk_sheet() -> Stylesheet {
    Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .compile()
        .unwrap_or_else(|d| panic!("{d}"))
}

pub fn shadcn_sheet() -> Stylesheet {
    Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .layer("look", shadcn_sources())
        .compile()
        .unwrap_or_else(|d| panic!("{d}"))
}

pub fn radix_sheet() -> Stylesheet {
    Stylesheet::builder(radix_config())
        .layer("sdk", sdk_sources())
        .layer("look", radix_sources())
        .compile()
        .unwrap_or_else(|d| panic!("{d}"))
}

/// The SDK alone: no native tokens, light mode.
pub fn sdk_theme() -> Theme {
    Theme::bind(&sdk_sheet(), Arc::new(NoNativeTokens), "light").unwrap_or_else(|d| panic!("{d}"))
}

pub fn shadcn_theme(mode: &str) -> Theme {
    Theme::bind(&shadcn_sheet(), Arc::new(ShadcnCatalog::new()), mode).unwrap_or_else(|d| panic!("{d}"))
}

pub fn radix_theme(mode: &str) -> Theme {
    Theme::bind(&radix_sheet(), Arc::new(RadixScales), mode).unwrap_or_else(|d| panic!("{d}"))
}

/// A key from names, as a tool would build it (templates use handles; see `templates.rs`).
pub fn key(theme: &Theme, control: &str, attrs: &[(&str, &str)], states: &[&str]) -> StyleKey {
    theme.key_named(control, attrs, states).unwrap_or_else(|d| panic!("{d}"))
}

/// The SDK palette value a token resolves to, read straight from the SDK token numbers.
pub fn sdk_gray(lightness: f32) -> Color {
    let l = lightness / 100.0;
    Color::srgb(l, l, l, 1.0)
}

pub fn assert_color(actual: Option<Color>, expected: Color, what: &str) {
    let actual = actual.unwrap_or_else(|| panic!("{what}: no color"));
    assert!(actual.approx_eq(expected, 1e-4), "{what}: {actual:?} != {expected:?}");
}

// ---- Shadcn: the CSS catalog ---------------------------------------------------------------

/// A Shadcn CSS theme (the default neutral theme), as the look would load it from CSS.
pub struct ShadcnCatalog {
    light: HashMap<&'static str, Value>,
    dark: HashMap<&'static str, Value>,
}

fn ok(l: f32) -> Value {
    Value::Color(Color::oklch(l, 0.0, 0.0, 1.0))
}

impl ShadcnCatalog {
    pub fn new() -> Self {
        let shadow_xs = Value::Shadow(
            vec![ShadowLayer {
                x: 0.0,
                y: 1.0,
                blur: 2.0,
                spread: 0.0,
                color: Color::BLACK.with_alpha(0.05),
                inset: false,
            }]
            .into(),
        );
        let light = HashMap::from([
            ("background", ok(1.0)),
            ("foreground", ok(0.145)),
            ("card", ok(1.0)),
            ("popover", ok(1.0)),
            ("primary", ok(0.205)),
            ("primary_foreground", ok(0.985)),
            ("secondary", ok(0.97)),
            ("secondary_foreground", ok(0.205)),
            ("muted", ok(0.97)),
            ("muted_foreground", ok(0.556)),
            ("accent", ok(0.97)),
            ("accent_foreground", ok(0.205)),
            ("border", ok(0.922)),
            ("input", ok(0.922)),
            ("ring", ok(0.708)),
            ("radius", Value::Number(10.0)),
            ("shadow_xs", shadow_xs.clone()),
            ("font_sans", Value::Str("Geist".into())),
        ]);
        let mut dark = light.clone();
        for (k, l) in [
            ("background", 0.145),
            ("foreground", 0.985),
            ("card", 0.205),
            ("popover", 0.205),
            ("primary", 0.922),
            ("primary_foreground", 0.205),
            ("secondary", 0.269),
            ("secondary_foreground", 0.985),
            ("muted", 0.269),
            ("muted_foreground", 0.708),
            ("accent", 0.269),
            ("accent_foreground", 0.985),
            ("ring", 0.556),
        ] {
            dark.insert(k, ok(l));
        }
        dark.insert("border", Value::Color(Color::oklch(1.0, 0.0, 0.0, 0.1)));
        dark.insert("input", Value::Color(Color::oklch(1.0, 0.0, 0.0, 0.15)));
        Self { light, dark }
    }

    pub fn color(&self, mode: &str, name: &str) -> Color {
        let table = if mode == "dark" { &self.dark } else { &self.light };
        table[name].as_color().expect("a color token")
    }

    /// The look's state colors: shift lightness by the mode's delta.
    pub fn layer(color: Color, state: &str, mode: &str) -> Color {
        let delta = match (mode, state) {
            ("dark", "hover") => 0.04,
            ("dark", "pressed") => 0.08,
            (_, "hover") => -0.03,
            (_, "pressed") => -0.06,
            _ => 0.0,
        };
        Color { l: (color.l + delta).clamp(0.0, 1.0), ..color }
    }
}

impl TokenSource for ShadcnCatalog {
    fn roots(&self) -> &[&str] {
        &["css"]
    }

    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value> {
        let name = path.strip_prefix("css.")?;
        let table = if cx.mode() == "dark" { &self.dark } else { &self.light };
        table.get(name).cloned()
    }

    fn function(&self, name: &str) -> Option<FunctionSig> {
        Some(match name {
            "px" => FunctionSig { min_args: 1, max_args: 1, returns: Kind::Number },
            "css_shadow" => FunctionSig { min_args: 1, max_args: 1, returns: Kind::Shadow },
            "css_font" => FunctionSig { min_args: 1, max_args: 1, returns: Kind::Font },
            "layer" => FunctionSig { min_args: 2, max_args: 2, returns: Kind::Color },
            "first_layer" => FunctionSig { min_args: 2, max_args: 8, returns: Kind::Color },
            _ => return None,
        })
    }

    fn call(&self, name: &str, args: &[Value], cx: &TokenCx<'_>) -> Result<Value, String> {
        let present = |v: &Value| !matches!(v, Value::None);
        match name {
            // An absent argument arrives as `none`; returning `none` lets `first()` fall back.
            "px" | "css_shadow" | "css_font" => Ok(args[0].clone()),
            "layer" => match (&args[0], args[1].as_str()) {
                (Value::Color(c), Some(state)) => Ok(Value::Color(Self::layer(*c, state, cx.mode()))),
                (Value::None, _) => Ok(Value::None),
                _ => Err("layer(color, state)".into()),
            },
            "first_layer" => {
                let (state, colors) = args.split_last().expect("arity checked");
                let state = state.as_str().ok_or("first_layer(…, state)")?;
                match colors.iter().find(|v| present(v)) {
                    Some(Value::Color(c)) => Ok(Value::Color(Self::layer(*c, state, cx.mode()))),
                    _ => Ok(Value::None),
                }
            }
            other => Err(format!("unknown function {other}")),
        }
    }
}

// ---- Radix: the color scales ---------------------------------------------------------------

pub struct RadixScales;

const LIGHT_L: [f32; 12] = [0.99, 0.98, 0.955, 0.93, 0.91, 0.885, 0.85, 0.79, 0.64, 0.61, 0.50, 0.24];
const DARK_L: [f32; 12] = [0.18, 0.21, 0.25, 0.28, 0.31, 0.35, 0.41, 0.50, 0.64, 0.68, 0.78, 0.94];

/// Radix step `n` (1 to 12) of `scale` ("gray" or "accent") in `mode`.
pub fn radix_step(scale: &str, n: usize, mode: &str) -> Color {
    let l = if mode == "dark" { DARK_L[n - 1] } else { LIGHT_L[n - 1] };
    match scale {
        "gray" => Color::oklch(l, 0.004, 250.0, 1.0),
        _ => Color::oklch(l, 0.02 + 0.015 * n as f32, 275.0, 1.0),
    }
}

impl TokenSource for RadixScales {
    fn roots(&self) -> &[&str] {
        &["gray", "accent", "tone"]
    }

    fn token(&self, _path: &str, _cx: &TokenCx<'_>) -> Option<Value> {
        None
    }

    fn index(&self, root: &str, index: i64, cx: &TokenCx<'_>) -> Option<Value> {
        if !(1..=12).contains(&index) {
            return None;
        }
        let scale = match root {
            "tone" if cx.attr("tone") == Some("gray") => "gray",
            "tone" => "accent",
            other => other,
        };
        Some(Value::Color(radix_step(scale, index as usize, cx.mode())))
    }

    fn function(&self, name: &str) -> Option<FunctionSig> {
        matches!(name, "resting_shadow" | "checked_shadow").then_some(FunctionSig {
            min_args: 2,
            max_args: 2,
            returns: Kind::Shadow,
        })
    }

    fn call(&self, name: &str, args: &[Value], _cx: &TokenCx<'_>) -> Result<Value, String> {
        let (Some(a), Some(b)) = (args[0].as_color(), args[1].as_color()) else {
            return Err(format!("{name}(color, color)"));
        };
        let inset =
            |y: f32, blur: f32, spread: f32, color: Color| ShadowLayer { x: 0.0, y, blur, spread, color, inset: true };
        let layers = match name {
            // Radix --shadow-1: a hairline rim with a soft top fade.
            "resting_shadow" => vec![inset(0.0, 0.0, 1.0, a), inset(1.5, 2.0, 0.0, b)],
            // The checked classic face: a light rim on top, a dark one underneath.
            "checked_shadow" => vec![inset(0.5, 0.5, 0.0, a), inset(-0.5, 0.5, 0.0, b)],
            other => return Err(format!("unknown function {other}")),
        };
        Ok(Value::Shadow(layers.into()))
    }
}
