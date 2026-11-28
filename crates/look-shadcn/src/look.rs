#![allow(deprecated)] // Compatibility paths and parity tests still read the deprecated Looks (#433).
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{App, BoxShadow, Global, Hsla, SharedString};
use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use xgpui_luma::infra::lock;
use xgpui_luma::color::ColorValue;
use xgpui_luma::theme::pack::LumaChrome;
use xgpui_luma::theme::{InteractionLayer, InteractionState, LumaTextStyle, ThemeMode};

use crate::catalog::{CssTokenCatalog, CssTokenMap, parse_css_catalog};
use crate::controls::ShadcnButtonStyle;
use crate::controls::{selection_panel::selection_panel_look, selector_items_panel_look, templates};
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{StylesheetConfig};

use crate::shadow::parse_shadow_token;
use crate::size::ShadcnSize;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnTextRole, ShadcnTextSize, ShadcnToken};

const GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
];

/// Runtime shadcn look resolver backed by parsed CSS token catalogs.
#[derive(Clone)]
pub struct ShadcnLook {
    state: Arc<ShadcnLookState>,
    /// Size selected for themes and templates created from this handle.
    size: crate::ShadcnSize,
}

impl Global for ShadcnLook {}

/// `.look(&…)` if set, else ambient Global, else [`ShadcnLook::built_in()`]. Never panics.
pub(crate) fn resolve_look(explicit: Option<&ShadcnLook>, ambient: Option<&ShadcnLook>) -> ShadcnLook {
    if let Some(look) = explicit {
        return look.clone();
    }
    if let Some(look) = ambient {
        return look.clone();
    }
    ShadcnLook::built_in()
}

pub(crate) fn resolve_look_from(explicit: Option<&ShadcnLook>, cx: &App) -> ShadcnLook {
    resolve_look(explicit, cx.try_global::<ShadcnLook>())
}

struct ShadcnLookState {
    motion: RwLock<xgpui_luma::motion::MotionOverrides<xgpui_luma::theme::MotionDefaults>>,
    snapshot: RwLock<Arc<ShadcnLookSnapshot>>,
    mode: AtomicU8,
    /// A style mode other than the scheme's own (e.g. `hi_contrast`), set by
    /// [`ShadcnLook::set_style_mode`].
    style_mode: RwLock<Option<String>>,
    /// The `.lss` theme last bound, keyed by its mode tokens and mode (see [`ShadcnLook::style_theme`]).
    style_theme: crate::style::ThemeCache,
}

struct ShadcnLookSnapshot {
    catalog: Arc<CssTokenCatalog>,
    light: Arc<ShadcnModeTokens>,
    dark: Arc<ShadcnModeTokens>,
    /// Tokens for the modes the CSS adds with class blocks (`hi_contrast`, …), by name.
    modes: Vec<(String, Arc<ShadcnModeTokens>)>,
    stylesheet: Arc<StylesheetConfig>,
}

impl ShadcnLookSnapshot {
    fn new(catalog: CssTokenCatalog, stylesheet: StylesheetConfig) -> anyhow::Result<Self> {
        let stylesheet = Arc::new(stylesheet);
        let light_catalog = catalog.light_map();
        let dark_catalog = catalog.dark_map();
        Ok(Self {
            light: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                light_catalog,
                ThemeMode::Light,
                Arc::clone(&stylesheet),
            )?),
            dark: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                dark_catalog,
                ThemeMode::Dark,
                Arc::clone(&stylesheet),
            )?),
            modes: build_css_modes(&catalog, &stylesheet, |map, _| Ok(map))?,
            catalog: Arc::new(catalog),
            stylesheet,
        })
    }

    fn with_color_overrides(&self, overrides: &HashMap<String, ColorValue>) -> anyhow::Result<Self> {
        self.with_mode_color_overrides(overrides, overrides)
    }

    fn with_mode_color_overrides(
        &self,
        light_overrides: &HashMap<String, ColorValue>,
        dark_overrides: &HashMap<String, ColorValue>,
    ) -> anyhow::Result<Self> {
        if light_overrides.is_empty() && dark_overrides.is_empty() {
            return Ok(Self {
                catalog: Arc::clone(&self.catalog),
                light: Arc::clone(&self.light),
                dark: Arc::clone(&self.dark),
                modes: self.modes.clone(),
                stylesheet: Arc::clone(&self.stylesheet),
            });
        }

        let light_catalog = apply_color_overrides_to_catalog(self.light.catalog.clone(), light_overrides)?;
        let dark_catalog = apply_color_overrides_to_catalog(self.dark.catalog.clone(), dark_overrides)?;
        let modes = build_css_modes(&self.catalog, &self.stylesheet, |map, base| {
            apply_color_overrides_to_catalog(
                map,
                if base == ThemeMode::Light {
                    light_overrides
                } else {
                    dark_overrides
                },
            )
        })?;

        Ok(Self {
            modes,
            catalog: Arc::clone(&self.catalog),
            light: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                light_catalog,
                ThemeMode::Light,
                Arc::clone(&self.stylesheet),
            )?),
            dark: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                dark_catalog,
                ThemeMode::Dark,
                Arc::clone(&self.stylesheet),
            )?),
            stylesheet: Arc::clone(&self.stylesheet),
        })
    }

    fn with_token_overrides(&self, overrides: &HashMap<String, String>) -> anyhow::Result<Self> {
        if overrides.is_empty() {
            return Ok(Self {
                catalog: Arc::clone(&self.catalog),
                light: Arc::clone(&self.light),
                dark: Arc::clone(&self.dark),
                modes: self.modes.clone(),
                stylesheet: Arc::clone(&self.stylesheet),
            });
        }

        let light_catalog = apply_string_overrides_to_catalog(self.light.catalog.clone(), overrides);
        let dark_catalog = apply_string_overrides_to_catalog(self.dark.catalog.clone(), overrides);
        let modes = build_css_modes(&self.catalog, &self.stylesheet, |map, _| {
            Ok(apply_string_overrides_to_catalog(map, overrides))
        })?;

        Ok(Self {
            modes,
            catalog: Arc::clone(&self.catalog),
            light: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                light_catalog,
                ThemeMode::Light,
                Arc::clone(&self.stylesheet),
            )?),
            dark: Arc::new(ShadcnModeTokens::from_catalog_with_stylesheet(
                dark_catalog,
                ThemeMode::Dark,
                Arc::clone(&self.stylesheet),
            )?),
            stylesheet: Arc::clone(&self.stylesheet),
        })
    }
}

