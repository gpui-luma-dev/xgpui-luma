//! The Shadcn look in the Luma style language (issue #285).
//!
//! [`sources`] lists the look's `.lss` files, layered over the SDK's in [`stylesheet`]. [`ShadcnTokens`]
//! is the native token source: the `css` root reads the bound mode's CSS catalog, and the look functions
//! (`outline_layer`, `action_layer`, `action_foreground`) call the existing [`LookResolver`]. Sizes
//! (`size sm`, `size lg`) and variants live only in these files; `md` is the base.

use std::sync::{Arc, OnceLock};

use gpui::Hsla;
use xgpui_luma::style::lss::{
    Color, Diagnostics, FunctionSig, Kind, ShadowLayer, Source, Stylesheet, Theme, TokenCx, TokenSource, Value,
};
use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::catalog::{first_font_family, parse_length_px};
use crate::controls::ShadcnButtonStyle;
use crate::look::ShadcnLook;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;

macro_rules! shadcn_sources {
    ($($path:literal),+ $(,)?) => {
        /// The Shadcn look's style files, for the `look` layer.
        pub fn sources() -> Vec<Source> {
            vec![$(Source::new(concat!("shadcn/", $path), include_str!($path))),+]
        }

        #[cfg(test)]
        const SOURCE_PATHS: &[&str] = &[$($path),+];
    };
}

shadcn_sources![
    "tokens.lss",
    "checkbox.lss",
    "radio.lss",
    "switch.lss",
    "button.lss",
    "toggle.lss",
    "textfield.lss",
    "selector.lss",
    "floating_menu.lss",
    "context_menu.lss",
    "tooltip.lss",
    "overlay_window.lss",
    "autocomplete.lss",
    "slider.lss",
    "progress.lss",
    "control_group.lss",
    "toolbar.lss",
    "stepper.lss",
    "accordion.lss",
    "pager.lss",
    "tabs.lss",
    "sidebar.lss",
    "drag_preview.lss",
    "tree_view.lss",
    "table.lss",
    "listbox.lss"
];

/// The SDK's files with the Shadcn look's files over them.
pub fn stylesheet() -> Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET
        .get_or_init(|| {
            Stylesheet::builder(xgpui_luma::style::config())
                .layer("sdk", xgpui_luma::style::sources())
                .layer("look", sources())
                .compile()
                .unwrap_or_else(|d| panic!("the Shadcn style files compile:\n{d}"))
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

/// `look`'s catalog bound to the Shadcn stylesheet in the look's current mode.
pub fn theme(look: &ShadcnLook) -> Result<Theme, Diagnostics> {
    Theme::bind(&stylesheet(), Arc::new(ShadcnTokens::new(look)), mode_name(look.mode()))
}

/// One mode's tokens bound to the Shadcn stylesheet in that mode. The SDK alone if the catalog cannot bind.
pub(crate) fn theme_for_mode(tokens: &ShadcnModeTokens) -> Theme {
    let tokens = Arc::new(tokens.clone());
    let mode = tokens.theme_mode;
    Theme::bind(&stylesheet(), Arc::new(ShadcnTokens { light: Arc::clone(&tokens), dark: tokens }), mode_name(mode))
        .unwrap_or_else(|_| xgpui_luma::style::default_theme())
}

/// The last bound theme of a look. Keyed by the identity of the mode tokens it was bound to (the cached
/// theme keeps them alive, so the key cannot be reused) and the mode.
#[derive(Default)]
pub(crate) struct ThemeCache(std::sync::Mutex<Option<(usize, ThemeMode, Theme)>>);

impl ThemeCache {
    pub(crate) fn get(&self, look: &ShadcnLook) -> Theme {
        let mode = look.mode();
        let tokens = look.mode_tokens_for(mode);
        let id = Arc::as_ptr(&tokens) as usize;
        let mut cached = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((cached_id, cached_mode, theme)) = cached.as_ref()
            && *cached_id == id
            && *cached_mode == mode
        {
            return theme.clone();
        }
        // The look's files are compiled in and bind for every catalog the tests cover; should a catalog
        // still fail to bind, the SDK alone keeps the control drawable.
        let theme = self::theme(look).unwrap_or_else(|_| xgpui_luma::style::default_theme());
        *cached = Some((id, mode, theme.clone()));
        theme
    }
}

/// Native Shadcn tokens: `css.<name>` (the CSS custom property `--<name>`, `_` for `-`) and look
/// functions over the Shadcn resolver. Holds both modes; the bound mode picks one.
pub struct ShadcnTokens {
    light: Arc<ShadcnModeTokens>,
    dark: Arc<ShadcnModeTokens>,
}

impl ShadcnTokens {
    pub fn new(look: &ShadcnLook) -> Self {
        Self { light: look.mode_tokens_for(ThemeMode::Light), dark: look.mode_tokens_for(ThemeMode::Dark) }
    }

    fn mode(&self, cx: &TokenCx<'_>) -> (&ShadcnModeTokens, ThemeMode) {
        if cx.mode() == "dark" {
            (&self.dark, ThemeMode::Dark)
        } else {
            (&self.light, ThemeMode::Light)
        }
    }

    fn resolver<'a>(&'a self, cx: &TokenCx<'_>) -> LookResolver<'a> {
        let (tokens, mode) = self.mode(cx);
        LookResolver::new(&tokens.catalog, mode, "style").with_stylesheet(tokens.stylesheet())
    }
}

const ROOTS: &[&str] = &["css"];

fn color(hsla: Hsla) -> Value {
    let rgb = hsla.to_rgb();
    Value::Color(Color::srgb(rgb.r, rgb.g, rgb.b, rgb.a))
}

fn shadow(shadow: &gpui::BoxShadow) -> ShadowLayer {
    let Value::Color(color) = color(shadow.color) else {
        unreachable!("color() returns a color")
    };
    ShadowLayer {
        x: shadow.offset.x.into(),
        y: shadow.offset.y.into(),
        blur: shadow.blur_radius.into(),
        spread: shadow.spread_radius.into(),
        color,
        inset: shadow.inset,
    }
}

fn symbol<'a>(args: &'a [Value], index: usize, what: &str) -> Result<&'a str, String> {
    match args.get(index) {
        Some(Value::Symbol(name)) => Ok(name),
        other => Err(format!("expected {what} name, got {other:?}")),
    }
}

