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
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeMode};

use crate::catalog::{CssTokenCatalog, CssTokenMap, parse_css_catalog};
use crate::controls::ShadcnButtonStyle;
use crate::controls::{button_look, selection_panel::selection_panel_look, selector_items_panel_look, templates};
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{
    StylesheetConfig, find_typography_scale_rule, find_typography_semantic_rule, resolve_typography_rule,
};

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
}

struct ShadcnLookSnapshot {
    catalog: Arc<CssTokenCatalog>,
    light: Arc<ShadcnModeTokens>,
    dark: Arc<ShadcnModeTokens>,
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
                stylesheet: Arc::clone(&self.stylesheet),
            });
        }

        let light_catalog = apply_color_overrides_to_catalog(self.light.catalog.clone(), light_overrides)?;
        let dark_catalog = apply_color_overrides_to_catalog(self.dark.catalog.clone(), dark_overrides)?;

        Ok(Self {
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
                stylesheet: Arc::clone(&self.stylesheet),
            });
        }

        let light_catalog = apply_string_overrides_to_catalog(self.light.catalog.clone(), overrides);
        let dark_catalog = apply_string_overrides_to_catalog(self.dark.catalog.clone(), overrides);

        Ok(Self {
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
            }),
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
            }),
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
            .stylesheet(&self.mode_tokens().stylesheet().common)
            .motion_defaults(self.motion_defaults().slide_panel)
    }

    pub fn mode(&self) -> ThemeMode {
        u8_to_mode(self.state.mode.load(Ordering::Relaxed))
    }

    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    pub fn mode_tokens(&self) -> Arc<ShadcnModeTokens> {
        let snapshot = self.snapshot();
        match self.mode() {
            ThemeMode::Light => Arc::clone(&snapshot.light),
            ThemeMode::Dark => Arc::clone(&snapshot.dark),
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
        let base_radius = self.parse_pixel_token("radius").unwrap_or(8.0);
        match role {
            ShadcnRadius::None => 0.0,
            ShadcnRadius::Sm => (base_radius - 4.0).max(0.0),
            ShadcnRadius::Md => (base_radius - 2.0).max(0.0),
            ShadcnRadius::Lg => base_radius,
            ShadcnRadius::Xl => base_radius + 4.0,
        }
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

    pub fn typography_scale(&self, size: ShadcnTextSize) -> LumaTextStyle {
        let tokens = self.mode_tokens();
        find_typography_scale_rule(self.stylesheet().as_ref(), size)
            .map(resolve_typography_rule)
            .unwrap_or_else(|| tokens.typography.text.scale(size.into()))
    }

    pub fn typography_role(&self, role: ShadcnTextRole) -> LumaTextStyle {
        let tokens = self.mode_tokens();
        find_typography_semantic_rule(self.stylesheet().as_ref(), role)
            .map(resolve_typography_rule)
            .unwrap_or_else(|| tokens.typography.text.role(role.into()))
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
            }),
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

    pub fn resolve_primary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        let tokens = self.mode_tokens();
        button_look(tokens.as_ref(), self.mode(), ShadcnButtonStyle::Primary, role, size, state)
    }

    pub fn resolve_secondary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        let tokens = self.mode_tokens();
        button_look(tokens.as_ref(), self.mode(), ShadcnButtonStyle::Secondary, role, size, state)
    }

    pub fn resolve_outline_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        let tokens = self.mode_tokens();
        button_look(tokens.as_ref(), self.mode(), ShadcnButtonStyle::Outline, role, size, state)
    }

    pub fn resolve_ghost_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        let tokens = self.mode_tokens();
        button_look(tokens.as_ref(), self.mode(), ShadcnButtonStyle::Ghost, role, size, state)
    }

    pub fn resolve_content_only_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        let tokens = self.mode_tokens();
        button_look(tokens.as_ref(), self.mode(), ShadcnButtonStyle::ContentOnly, role, size, state)
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

    pub fn selector_items_panel_look(
        &self,
        size: ShadcnSize,
    ) -> xgpui_luma::controls::selector_list::SelectorItemsPanelLook {
        let tokens = self.mode_tokens();
        selector_items_panel_look(tokens.as_ref(), self.mode(), size.control_size())
    }

    pub fn selection_panel_look(&self, size: ShadcnSize) -> xgpui_luma::controls::selection_panel::SelectionPanelLook {
        let tokens = self.mode_tokens();
        selection_panel_look(tokens.as_ref(), self.mode(), size.control_size())
    }

    pub(crate) fn selection_panel_look_provider(
        &self,
    ) -> xgpui_luma::controls::selection_panel::SelectionPanelLookProvider {
        templates::selection_panel_look_provider(self.clone())
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
        let stylesheet = self.stylesheet();
        crate::stylesheet::resolve_sidebar_metrics(
            crate::stylesheet::find_sidebar_metrics(stylesheet.as_ref()),
            &self.mode_tokens().metrics,
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

    pub fn button_family_theme(&self) -> Arc<dyn xgpui_luma::controls::button_family::ButtonFamilyTheme> {
        templates::button_family_theme(self.clone())
    }

    pub fn checkbox_theme(&self) -> Arc<dyn xgpui_luma::controls::checkbox::CheckboxTheme> {
        templates::checkbox_theme(self.clone())
    }

    pub fn switch_theme(&self) -> Arc<dyn xgpui_luma::controls::switch::SwitchTheme> {
        templates::switch_theme(self.clone())
    }

    pub fn radio_button_theme(&self) -> Arc<dyn xgpui_luma::controls::radio_button::RadioButtonTheme> {
        templates::radio_button_theme(self.clone())
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
}