impl ShadcnLook {
    /// A handle to the same look whose themes and templates use `size`.
    /// The look resolves the size to pixel metrics; SDK controls only see pixels.
    pub fn with_size(&self, size: crate::ShadcnSize) -> Self {
        Self { state: self.state.clone(), size }
    }

    /// Size used by themes and templates created from this handle.
    pub fn size(&self) -> crate::ShadcnSize {
        self.size
    }

    /// Bundled fallback theme used when no look is bound and no Global is set.
    pub fn built_in() -> Self {
        static BUILT_IN: std::sync::OnceLock<ShadcnLook> = std::sync::OnceLock::new();
        BUILT_IN
            .get_or_init(|| Self::from_css_str(crate::FALLBACK_CSS).expect("bundled built-in CSS should parse"))
            .clone()
    }

    /// Loads shadcn palette tokens from tweakcn-style CSS (`:root` / `.dark` custom properties).
    ///
    /// Creates an independently mutable look. For file-backed themes, the application
    /// owns file access and must enforce any required path and input-size restrictions
    /// before calling this parser. This method does not impose an input-size limit.
    pub fn from_css_str(source: &str) -> anyhow::Result<Self> {
        let catalog = parse_css_catalog(source)?;
        Ok(Self {
            state: Arc::new(ShadcnLookState {
                motion: RwLock::new(Default::default()),
                snapshot: RwLock::new(Arc::new(ShadcnLookSnapshot::new(
                    catalog,
                    crate::stylesheet::embedded_stylesheet().clone(),
                )?)),
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
                style_mode: RwLock::new(None),
                style_theme: Default::default(),
            }),
            size: crate::ShadcnSize::DEFAULT,
        })
    }

    /// Loads CSS palette tokens with an already parsed stylesheet.
    ///
    /// File access and input-size restrictions for CSS and TOML are application-owned;
    /// parse the stylesheet text with [`StylesheetConfig::parse`].
    pub fn from_css_str_with_stylesheet(source: &str, stylesheet: StylesheetConfig) -> anyhow::Result<Self> {
        let catalog = parse_css_catalog(source)?;
        Ok(Self {
            state: Arc::new(ShadcnLookState {
                motion: RwLock::new(Default::default()),
                snapshot: RwLock::new(Arc::new(ShadcnLookSnapshot::new(catalog, stylesheet)?)),
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
                style_mode: RwLock::new(None),
                style_theme: Default::default(),
            }),
            size: crate::ShadcnSize::DEFAULT,
        })
    }

    fn snapshot(&self) -> Arc<ShadcnLookSnapshot> {
        lock::read(&self.state.snapshot).clone()
    }

    pub fn stylesheet(&self) -> Arc<StylesheetConfig> {
        Arc::clone(&self.snapshot().stylesheet)
    }

    /// Common SDK configuration; tabs builders snapshot it when spawning.
    pub fn common_stylesheet(&self) -> xgpui_luma::theme::stylesheet::CommonStylesheet {
        self.stylesheet().common.clone()
    }

    /// Edit SDK-defined motion defaults for future controls. Clones share edits;
    /// independent copies retain their own edits. Existing controls keep snapshots.
    pub fn motion(&self, configure: impl Fn(&mut xgpui_luma::theme::MotionDefaults) + Send + Sync + 'static) {
        lock::write(&self.state.motion).push(configure);
    }

    /// Resolve stylesheet defaults followed by programmatic look edits.
    pub fn motion_defaults(&self) -> xgpui_luma::theme::MotionDefaults {
        let mut defaults = self.common_stylesheet().motion_defaults();
        let edits = lock::read(&self.state.motion).clone();
        edits.apply(&mut defaults);
        defaults
    }

    pub fn tabs_overlay_state(&self) -> xgpui_luma::controls::tabs::TabsOverlayState {
        xgpui_luma::controls::tabs::TabsOverlayState::new().motion_defaults(self.motion_defaults().tabs_overlay)
    }

    pub fn scroll_container(
        &self,
        id: impl Into<SharedString>,
        cx: &mut impl gpui::AppContext,
    ) -> xgpui_luma::controls::scroll_container::ScrollContainer {
        xgpui_luma::controls::scroll_container::ScrollContainer::new(id, self.scrollbar_template(), cx)
            .motion_defaults(self.motion_defaults().scroll_container)
    }

    pub fn slide_panel_state(
        &self,
        anchor: xgpui_luma::controls::slide_panel::SlidePanelTopAnchor,
        size: xgpui_luma::controls::slide_panel::SlidePanelSizeConfig,
    ) -> xgpui_luma::controls::slide_panel::SlidePanelState {
        xgpui_luma::controls::slide_panel::SlidePanelState::new(anchor, size)
            .with_geometry(xgpui_luma::controls::slide_panel::SlidePanelGeometry::from_binding(
                &self.style_binding("slide_panel"),
            ))
            .handle_metrics(self.resizable_panels_theme().resize_handle_metrics())
            .motion_defaults(self.motion_defaults().slide_panel)
    }

    pub fn mode(&self) -> ThemeMode {
        u8_to_mode(self.state.mode.load(Ordering::Relaxed))
    }