fn layer_arg(args: &[Value], index: usize) -> Result<InteractionLayer, String> {
    Ok(match symbol(args, index, "an interaction layer")? {
        "default" => InteractionLayer::Default,
        "hover" => InteractionLayer::Hovered,
        "pressed" => InteractionLayer::Pressed,
        "disabled" => InteractionLayer::Disabled,
        other => return Err(format!("unknown layer \"{other}\" (default, hover, pressed, disabled)")),
    })
}

fn style_arg(args: &[Value], index: usize) -> Result<ShadcnButtonStyle, String> {
    Ok(match symbol(args, index, "a style")? {
        "primary" => ShadcnButtonStyle::Primary,
        "secondary" => ShadcnButtonStyle::Secondary,
        "outline" => ShadcnButtonStyle::Outline,
        "ghost" => ShadcnButtonStyle::Ghost,
        "content_only" => ShadcnButtonStyle::ContentOnly,
        other => return Err(format!("unknown style \"{other}\" (primary, secondary, outline, ghost, content_only)")),
    })
}

impl TokenSource for ShadcnTokens {
    fn roots(&self) -> &[&str] {
        ROOTS
    }

    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value> {
        let name = path.strip_prefix("css.")?.replace('_', "-");
        let (tokens, _) = self.mode(cx);
        // Colors go through the resolver, which also derives `-hover` / `-pressed` states the catalog
        // may not list.
        let raw = tokens.catalog.get(&name);
        if let Some(raw) = raw {
            if name.starts_with("shadow") {
                let layers = crate::shadow::parse_shadow_token(&tokens.catalog, &name).ok()?;
                return Some(Value::Shadow(layers.iter().map(shadow).collect()));
            }
            if name.starts_with("font-") {
                return Some(Value::Str(first_font_family(raw).into()));
            }
        }
        if let Ok(resolved) = self.resolver(cx).resolve_decl(&name) {
            return Some(color(resolved.hsla()));
        }
        parse_length_px(raw?).map(Value::Number)
    }

    fn function(&self, name: &str) -> Option<FunctionSig> {
        let (min_args, max_args) = match name {
            "outline_layer" | "action_foreground" => (1, 1),
            "action_layer" => (2, 2),
            _ => return None,
        };
        Some(FunctionSig { min_args, max_args, returns: Kind::Color })
    }

    fn describe(&self, path: &str, cx: &TokenCx<'_>) -> Option<String> {
        let name = path.strip_prefix("css.")?.replace('_', "-");
        let raw = self.mode(cx).0.catalog.get(&name)?;
        Some(format!("--{name}: {raw} ({})", cx.mode()))
    }

    fn call(&self, name: &str, args: &[Value], cx: &TokenCx<'_>) -> Result<Value, String> {
        let resolver = self.resolver(cx);
        let resolved = match name {
            "outline_layer" => resolver.resolve_outline_layer_decl(layer_arg(args, 0)?),
            "action_layer" => resolver.resolve_action_layer_decl(style_arg(args, 0)?, layer_arg(args, 1)?),
            "action_foreground" => resolver.resolve_action_foreground_decl(style_arg(args, 0)?),
            _ => return Err(format!("unknown function \"{name}\"")),
        };
        resolved.map(|c| color(c.hsla())).map_err(|e| format!("{name}: {e}"))
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod native_button_parity;
