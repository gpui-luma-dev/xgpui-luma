//! The Radix look in the Luma style language (issue #285).
//!
//! [`config`] extends the SDK's configuration with Radix's own axes: `tone` (accent or gray),
//! `radius` (none, small, medium, large, full) and the `high_contrast` state. [`sources`] lists the
//! look's `.lss` files, layered over the SDK's in [`stylesheet`]. [`RadixTokens`] is the native token
//! source: the color scales `gray[n]`, `accent[n]` and `tone[n]` for the bound mode, the accent
//! contrast color (`look.accent_contrast`) and the label font (`look.font_sans`). Sizes (`size 1`,
//! `size 3`) and variants live only in these files; size 2 and the surface variant are the base.

use std::sync::{Arc, OnceLock};

use gpui::Hsla;
use xgpui_luma::style::lss::{Color, Config, Diagnostics, Source, Stylesheet, Theme, TokenCx, TokenSource, Value};
use xgpui_luma::theme::ThemeMode;

use crate::look::Look;
use crate::scale::{ModeScales, ScaleFamily};
use crate::semantic::SemanticRole;

/// The SDK configuration plus Radix's `tone` and `radius` attributes and `high_contrast` state.
pub fn config() -> Arc<Config> {
    static CONFIG: OnceLock<Arc<Config>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            Arc::new(
                xgpui_luma::style::config()
                    .extend()
                    .attributes(&["tone", "radius"])
                    .states(&["high_contrast"])
                    .build()
                    .expect("the Radix style configuration is valid"),
            )
        })
        .clone()
}

macro_rules! radix_sources {
    ($($path:literal),+ $(,)?) => {
        /// The Radix look's style files, for the `look` layer.
        pub fn sources() -> Vec<Source> {
            vec![$(Source::new(concat!("radix/", $path), include_str!($path))),+]
        }

        #[cfg(test)]
        const SOURCE_PATHS: &[&str] = &[$($path),+];
    };
}

radix_sources!["tokens.lss", "checkbox.lss", "radio.lss", "switch.lss"];

/// The SDK's files with the Radix look's files over them.
pub fn stylesheet() -> Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Stylesheet::builder(config())
                .layer("sdk", xgpui_luma::style::sources())
                .layer("look", sources())
                .compile()
                .unwrap_or_else(|d| panic!("the Radix style files compile:\n{d}"))
        })
        .clone()
}

/// The style mode name for a theme mode.
pub fn mode_name(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

/// `look`'s scales bound to the Radix stylesheet in the look's current mode.
pub fn theme(look: &Look) -> Result<Theme, Diagnostics> {
    Theme::bind(&stylesheet(), Arc::new(RadixTokens::new(look)), mode_name(look.mode()))
}

/// The last bound theme of a look, keyed by the look's revision (bumped by every setter) and mode.
#[derive(Default)]
pub(crate) struct ThemeCache(std::sync::Mutex<Option<(u64, ThemeMode, Theme)>>);

impl ThemeCache {
    pub(crate) fn get(&self, look: &Look) -> Theme {
        let (revision, mode) = (look.revision(), look.mode());
        let mut cached = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((cached_revision, cached_mode, theme)) = cached.as_ref()
            && *cached_revision == revision
            && *cached_mode == mode
        {
            return theme.clone();
        }
        // The look's files are compiled in and bind for every look the tests cover; should one still fail
        // to bind, the SDK alone keeps the control drawable.
        let theme = self::theme(look).unwrap_or_else(|_| xgpui_luma::style::default_theme());
        *cached = Some((revision, mode, theme.clone()));
        theme
    }
}

struct ModeTokens {
    /// The light-mode gray scale, whatever the bound mode (switch thumbs keep their light palette).
    light: ModeScales,
    scales: ModeScales,
    /// The color on solid accent faces (custom contrast, or dark labels for bright accents).
    accent_contrast: Hsla,
}

/// Native Radix tokens, captured from a [`Look`] for both modes; the bound mode picks one.
pub struct RadixTokens {
    light: ModeTokens,
    dark: ModeTokens,
    font_sans: Arc<str>,
}

impl RadixTokens {
    pub fn new(look: &Look) -> Self {
        let mode = |mode| ModeTokens {
            scales: look.scales_for(mode),
            light: look.scales_for(ThemeMode::Light),
            accent_contrast: look.resolve_role_for(SemanticRole::PrimaryForeground, mode).hsla(),
        };
        Self {
            light: mode(ThemeMode::Light),
            dark: mode(ThemeMode::Dark),
            font_sans: crate::typography::font_family(look).as_ref().into(),
        }
    }

    fn mode(&self, cx: &TokenCx<'_>) -> &ModeTokens {
        if cx.mode() == "dark" { &self.dark } else { &self.light }
    }
}

const ROOTS: &[&str] = &["gray", "accent", "tone", "light_gray", "look"];

fn color(hsla: Hsla) -> Value {
    let rgb = hsla.to_rgb();
    Value::Color(Color::srgb(rgb.r, rgb.g, rgb.b, rgb.a))
}

impl TokenSource for RadixTokens {
    fn roots(&self) -> &[&str] {
        ROOTS
    }

    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value> {
        match path {
            "look.accent_contrast" => Some(color(self.mode(cx).accent_contrast)),
            "look.font_sans" => Some(Value::Str(self.font_sans.clone())),
            _ => None,
        }
    }

    /// `gray[n]`, `accent[n]`, `tone[n]` (the accent scale, or gray for `tone gray`), and `light_gray[n]`.
    fn index(&self, root: &str, index: i64, cx: &TokenCx<'_>) -> Option<Value> {
        let step = u8::try_from(index).ok().filter(|n| (1..=12).contains(n))?;
        if root == "light_gray" {
            return Some(color(self.mode(cx).light.resolved(ScaleFamily::Gray, step).hsla()));
        }
        let family = match root {
            "gray" => ScaleFamily::Gray,
            "accent" => ScaleFamily::Color,
            "tone" if cx.attr("tone") == Some("gray") => ScaleFamily::Gray,
            "tone" => ScaleFamily::Color,
            _ => return None,
        };
        Some(color(self.mode(cx).scales.resolved(family, step).hsla()))
    }

    fn describe(&self, path: &str, cx: &TokenCx<'_>) -> Option<String> {
        match path {
            "look.accent_contrast" => Some(format!("Radix accent contrast ({})", cx.mode())),
            "look.font_sans" => Some("Radix common stylesheet font.sans".into()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