    /// Change the active scheme (light or dark); the style mode becomes the one of the same name.
    pub fn set_mode(&self, mode: ThemeMode) {
        if let Ok(mut style_mode) = self.state.style_mode.write() {
            *style_mode = None;
        }
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    /// The style mode the look's `.lss` theme binds: one set with [`ShadcnLook::set_style_mode`], else
    /// the scheme's own (`"light"` or `"dark"`).
    pub fn style_mode(&self) -> String {
        // A named mode the current stylesheet no longer declares (the CSS changed) falls back to the scheme.
        let named = self.state.style_mode.read().ok().and_then(|m| m.clone());
        named
            .filter(|m| self.style_stylesheet().config().modes().any(|declared| declared == m))
            .unwrap_or_else(|| xgpui_luma::style::scheme_name(self.mode()).to_string())
    }

    /// The modes this look's CSS adds with class blocks, with their bases (see [`crate::CssMode`]).
    pub fn css_modes(&self) -> Vec<(String, ThemeMode)> {
        self.snapshot().catalog.modes.iter().map(|m| (m.name.clone(), m.base)).collect()
    }

    /// The Shadcn stylesheet with this look's CSS modes declared (the shared one when it adds none).
    pub fn style_stylesheet(&self) -> xgpui_luma::style::lss::Stylesheet {
        crate::style::stylesheet_with_modes(&self.css_modes())
    }

    /// Choose a declared style mode by name: `light`, `dark`, or a mode the CSS adds (`.hi-contrast`
    /// is `hi_contrast`, based on light). The scheme follows the mode's base, so the CSS values and window
    /// appearance match. Errors for a mode the look's stylesheet doesn't declare, or one that comes down
    /// to neither light nor dark.
    pub fn set_style_mode(&self, mode: &str) -> Result<(), String> {
        let sheet = self.style_stylesheet();
        let config = sheet.config();
        let scheme = xgpui_luma::style::mode_scheme(&config, mode).ok_or_else(|| {
            let modes: Vec<&str> = config.modes().collect();
            format!("unknown style mode \"{mode}\" (modes: {})", modes.join(", "))
        })?;
        self.set_mode(scheme);
        if mode != xgpui_luma::style::scheme_name(scheme)
            && let Ok(mut style_mode) = self.state.style_mode.write()
        {
            *style_mode = Some(mode.to_string());
        }
        Ok(())
    }

    /// The tokens for the current style mode: a CSS mode's own, else the scheme's.
    pub fn mode_tokens(&self) -> Arc<ShadcnModeTokens> {
        let snapshot = self.snapshot();
        let mode = self.style_mode();
        if let Some((_, tokens)) = snapshot.modes.iter().find(|(name, _)| *name == mode) {
            return Arc::clone(tokens);
        }
        match self.mode() {
            ThemeMode::Light => Arc::clone(&snapshot.light),
            ThemeMode::Dark => Arc::clone(&snapshot.dark),
        }
    }

    /// Tokens for each mode the CSS adds, by name (style token sources hold them all).
    pub(crate) fn css_mode_tokens(&self) -> Vec<(String, Arc<ShadcnModeTokens>)> {
        self.snapshot().modes.clone()
    }

    /// Mode tokens for `mode`, whatever the current mode is (style token sources hold both).
    pub(crate) fn mode_tokens_for(&self, mode: ThemeMode) -> Arc<ShadcnModeTokens> {
        let snapshot = self.snapshot();
        match mode {
            ThemeMode::Light => Arc::clone(&snapshot.light),
            ThemeMode::Dark => Arc::clone(&snapshot.dark),
        }
    }

    /// The look's catalog bound to the Shadcn `.lss` files in the current mode. Cached: rebinds only
    /// after the mode or the catalog changed.
    pub fn style_theme(&self) -> xgpui_luma::style::lss::Theme {
        self.state.style_theme.get(self)
    }

    /// A live binding of `control` to the look's `.lss` at the look's size.
    pub(crate) fn style_binding(&self, control: &'static str) -> xgpui_luma::style::StyleBinding {
        let look = self.clone();
        let binding = xgpui_luma::style::StyleBinding::new(
            control,
            xgpui_luma::style::ThemeSource::Live(Arc::new(move || look.style_theme())),
        );
        let size = self.size();
        if size != crate::ShadcnSize::Md {
            binding.attr("size", size.key())
        } else {
            binding
        }
    }

    pub fn catalog(&self) -> Arc<CssTokenCatalog> {
        Arc::clone(&self.snapshot().catalog)
    }

    /// `true` when the look was loaded from tweakcn/shadcn CSS (non-empty catalog).
    pub fn has_css_catalog(&self) -> bool {
        let snapshot = self.snapshot();
        !snapshot.light.catalog.tokens.is_empty() || !snapshot.dark.catalog.tokens.is_empty()
    }

    fn token(&self, name: &str) -> Option<String> {
        let tokens = self.mode_tokens();
        tokens.catalog.get(name).map(ToOwned::to_owned)
    }

    /// Retained source color for the semantic token, with theme fallbacks resolved.
    pub fn source_color(&self, token: ShadcnToken) -> ColorValue {
        self.resolve_source_color_state(token, InteractionLayer::Default)
    }

    /// Source/derived state from the theme snapshot, without gamut mapping.
    pub fn resolve_source_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> ColorValue {
        self.mode_tokens().resolve_source_color_state(token, layer)
    }

    /// Resolves colors directly from the loaded palette.
    pub fn color(&self, token: ShadcnToken) -> Hsla {
        self.mode_tokens().palette.token_color(token)
    }

    /// Dynamically resolves a token color for a specific interaction state.
    ///
    /// Explicit CSS overrides (e.g. `--primary-hover`) take precedence; otherwise
    /// mode-aware Oklch lightness shifts are applied.
    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> Hsla {
        self.mode_tokens().resolve_color_state(token, layer)
    }

    /// Mode-aware lightness shift for outline/surface fills (darker in light, lighter in dark).
    pub fn adjust_surface_color(&self, base: Hsla, layer: InteractionLayer) -> Hsla {
        crate::state_color::algorithmic_state_color(base, layer, self.mode(), false)
    }

    /// Resolves fonts to their loaded families.
    pub fn font(&self, role: ShadcnFont) -> SharedString {
        let key = match role {
            ShadcnFont::Sans => "font-sans",
            ShadcnFont::Serif => "font-serif",
            ShadcnFont::Mono => "font-mono",
        };
        self.token(key).as_deref().map(first_font_family).unwrap_or_else(|| match role {
            ShadcnFont::Sans => SharedString::from("sans-serif"),
            ShadcnFont::Serif => SharedString::from("serif"),
            ShadcnFont::Mono => SharedString::from("monospace"),
        })
    }

    /// Resolves border-radius to raw pixels based on the current theme radius.
    pub fn radius(&self, role: ShadcnRadius) -> f32 {
        self.mode_tokens().radii.step(role)
    }

    /// Resolves custom shadow structures for GPUI box-shadow arrays.
    pub fn shadow(&self, role: ShadcnShadow) -> Vec<BoxShadow> {
        let shadow_key = match role {
            ShadcnShadow::None => return vec![],
            ShadcnShadow::TwoXs => "shadow-2xs",
            ShadcnShadow::Xs => "shadow-xs",
            ShadcnShadow::Sm => "shadow-sm",
            ShadcnShadow::Default => "shadow",
            ShadcnShadow::Md => "shadow-md",
            ShadcnShadow::Lg => "shadow-lg",
            ShadcnShadow::Xl => "shadow-xl",
            ShadcnShadow::TwoXl => "shadow-2xl",
        };

        self.parse_shadow_token(shadow_key).unwrap_or_default()
    }

    pub fn parse_pixel_token(&self, name: &str) -> Option<f32> {
        self.token(name).as_deref().and_then(parse_length_px)
    }

    /// Retrieve shadow source colors without rendering or gamut mapping.
    pub fn parse_shadow_source_token(&self, token_key: &str) -> anyhow::Result<Vec<crate::ShadowTokenParts>> {
        let tokens = self.mode_tokens();
        crate::shadow::parse_shadow_source_token(&tokens.catalog, token_key)
    }

    pub fn parse_shadow_token(&self, token_key: &str) -> anyhow::Result<Vec<BoxShadow>> {
        let tokens = self.mode_tokens();
        if let Some(preview) = tokens.shadow_preview(token_key) {
            return Ok(preview.to_vec());
        }
        parse_shadow_token(&tokens.catalog, token_key)
    }

    /// The text style for `size`, from the look's `.lss` `text.scale` tokens.
    pub fn typography_scale(&self, size: ShadcnTextSize) -> LumaTextStyle {
        self.text_token(&format!("text.scale.{}", size.token_key()))
            .unwrap_or_else(|| self.mode_tokens().typography.text.label)
    }

    /// The text style for a heading or paragraph `role`, from the look's `.lss` `text.role` tokens.
    pub fn typography_role(&self, role: ShadcnTextRole) -> LumaTextStyle {
        self.text_token(&format!("text.role.{}", role.as_str()))
            .unwrap_or_else(|| self.mode_tokens().typography.text.role(role.into()))
    }

    fn text_token(&self, path: &str) -> Option<LumaTextStyle> {
        match self.style_theme().token_with(path, &[]).ok()? {
            xgpui_luma::style::lss::Value::Text(text) => Some(LumaTextStyle {
                size: text.size,
                line_height: text.line_height,
                weight: gpui::FontWeight(text.weight),
            }),
            _ => None,
        }
    }

    pub fn light_tokens(&self) -> Arc<ShadcnModeTokens> {
        Arc::clone(&self.snapshot().light)
    }

    pub fn dark_tokens(&self) -> Arc<ShadcnModeTokens> {
        Arc::clone(&self.snapshot().dark)
    }

    /// Original arbitrary CSS token color, without a render projection.
    pub fn token_source_color(&self, name: &str) -> anyhow::Result<ColorValue> {
        self.mode_tokens().catalog.source_color(name)
    }

    pub fn token_color(&self, name: &str) -> anyhow::Result<Hsla> {
        let tokens = self.mode_tokens();
        tokens.catalog.color(name)
    }

    pub fn replace_theme(&self, other: &ShadcnLook) {
        *lock::write(&self.state.snapshot) = other.snapshot();
    }

    /// Apply validated source colors, preserving their space and precision.
    pub fn apply_color_overrides(&self, overrides: &HashMap<String, ColorValue>) -> anyhow::Result<()> {
        let snapshot = self.snapshot();
        let next = snapshot.with_color_overrides(overrides)?;
        *lock::write(&self.state.snapshot) = Arc::new(next);
        Ok(())
    }

    /// Apply independently validated source colors to each mode.
    pub fn apply_mode_color_overrides(
        &self,
        light_overrides: &HashMap<String, ColorValue>,
        dark_overrides: &HashMap<String, ColorValue>,
    ) -> anyhow::Result<()> {
        let snapshot = self.snapshot();
        let next = snapshot.with_mode_color_overrides(light_overrides, dark_overrides)?;
        *lock::write(&self.state.snapshot) = Arc::new(next);
        Ok(())
    }

    pub fn apply_token_overrides(&self, overrides: &HashMap<String, String>) -> anyhow::Result<()> {
        let snapshot = self.snapshot();
        let next = snapshot.with_token_overrides(overrides)?;
        *lock::write(&self.state.snapshot) = Arc::new(next);
        Ok(())
    }

    /// Copy with source color overrides in both modes; invalid sources return an error.
    pub fn with_color_overrides(&self, overrides: &HashMap<String, ColorValue>) -> anyhow::Result<Self> {
        let snapshot = self.snapshot();
        let next = snapshot.with_color_overrides(overrides)?;

        Ok(Self {
            state: Arc::new(ShadcnLookState {
                motion: RwLock::new(lock::read(&self.state.motion).clone()),
                snapshot: RwLock::new(Arc::new(next)),
                mode: AtomicU8::new(self.state.mode.load(Ordering::Relaxed)),
                style_mode: RwLock::new(None),
                style_theme: Default::default(),
            }),
            size: crate::ShadcnSize::DEFAULT,
        })
    }

    pub fn chrome(&self) -> LumaChrome {
        let tokens = self.mode_tokens();
        let palette = &tokens.palette;

        LumaChrome {
            app_background: palette.app_background,
            content_background: palette.app_background,
            title_text: palette.app_foreground,
            body_text: palette.body_text,
            muted_text: palette.app_muted_foreground,
            border: palette.border_default,
            panel_background: palette.panel_background,
        }
    }

    /// A button face from `.lss` for `style` and `role` at `size` as a resolved style (#433): the root carries the
    /// face's colors, text, box and shadow. `radius` is an optional corner preset.
    pub fn button_face_style(
        &self,
        style: ShadcnButtonStyle,
        role: xgpui_luma::controls::button_family::ButtonFamilyRole,
        size: ShadcnSize,
        radius: Option<crate::ButtonRadiusPreset>,
        state: xgpui_luma::theme::InteractionState,
    ) -> Arc<xgpui_luma::style::lss::Resolved> {
        crate::controls::button_builder::template_for::<()>(&self.with_size(size), style, radius)
            .resolve_face(role, state, 1.0)
    }

    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn resolve_primary_button(
        &self,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        self.button_face(ShadcnButtonStyle::Primary, role, size, None, state)
    }

    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn resolve_secondary_button(
        &self,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        self.button_face(ShadcnButtonStyle::Secondary, role, size, None, state)
    }

    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn resolve_outline_button(
        &self,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        self.button_face(ShadcnButtonStyle::Outline, role, size, None, state)
    }

    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn resolve_ghost_button(
        &self,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        self.button_face(ShadcnButtonStyle::Ghost, role, size, None, state)
    }

    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn resolve_content_only_button(
        &self,
        role: ButtonFamilyRole,
        size: ShadcnSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        self.button_face(ShadcnButtonStyle::ContentOnly, role, size, None, state)
    }

    pub fn switch_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<xgpui_luma::controls::switch::SwitchData>> {
        templates::switch_template(self.clone(), style)
    }

    pub fn checkbox_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<xgpui_luma::controls::checkbox::CheckboxData>> {
        templates::checkbox_template(self.clone(), style)
    }

    pub fn radio_button_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<xgpui_luma::controls::radio_button::RadioButtonData>>
    {
        templates::radio_button_template(self.clone(), style)
    }

    /// The `.lss` radio template before it is shared, for adding modifiers.
    pub fn styled_radio_button_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> xgpui_luma::controls::radio_button::StyledRadioButtonTemplate {
        templates::styled_radio_button_template(self.clone(), style)
    }

    pub fn button_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<()>> {
        templates::button_template(self.clone(), style)
    }

    pub fn slider_template(&self) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
        templates::slider_template(self.clone())
    }

    pub fn slider_template_with_style(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
        templates::slider_template_with_style(self.clone(), style)
    }

    pub fn slider_angular_template(&self) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
        templates::slider_angular_template(self.clone())
    }

    pub fn slider_circular_ring_template(&self) -> Arc<dyn xgpui_luma::controls::slider::SliderTemplate> {
        templates::slider_circular_ring_template(self.clone())
    }

    pub fn scrollbar_template(&self) -> Arc<dyn xgpui_luma::controls::scrollbar::ScrollbarTemplate> {
        templates::scrollbar_template(self.clone())
    }

    pub fn floating_menu_theme(&self) -> Arc<dyn xgpui_luma::controls::floating_menu::FloatingMenuTheme> {
        templates::floating_menu_theme(self.clone())
    }

    pub fn popup_menu_template(&self) -> Arc<dyn xgpui_luma::controls::popup_menu::PopupMenuTemplate> {
        templates::popup_menu_template(self.clone())
    }

    pub fn split_button_theme(&self) -> Arc<dyn xgpui_luma::controls::split_button::SplitButtonTheme> {
        templates::popup_menu_theme(self.clone())
    }

    pub fn split_button_template(&self) -> Arc<dyn xgpui_luma::controls::split_button::SplitButtonTemplate> {
        templates::split_button_template(self.clone())
    }

    pub fn context_menu_theme(&self) -> Arc<dyn xgpui_luma::controls::context_menu::ContextMenuTheme> {
        templates::context_menu_theme(self.clone())
    }

    pub fn context_menu_template(&self) -> Arc<dyn xgpui_luma::controls::context_menu::ContextMenuTemplate> {
        templates::context_menu_template(self.clone())
    }

    pub fn selector_template<T>(&self) -> Arc<dyn xgpui_luma::controls::selector::SelectorTemplate<T>>
    where
        T: xgpui_luma::controls::selector::SelectorItemLike + 'static,
    {
        templates::selector_template(self.clone())
    }

    pub fn textfield_template(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTemplate> {
        templates::textfield_template(self.clone())
    }

    pub fn textfield_theme(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTheme> {
        templates::textfield_theme(self.clone())
    }

    pub fn input_textfield_theme(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTheme> {
        templates::input_textfield_theme(self.clone())
    }

    pub fn input_textfield_template(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTemplate> {
        templates::input_textfield_template(self.clone())
    }

    pub fn surface_textfield_theme(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTheme> {
        templates::surface_textfield_theme(self.clone())
    }

    pub fn primary_textfield_theme(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTheme> {
        templates::primary_textfield_theme(self.clone())
    }

    pub fn primary_textfield_template(&self) -> Arc<dyn xgpui_luma::controls::textfield::TextFieldTemplate> {
        templates::primary_textfield_template(self.clone())
    }

    pub fn textarea_template(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
        templates::textarea_template(self.clone())
    }

    pub fn textarea_theme(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTheme> {
        templates::textarea_theme(self.clone())
    }

    pub fn surface_textarea_theme(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTheme> {
        templates::surface_textarea_theme(self.clone())
    }

    pub fn primary_textarea_theme(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTheme> {
        templates::primary_textarea_theme(self.clone())
    }

    pub fn input_textarea_theme(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTheme> {
        templates::input_textarea_theme(self.clone())
    }

    pub(crate) fn input_textarea_template(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
        templates::input_textarea_template(self.clone())
    }

    pub fn primary_textarea_template(&self) -> Arc<dyn xgpui_luma::controls::textarea::TextAreaTemplate> {
        templates::primary_textarea_template(self.clone())
    }

    pub fn autocomplete_theme(&self) -> Arc<dyn xgpui_luma::controls::autocomplete::AutocompleteTheme> {
        templates::autocomplete_theme(self.clone())
    }

    /// The selector and combobox dropdown panel's style at `size` (#433).
    pub fn selector_items_panel_style(&self, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
        crate::controls::selector_items_panel::selector_items_panel_style(self.mode_tokens().as_ref(), size)
    }

    /// The selection panel's style at `size` (#433).
    pub fn selection_panel_style(&self, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
        crate::controls::selection_panel::selection_panel_style(self.mode_tokens().as_ref(), size)
    }

    #[deprecated(note = "use `selector_items_panel_style`; the SDK's panel templates draw from the style (#433)")]
    pub fn selector_items_panel_look(
        &self,
        size: ShadcnSize,
    ) -> xgpui_luma::controls::selector_list::SelectorItemsPanelLook {
        let tokens = self.mode_tokens();
        selector_items_panel_look(tokens.as_ref(), self.mode(), size)
    }

    #[deprecated(note = "use `selection_panel_style`; the SDK's template draws from the style (#433)")]
    pub fn selection_panel_look(&self, size: ShadcnSize) -> xgpui_luma::controls::selection_panel::SelectionPanelLook {
        let tokens = self.mode_tokens();
        selection_panel_look(tokens.as_ref(), self.mode(), size)
    }

    pub(crate) fn selection_panel_menu(&self) -> xgpui_luma::controls::popup_menu::MenuSource {
        templates::selection_panel_menu(self.clone())
    }

    pub fn tabs_template(&self) -> Arc<dyn xgpui_luma::controls::tabs::TabsTemplate> {
        templates::tabs_template(self.clone())
    }

    pub fn tabs_theme(&self) -> Arc<dyn xgpui_luma::controls::tabs::TabsTheme> {
        templates::tabs_theme(self.clone())
    }

    pub fn accordion_template(&self) -> Arc<dyn xgpui_luma::controls::accordion::AccordionTemplate> {
        templates::accordion_template(self.clone())
    }

    pub fn accordion_theme(&self) -> Arc<dyn xgpui_luma::controls::accordion::AccordionTheme> {
        templates::accordion_theme(self.clone())
    }

    pub fn tree_view_template<T>(&self) -> Arc<dyn xgpui_luma::controls::tree_view::TreeViewTemplate<T>>
    where
        T: Send + Sync + 'static,
    {
        templates::tree_view_template(self.clone())
    }

    /// Content-free chrome for the SDK's borrowed listbox renderer.
    pub fn listbox_template(&self) -> Arc<dyn xgpui_luma::controls::listbox::ListBoxTemplate> {
        templates::listbox_template(self.clone())
    }

    pub fn tree_view_theme(&self) -> Arc<dyn xgpui_luma::controls::tree_view::TreeViewTheme> {
        templates::tree_view_theme(self.clone())
    }

    pub fn dock_splitter_theme(&self) -> Arc<dyn xgpui_luma::controls::dock_splitter::DockSplitterTheme> {
        templates::dock_splitter_theme(self.clone())
    }

    pub fn resizable_panels_theme(&self) -> Arc<dyn xgpui_luma::controls::resizable_panels::ResizablePanelsTheme> {
        templates::resizable_panels_theme(self.clone())
    }

    pub fn resizable_panels(
        &self,
        id: impl Into<SharedString>,
    ) -> xgpui_luma::controls::resizable_panels::ResizablePanelsBuilder {
        xgpui_luma::controls::resizable_panels::ResizablePanels::new(id)
            .theme(self.resizable_panels_theme())
            .motion_defaults(self.motion_defaults().resizable_panels)
    }

    pub fn split_view_theme(&self) -> Arc<dyn xgpui_luma::controls::split_view::SplitViewTheme> {
        templates::split_view_theme(self.clone())
    }

    pub fn split_view(&self, id: impl Into<SharedString>) -> xgpui_luma::controls::split_view::SplitViewBuilder {
        xgpui_luma::controls::split_view::SplitView::new(id)
            .theme(self.split_view_theme())
            .motion_defaults(self.motion_defaults().split_view)
    }

    pub fn sidebar_panel_template(&self) -> Arc<dyn xgpui_luma::controls::sidebar::SidebarPanelTemplate> {
        templates::sidebar_panel_template(self.clone())
    }

    pub fn sidebar_theme(&self) -> Arc<dyn xgpui_luma::controls::sidebar::SidebarTheme> {
        templates::sidebar_theme(self.clone())
    }

    pub fn sidebar_metric_scale(&self) -> xgpui_luma::controls::sidebar::SidebarMetricScale {
        xgpui_luma::controls::sidebar::SidebarMetricScale::from_geometry(
            &xgpui_luma::theme::stylesheet::SidebarGeometry::from_binding(&self.style_binding("sidebar")),
        )
    }

    pub fn control_group_template<T>(&self) -> xgpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: xgpui_luma::controls::control_group::ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        templates::control_group_template(self.clone())
    }

    pub fn control_group_theme(&self) -> Arc<dyn xgpui_luma::controls::control_group::ControlGroupTheme> {
        templates::control_group_theme(self.clone())
    }

    pub fn toolbar_template(&self) -> Arc<dyn xgpui_luma::controls::toolbar::ToolbarTemplate> {
        templates::toolbar_template(self.clone())
    }

    pub fn toolbar_theme(&self) -> Arc<dyn xgpui_luma::controls::toolbar::ToolbarTheme> {
        templates::toolbar_theme(self.clone())
    }

    pub fn table_template(&self) -> Arc<dyn xgpui_luma::controls::table::TableTemplate> {
        templates::table_template(self.clone())
    }

    pub fn table_theme(&self) -> Arc<dyn xgpui_luma::controls::table::TableTheme> {
        templates::table_theme(self.clone())
    }

    pub fn pager_template(&self) -> Arc<dyn xgpui_luma::controls::pager::PagerTemplate> {
        templates::pager_template(self.clone())
    }

    pub fn pager_theme(&self) -> Arc<dyn xgpui_luma::controls::pager::PagerTheme> {
        templates::pager_theme(self.clone())
    }

    pub fn radio_group_template<T>(
        &self,
        style: ShadcnButtonStyle,
    ) -> xgpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: xgpui_luma::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            self.clone(),
            style,
            xgpui_luma::controls::radio_group::RadioGroupLayout::Vertical,
        )
    }

    pub fn radio_group_horizontal_template<T>(
        &self,
        style: ShadcnButtonStyle,
    ) -> xgpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: xgpui_luma::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            self.clone(),
            style,
            xgpui_luma::controls::radio_group::RadioGroupLayout::Horizontal,
        )
    }

    pub fn progress_template(&self) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate> {
        templates::progress_template(self.clone())
    }

    pub fn linear_progress_template(&self) -> Arc<dyn xgpui_luma::controls::progress::ProgressTemplate> {
        templates::linear_progress_template(self.clone())
    }

    pub fn progress_theme(&self) -> Arc<dyn xgpui_luma::controls::progress::ProgressTheme> {
        templates::progress_theme(self.clone())
    }

    pub fn stepper_template(&self) -> Arc<dyn xgpui_luma::controls::stepper::StepperTemplate> {
        templates::stepper_template(self.clone())
    }

    pub fn stepper_theme(&self) -> Arc<dyn xgpui_luma::controls::stepper::StepperTheme> {
        templates::stepper_theme(self.clone())
    }

    pub fn overlay_window_template(&self) -> Arc<dyn xgpui_luma::controls::overlay_window::OverlayWindowTemplate> {
        templates::overlay_window_template(self.clone())
    }

    pub fn overlay_window_theme(&self) -> Arc<dyn xgpui_luma::controls::overlay_window::OverlayWindowTheme> {
        templates::overlay_window_theme(self.clone())
    }

    pub fn overlay_window(
        &self,
        id: impl Into<SharedString>,
    ) -> xgpui_luma::controls::overlay_window::OverlayWindowBuilder {
        xgpui_luma::controls::overlay_window::overlay_window(id).template(self.overlay_window_template())
    }

    /// A button face from `.lss` for `style` and `role` at `size`, with an optional corner preset (the
    /// `radius` attribute; none keeps the size's corner). A toggle role draws the toggle.
    #[deprecated(note = "use `button_face_style` and read the resolved style (#433)")]
    pub fn button_face(
        &self,
        style: ShadcnButtonStyle,
        role: xgpui_luma::controls::button_family::ButtonFamilyRole,
        size: ShadcnSize,
        radius: Option<crate::ButtonRadiusPreset>,
        state: xgpui_luma::theme::InteractionState,
    ) -> xgpui_luma::controls::button_family::ButtonFamilyLook {
        crate::controls::button_builder::template_for::<()>(&self.with_size(size), style, radius).look(role, state, 1.0)
    }

    /// The `.lss` button-family template for `style` at this look's size, for any button data
    /// (buttons, toggles), with an optional radius preset (`radius none|small|…` in `button.lss`).
    /// Draws what [`Self::button_face`] resolves.
    pub fn button_family_template<D: xgpui_luma::controls::button::ButtonData>(
        &self,
        style: ShadcnButtonStyle,
        radius: Option<crate::ButtonRadiusPreset>,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<D>> {
        Arc::new(crate::controls::button_builder::template_for::<D>(self, style, radius))
    }

    /// The `.lss` button-family template for `style` with its shadow replaced by `shadows` (#433): for a design
    /// surface that previews a shadow token on a button's face.
    pub fn button_family_template_with_shadow<D: xgpui_luma::controls::button::ButtonData>(
        &self,
        style: ShadcnButtonStyle,
        shadows: &[gpui::BoxShadow],
    ) -> xgpui_luma::controls::button::StyledButtonTemplate<D> {
        use xgpui_luma::style::lss::{Overrides, PartId, Prop, ShadowLayer};
        let layers: Arc<[ShadowLayer]> = shadows
            .iter()
            .map(|shadow| ShadowLayer {
                x: f32::from(shadow.offset.x),
                y: f32::from(shadow.offset.y),
                blur: f32::from(shadow.blur_radius),
                spread: f32::from(shadow.spread_radius),
                color: xgpui_luma::style::lss_color(shadow.color),
                inset: shadow.inset,
            })
            .collect();
        crate::controls::button_builder::template_for::<D>(self, style, None).with_overrides(&Overrides::new().set(
            PartId::ROOT,
            Prop::<Arc<[ShadowLayer]>>::new("shadow"),
            layers,
        ))
    }

    /// An icon-only toggle's template (#433): the toggle's colors, shadow and corner on the icon button's square box
    /// at `size`, with an optional corner preset.
    pub fn icon_toggle_template<D: xgpui_luma::controls::button::ButtonData>(
        &self,
        style: ShadcnButtonStyle,
        size: ShadcnSize,
        radius: Option<crate::ButtonRadiusPreset>,
    ) -> xgpui_luma::controls::button::StyledButtonTemplate<D> {
        use xgpui_luma::controls::button_family::ButtonFamilyRole;
        use xgpui_luma::style::PartRead;
        use xgpui_luma::style::lss::{Overrides, PartId, Prop};
        let icon = self.button_face_style(style, ButtonFamilyRole::Icon, size, radius, Default::default());
        let icon = icon.root();
        let root = PartId::ROOT;
        let mut overrides = Overrides::new()
            .set(root, Prop::<f32>::new("height"), icon.number("height"))
            .set(root, Prop::<f32>::new("padding_x"), 0.0)
            .set(root, Prop::<f32>::new("padding_y"), 0.0)
            .set(root, Prop::<f32>::new("text_icon_size"), icon.number("icon_size"));
        // A preset sizes the corner to the square box; otherwise the toggle keeps its own corner.
        if radius.is_some() {
            overrides = overrides.set(root, Prop::<f32>::new("radius"), icon.number("icon_radius"));
        }
        crate::controls::button_builder::template_for::<D>(&self.with_size(size), style, radius)
            .with_overrides(&overrides)
    }

    /// An icon-only toggle from `.lss`: the toggle's colors, shadow and corner on the icon button's square box.
    #[deprecated(note = "use `icon_toggle_template` (#433)")]
    pub fn icon_toggle_face(
        &self,
        style: ShadcnButtonStyle,
        selected: bool,
        size: ShadcnSize,
        radius: Option<crate::ButtonRadiusPreset>,
        state: xgpui_luma::theme::InteractionState,
    ) -> xgpui_luma::controls::button_family::ButtonFamilyLook {
        use xgpui_luma::controls::button_family::ButtonFamilyRole;
        let mut face = self.button_face(style, ButtonFamilyRole::Toggle { selected }, size, radius, state);
        let icon = self.button_face(style, ButtonFamilyRole::Icon, size, radius, state);
        face.height = icon.height;
        face.padding_x = 0.0;
        face.padding_y = 0.0;
        face.icon_size = icon.icon_size;
        // A preset sizes the corner to the square box; otherwise the toggle keeps its own corner.
        if radius.is_some() {
            face.radius = icon.radius;
        }
        face
    }

    pub fn toggle_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<xgpui_luma::controls::toggle::ToggleData>> {
        templates::toggle_template(self.clone(), style)
    }

    /// Toggle-looking chrome for control-group / toolbar items that use a `bool` selected payload.
    /// Prefer [`Self::toggle_template`] for the animated [`xgpui_luma::controls::toggle::Toggle`] control.
    pub fn toggle_item_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<bool>> {
        templates::toggle_item_template(self.clone(), style)
    }

    pub fn animated_toggle_item_template(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::button::ButtonTemplate<xgpui_luma::controls::toggle::ToggleData>> {
        templates::animated_toggle_item_template(self.clone(), style)
    }

    pub fn slider_theme(&self) -> Arc<dyn xgpui_luma::controls::slider::SliderTheme> {
        templates::slider_theme(self.clone())
    }

    pub fn slider_theme_with_style(
        &self,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn xgpui_luma::controls::slider::SliderTheme> {
        templates::slider_theme_with_style(self.clone(), style)
    }

    pub fn scrollbar_theme(&self) -> Arc<dyn xgpui_luma::controls::scrollbar::ScrollbarTheme> {
        templates::scrollbar_theme(self.clone())
    }

    pub fn selector_theme(&self) -> Arc<dyn xgpui_luma::controls::selector::SelectorTheme> {
        templates::selector_theme(self.clone())
    }

    pub fn popup_menu_theme(&self) -> Arc<dyn xgpui_luma::controls::popup_menu::PopupMenuTheme> {
        templates::popup_menu_theme(self.clone())
    }
}

fn apply_color_overrides_to_catalog(
    mut catalog: CssTokenMap,
    overrides: &HashMap<String, ColorValue>,
) -> anyhow::Result<CssTokenMap> {
    for (token, color) in overrides {
        let key = token.strip_prefix("--").unwrap_or(token.as_str()).to_string();
        catalog.tokens.insert(key, color.to_css()?);
    }
    Ok(catalog)
}

/// Mode tokens for each class-block mode in `catalog`: its base's values with the block's over them,
/// then `adjust` (the look's overrides for that base).
fn build_css_modes(
    catalog: &CssTokenCatalog,
    stylesheet: &Arc<StylesheetConfig>,
    adjust: impl Fn(CssTokenMap, ThemeMode) -> anyhow::Result<CssTokenMap>,
) -> anyhow::Result<Vec<(String, Arc<ShadcnModeTokens>)>> {
    catalog
        .modes
        .iter()
        .map(|mode| {
            let map = adjust(catalog.mode_map(mode), mode.base)?;
            let tokens = ShadcnModeTokens::from_catalog_with_stylesheet(map, mode.base, Arc::clone(stylesheet))?;
            Ok((mode.name.clone(), Arc::new(tokens)))
        })
        .collect()
}

fn apply_string_overrides_to_catalog(mut catalog: CssTokenMap, overrides: &HashMap<String, String>) -> CssTokenMap {
    for (token, value) in overrides {
        let key = token.strip_prefix("--").unwrap_or(token.as_str()).to_string();
        catalog.tokens.insert(key, value.clone());
    }
    catalog
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn u8_to_mode(value: u8) -> ThemeMode {
    match value {
        0 => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

fn normalize_font_family(family: &str) -> SharedString {
    if family.eq_ignore_ascii_case("Rajdhani") {
        return SharedString::from("Rajdhani Variable");
    }
    SharedString::from(family.to_string())
}

fn first_font_family(raw: &str) -> SharedString {
    for part in raw.split(',') {
        let candidate = part.trim().trim_matches(['\'', '"']);
        if candidate.is_empty() {
            continue;
        }
        if GENERIC_FAMILIES.iter().any(|generic| candidate.eq_ignore_ascii_case(generic)) {
            continue;
        }
        return normalize_font_family(candidate);
    }

    normalize_font_family(raw.split(',').next().unwrap_or(raw).trim().trim_matches(['\'', '"']))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_css_class_block_is_a_style_mode_based_on_light() {
        let css = format!("{}\n.hi-contrast {{ --primary: oklch(0.45 0.2 260); }}", crate::FALLBACK_CSS);
        let look = ShadcnLook::from_css_str(&css).expect("parses");
        assert_eq!(look.css_modes(), [("hi_contrast".to_string(), ThemeMode::Light)]);
        assert_eq!(look.style_stylesheet().modes(), ["light", "dark", "hi_contrast"]);

        let light = look.style_theme();
        look.set_style_mode("hi_contrast").expect("the CSS declares it");
        assert_eq!((look.mode(), look.style_mode().as_str()), (ThemeMode::Light, "hi_contrast"));
        let high = look.style_theme();
        assert_eq!((high.mode(), high.scheme()), ("hi_contrast", "light"));

        // The block's value, the rest from `:root`.
        let tokens = look.mode_tokens();
        assert_eq!(tokens.catalog.get("primary"), Some("oklch(0.45 0.2 260)"));
        assert_eq!(tokens.catalog.get("background"), look.mode_tokens_for(ThemeMode::Light).catalog.get("background"));
        let checked = |theme: &xgpui_luma::style::lss::Theme| {
            let key = theme.key_named("checkbox", &[], &["checked"]).expect("key");
            theme.resolve(&key).part(xgpui_luma::style::lss::PartId::new("indicator")).background()
        };
        assert_ne!(checked(&light), checked(&high), "the checked indicator reads the mode's --primary");

        let plain = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("parses");
        assert!(plain.css_modes().is_empty());
        assert_eq!(plain.style_stylesheet().modes(), ["light", "dark"], "no CSS modes: the shared stylesheet");
        assert!(plain.set_style_mode("hi_contrast").is_err());
    }

    #[test]
    fn style_mode_is_the_scheme_unless_set_by_name_and_follows_set_mode() {
        // Its own look: `built_in()` is shared, and other tests read its mode.
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS parses");
        assert_eq!(look.style_mode(), "light");
        look.set_style_mode("dark").expect("declared");
        assert_eq!((look.mode(), look.style_mode().as_str()), (ThemeMode::Dark, "dark"));
        assert_eq!(look.style_theme().mode(), "dark", "the theme binds the style mode");
        assert!(look.set_style_mode("sepia").is_err(), "undeclared");
        look.set_mode(ThemeMode::Light);
        assert_eq!(look.style_mode(), "light");
    }

    use super::*;

    #[test]
    fn fallback_look_resolves_primary_color() {
        let look = crate::test_support::fallback_look();
        let primary = look.color(ShadcnToken::Primary);
        assert!(primary.a > 0.0);
    }

    #[test]
    fn radius_scales_from_base_token() {
        let look = crate::test_support::fallback_look();
        assert!(look.radius(ShadcnRadius::Lg) >= look.radius(ShadcnRadius::Sm));
    }

    #[test]
    fn color_overrides_preserve_source_and_derive_before_mapping() {
        let look = crate::test_support::fallback_look();
        let color = ColorValue::display_p3(1.0, 0.0, 0.0, 0.37);
        let overridden = look.with_color_overrides(&HashMap::from([(String::from("--primary"), color)])).unwrap();
        assert_eq!(overridden.source_color(ShadcnToken::Primary), color);
        assert_eq!(overridden.token_source_color("primary").unwrap(), color);
        let hover = overridden.resolve_source_color_state(ShadcnToken::Primary, InteractionLayer::Hovered);
        assert_eq!(hover, color.adjust_ui_lightness(-0.03).unwrap());
        assert_eq!(overridden.color(ShadcnToken::Primary).a, 0.37);
        assert_ne!(look.source_color(ShadcnToken::Primary), color);
        overridden.set_mode(ThemeMode::Dark);
        assert_eq!(overridden.source_color(ShadcnToken::Primary), color);
        assert_eq!(
            overridden.resolve_source_color_state(ShadcnToken::Primary, InteractionLayer::Hovered),
            color.adjust_ui_lightness(0.04).unwrap()
        );
    }

    #[test]
    fn invalid_color_override_does_not_replace_snapshot() {
        let look = crate::test_support::fallback_look();
        let before = look.source_color(ShadcnToken::Primary);
        let invalid = HashMap::from([("primary".into(), ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0))]);
        assert!(look.apply_color_overrides(&invalid).is_err());
        assert!(look.with_color_overrides(&invalid).is_err());
        assert_eq!(look.source_color(ShadcnToken::Primary), before);
    }

    #[test]
    fn poisoned_snapshot_lock_does_not_abort_reads() {
        let look = crate::test_support::fallback_look();
        let expected = look.color(ShadcnToken::Primary);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = look.state.snapshot.write().unwrap();
            panic!("poison");
        }));
        assert_eq!(look.color(ShadcnToken::Primary), expected);
        assert!(look.has_css_catalog());
    }

    #[test]
    fn drawer_handle_and_scrollbar_minimum_follow_the_look_size() {
        use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
        use xgpui_luma::controls::slide_panel::{SlidePanelSizeConfig, SlidePanelTopAnchor};
        let look = crate::test_support::fallback_look();
        for size in crate::ShadcnSize::ALL {
            let sized = look.with_size(size);
            let drawer =
                sized.slide_panel_state(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 100.0, 500.0));
            assert_eq!(drawer.resize_handle_metrics(), sized.resizable_panels_theme().resize_handle_metrics());
        }
        let sm = look.with_size(crate::ShadcnSize::Sm).scrollbar_template();
        assert_eq!(sm.min_thumb_length(ScrollbarOrientation::Vertical, ScrollbarStyle::Ghost), 24.0);
    }
}
