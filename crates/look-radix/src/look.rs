//! Runtime radix look: mode + dual scale space (color + gray).

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{Background, Global, Hsla, linear_color_stop, linear_gradient};
use xgpui_luma::theme::{FontTokens, LumaTypography, MetricTokens, ThemeMode};
use xgpui_luma::theme::provenance::{ResolvedColor, ResolvedSourceColor};
use xgpui_luma::color::{ColorValue};

use crate::button::ClassicButtonParams;
use crate::palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
use crate::scale::{ModeScales, ScaleFamily, ScalePair, ScaleStep};
use crate::semantic::SemanticRole;
use crate::{CustomColors, generate_colors};

/// Canonical page wash: color step 3 at the top, settling to gray step 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageBackground {
    /// Chromatic scale step 3 (gradient start / top).
    pub color_step: ScaleStep,
    /// Gray scale step 1 (gradient settle / bottom).
    pub gray_step: ScaleStep,
    /// Fraction down the page where the wash reaches gray (`1/3` matches Radix Colors).
    pub settle_at: f32,
}

impl Default for PageBackground {
    fn default() -> Self {
        Self { color_step: 3, gray_step: 1, settle_at: 1.0 / 3.0 }
    }
}

impl PageBackground {
    pub fn stops(self, look: &Look) -> (Hsla, Hsla) {
        (
            look.resolve_step(ScaleFamily::Color, self.color_step).hsla(),
            look.resolve_step(ScaleFamily::Gray, self.gray_step).hsla(),
        )
    }

    /// Top → down linear wash suitable for `.bg(...)`.
    pub fn paint(self, look: &Look) -> Background {
        let (from, to) = self.stops(look);
        linear_gradient(180.0, linear_color_stop(from, 0.0), linear_color_stop(to, self.settle_at))
    }
}

/// Shared palette and control styling state; cloning shares state, `fork` copies it.
///
/// # Redrawing after changes
/// Setters update state and revision only: they do **not** send GPUI notifications.
/// Notify the owning view and any independently rendered control entities that use
/// this look. Controls resolve new colors on their next render. Rebuild passive
/// elements (Avatar, Badge, Card) in the owning view's render method.
///
/// ```no_run
/// use gpui::{Context, Entity};
/// use xgpui_luma::controls::button::Button;
/// use xgpui_luma::theme::ThemeMode;
/// use xgpui_luma_look_radix::Look;
///
/// fn change_mode<M: 'static>(look: &Look, button: &Entity<Button>, cx: &mut Context<M>) {
///     look.set_mode(ThemeMode::Dark);
///     button.update(cx, |_, cx| cx.notify()); // repeat for affected child entities
///     cx.notify(); // rebuild the owner's passive elements and layout
/// }
/// ```
#[derive(Clone)]
pub struct Look {
    state: Arc<LookState>,
    /// Size used by themes and templates created from this handle.
    size: crate::RadixSize,
}

impl Global for Look {}

/// `.look(&…)` if set, else ambient Global, else [`Look::built_in()`]. Never panics.
pub(crate) fn resolve_look(explicit: Option<&Look>, ambient: Option<&Look>) -> Look {
    if let Some(look) = explicit {
        return look.clone();
    }
    if let Some(look) = ambient {
        return look.clone();
    }
    Look::built_in()
}

pub(crate) fn embedded_common_stylesheet() -> &'static xgpui_luma::theme::stylesheet::CommonStylesheet {
    static STYLESHEET: std::sync::OnceLock<xgpui_luma::theme::stylesheet::CommonStylesheet> =
        std::sync::OnceLock::new();
    STYLESHEET.get_or_init(|| {
        let mut sheet = native_radix_stylesheet();
        apply_radix_slider_stylesheet(&mut sheet);
        sheet
    })
}

/// Construct the remaining Radix common overrides without bundled TOML parsing.
pub fn native_radix_stylesheet() -> xgpui_luma::theme::stylesheet::CommonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    let mut motion = CommonStylesheet::sdk_default().motion.clone();
    motion.tabs.content =
        Some(TransitionSpec::Timed { duration_ms: 300, easing: xgpui_luma::motion::Easing::OutCubic });
    let mut common = CommonStylesheet {
        motion,
        tabs: crate::tabs::native_tabs_stylesheet(),
        slider: native_radix_slider(),
        button: crate::button::native_button_stylesheet(),
        tooltip: crate::tooltip::native_tooltip_stylesheet(),
        ..Default::default()
    };
    crate::size::apply_size_presets(&mut common);
    common.with_sdk_defaults()
}

pub use crate::slider::{native_radix_slider};
pub(crate) use crate::slider::{apply_radix_slider_stylesheet};

fn sized_stylesheets(
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
) -> Arc<[Arc<xgpui_luma::theme::stylesheet::CommonStylesheet>; 3]> {
    Arc::new(crate::RadixSize::ALL.map(|size| Arc::new(stylesheet.with_size_key(size.key()))))
}

struct LookState {
    motion: RwLock<xgpui_luma::motion::MotionOverrides<xgpui_luma::theme::MotionDefaults>>,
    scales: RwLock<ScalePair>,
    palettes: RwLock<[ThemePalettes; 2]>,
    custom_contrast: RwLock<[Option<(ColorValue, Hsla)>; 2]>,
    custom_inputs: RwLock<[Option<CustomColors>; 2]>,
    metrics: MetricTokens,
    /// Font families and text styles; [`Look::set_fonts`] changes the families.
    typography: RwLock<LumaTypography>,
    /// Radix control metrics for sizes 1–3; `metrics.control` is size 2.
    control_sizes: crate::RadixControlMetrics,
    mode: AtomicU8,
    /// A style mode other than the scheme's own (e.g. `hi_contrast`), set by [`Look::set_style_mode`].
    style_mode: RwLock<Option<String>>,
    classic_shadow: RwLock<Option<ClassicButtonParams>>,
    tabs_style: RwLock<Option<crate::tabs::TabsStyle>>,
    common_stylesheet: RwLock<Arc<xgpui_luma::theme::stylesheet::CommonStylesheet>>,
    /// `common_stylesheet` with each [`crate::RadixSize`] merged into general geometry.
    sized_stylesheets: RwLock<Arc<[Arc<xgpui_luma::theme::stylesheet::CommonStylesheet>; 3]>>,
    /// Bumped by look setters; polling does not schedule GPUI redraws.
    revision: RwLock<u64>,
    /// The `.lss` theme last bound, keyed by revision and mode (see [`Look::style_theme`]).
    style_theme: crate::style::ThemeCache,
}

impl Look {
    /// Radix's own default theme: indigo accent with its paired slate gray.
    pub fn built_in() -> Self {
        Self::from_palettes(Accent::default(), Gray::default(), crate::button_layout::metric_tokens(), ThemeMode::Light)
    }

    pub fn from_palettes(accent: Accent, gray: Gray, metrics: MetricTokens, mode: ThemeMode) -> Self {
        let look = Self::new(scale_pair(accent, gray), metrics, mode);
        if let Ok(mut palettes) = look.state.palettes.write() {
            *palettes = [ThemePalettes::named(accent, gray); 2];
        }
        look
    }

    /// Scales without palette identity; both slots report as custom.
    pub fn new(scales: ScalePair, metrics: MetricTokens, mode: ThemeMode) -> Self {
        let common = Arc::new(embedded_common_stylesheet().clone());
        Self {
            size: crate::RadixSize::default(),
            state: Arc::new(LookState {
                motion: RwLock::new(Default::default()),
                scales: RwLock::new(scales),
                palettes: RwLock::new([ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom }; 2]),
                custom_contrast: RwLock::new([None; 2]),
                custom_inputs: RwLock::new([None; 2]),
                metrics,
                typography: RwLock::new(LumaTypography::default()),
                control_sizes: crate::RadixControlMetrics::from_button_boxes(metrics.control),
                mode: AtomicU8::new(mode_to_u8(mode)),
                style_mode: RwLock::new(None),
                classic_shadow: RwLock::new(None),
                tabs_style: RwLock::new(None),
                sized_stylesheets: RwLock::new(sized_stylesheets(&common)),
                common_stylesheet: RwLock::new(common),
                revision: RwLock::new(0),
                style_theme: Default::default(),
            }),
        }
    }

    /// An independent copy of this look's scales, palettes, metrics, mode, and tuning.
    ///
    /// Cloning a [`Look`] shares mutable state; callers still notify affected views.
    /// Forking is how a screen can edit a palette without touching the rest of the app.
    pub fn fork(&self) -> Self {
        let scales = *self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        let forked = Self::new(scales, self.metrics(), self.mode());
        *forked.state.motion.write().unwrap_or_else(|p| p.into_inner()) =
            self.state.motion.read().unwrap_or_else(|p| p.into_inner()).clone();
        if let Ok(mut palettes) = forked.state.palettes.write() {
            *palettes = *self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if let Ok(mut classic) = forked.state.classic_shadow.write() {
            *classic = *self.state.classic_shadow.read().unwrap_or_else(|p| p.into_inner());
        }
        if let Ok(mut tabs) = forked.state.tabs_style.write() {
            *tabs = self.tabs_style_override();
        }
        if let Ok(mut contrast) = forked.state.custom_contrast.write() {
            *contrast = *self.state.custom_contrast.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        *forked.state.custom_inputs.write().unwrap_or_else(|p| p.into_inner()) =
            *self.state.custom_inputs.read().unwrap_or_else(|p| p.into_inner());
        forked.store_stylesheet(self.stylesheet_snapshot());
        *forked.state.typography.write().unwrap_or_else(|p| p.into_inner()) = self.typography();
        forked
    }

    /// Edit SDK-defined motion defaults for future controls. Clones share edits;
    /// forks copy independently. Existing controls keep their spawn-time snapshot.
    pub fn motion(&self, configure: impl Fn(&mut xgpui_luma::theme::MotionDefaults) + Send + Sync + 'static) {
        self.state.motion.write().unwrap_or_else(|p| p.into_inner()).push(configure);
        self.bump_revision();
    }

    pub fn motion_defaults(&self) -> xgpui_luma::theme::MotionDefaults {
        let mut defaults = self.stylesheet_snapshot().motion_defaults();
        let edits = self.state.motion.read().unwrap_or_else(|p| p.into_inner()).clone();
        edits.apply(&mut defaults);
        defaults
    }

    pub fn tabs_overlay_state(&self) -> xgpui_luma::controls::tabs::TabsOverlayState {
        xgpui_luma::controls::tabs::TabsOverlayState::new().motion_defaults(self.motion_defaults().tabs_overlay)
    }

    pub fn slide_panel_state(
        &self,
        anchor: xgpui_luma::controls::slide_panel::SlidePanelTopAnchor,
        size: xgpui_luma::controls::slide_panel::SlidePanelSizeConfig,
    ) -> xgpui_luma::controls::slide_panel::SlidePanelState {
        xgpui_luma::controls::slide_panel::SlidePanelState::new(anchor, size)
            .stylesheet(&self.common_stylesheet())
            .motion_defaults(self.motion_defaults().slide_panel)
    }

    /// Font families and text styles for this look's controls.
    pub fn typography(&self) -> LumaTypography {
        self.state.typography.read().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    /// Replace the font families (`sans`, `mono`, `serif`), e.g. with an embedded font's family name.
    /// Clones share the change; forks copy independently. Notify affected views afterwards.
    pub fn set_fonts(&self, fonts: FontTokens) {
        self.state.typography.write().unwrap_or_else(|poisoned| poisoned.into_inner()).font = fonts;
        self.bump_revision();
    }

    /// Common SDK configuration. Clones share edits; forks copy independently.
    pub fn common_stylesheet(&self) -> xgpui_luma::theme::stylesheet::CommonStylesheet {
        self.stylesheet_snapshot().as_ref().clone()
    }

    // Immutable snapshots keep render-time reads cheap without holding a lock while
    // resolving controls. Setters replace the Arc; existing readers stay valid.
    pub(crate) fn stylesheet_snapshot(&self) -> Arc<xgpui_luma::theme::stylesheet::CommonStylesheet> {
        self.state.common_stylesheet.read().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    /// Update shared geometry for subsequent renders and motion for future controls.
    /// Existing motion retains its snapshot. Notify affected views after geometry edits.
    pub fn set_common_stylesheet(&self, stylesheet: xgpui_luma::theme::stylesheet::CommonStylesheet) {
        self.store_stylesheet(Arc::new(stylesheet));
        self.bump_revision();
    }

    fn store_stylesheet(&self, stylesheet: Arc<xgpui_luma::theme::stylesheet::CommonStylesheet>) {
        *self.state.sized_stylesheets.write().unwrap_or_else(|poisoned| poisoned.into_inner()) =
            sized_stylesheets(&stylesheet);
        *self.state.common_stylesheet.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = stylesheet;
    }

    /// Common stylesheet with `size` merged into general geometry, for SDK controls.
    pub(crate) fn sized_stylesheet(
        &self,
        size: crate::RadixSize,
    ) -> Arc<xgpui_luma::theme::stylesheet::CommonStylesheet> {
        self.state.sized_stylesheets.read().unwrap_or_else(|poisoned| poisoned.into_inner())[size.index()].clone()
    }

    /// A handle to the same look whose themes and templates use `size`.
    /// The look resolves the size to pixels; SDK controls only see pixels.
    pub fn with_size(&self, size: crate::RadixSize) -> Self {
        Self { state: self.state.clone(), size }
    }

    /// Size used by themes and templates created from this handle.
    /// Radix chrome corner radii in pixels.
    pub fn radii(&self) -> crate::RadixRadii {
        crate::RadixRadii::DEFAULT
    }

    pub fn size(&self) -> crate::RadixSize {
        self.size
    }

    /// Radix control metrics for each size.
    pub fn control_sizes(&self) -> crate::RadixControlMetrics {
        self.state.control_sizes
    }

    /// Shared metrics whose `control` scale is the given Radix size, for SDK geometry.
    pub fn metrics_for(&self, size: crate::RadixSize) -> MetricTokens {
        let mut metrics = self.state.metrics;
        metrics.control = self.state.control_sizes.get(size);
        metrics
    }

    /// Shared tab geometry, baseline, and indicator settings for this look.
    pub fn tabs_style(&self) -> crate::tabs::TabsStyle {
        self.tabs_style_override()
            .unwrap_or_else(|| crate::tabs::TabsStyle::from_stylesheet(&self.common_stylesheet()))
    }

    pub(crate) fn tabs_style_override(&self) -> Option<crate::tabs::TabsStyle> {
        *self.state.tabs_style.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Update tab defaults for subsequent renders. Notify affected views; see [`Look`].
    pub fn set_tabs_style(&self, style: crate::tabs::TabsStyle) {
        if let Ok(mut current) = self.state.tabs_style.write() {
            *current = Some(style);
        }
        self.bump_revision();
    }

    /// Palettes currently in use, for readouts and pickers.
    pub fn palettes(&self) -> ThemePalettes {
        self.palettes_for(self.mode())
    }

    fn palettes_for(&self, mode: ThemeMode) -> ThemePalettes {
        self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner())[mode_to_u8(mode) as usize]
    }

    /// Palette name behind one family, so swatches can report `indigo 9` over `color 9`.
    pub fn palette_label(&self, family: ScaleFamily) -> &'static str {
        self.scales().family(family).palette()
    }

    /// Select a named palette pair for subsequent renders.
    /// Notify affected views after changing it; see [`Look`].
    pub fn set_palettes(&self, accent: Accent, gray: Gray) {
        if let Ok(mut scales) = self.state.scales.write() {
            *scales = scale_pair(accent, gray);
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            *palettes = [ThemePalettes::named(accent, gray); 2];
        }
        if let Ok(mut contrast) = self.state.custom_contrast.write() {
            *contrast = [None; 2];
        }
        if let Ok(mut inputs) = self.state.custom_inputs.write() {
            *inputs = [None; 2];
        }
        self.bump_revision();
    }

    /// Current [`ButtonVariant::Classic`](crate::ButtonVariant::Classic) shadow geometry.
    pub fn classic_params(&self) -> ClassicButtonParams {
        self.state
            .classic_shadow
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_default()
    }

    /// Retunes the Classic bubble for every button resolved from this look.
    /// Notify affected views after changing it; see [`Look`].
    pub fn set_classic_params(&self, params: ClassicButtonParams) {
        if let Ok(mut current) = self.state.classic_shadow.write() {
            *current = Some(params);
        }
        if let Ok(mut revision) = self.state.revision.write() {
            *revision = revision.wrapping_add(1);
        }
    }

    pub fn mode(&self) -> ThemeMode {
        mode_from_u8(self.state.mode.load(Ordering::Relaxed))
    }

    /// Change the active scheme (light or dark); the style mode becomes the one of the same name.
    /// Notify affected views afterward; see [`Look`].
    pub fn set_mode(&self, mode: ThemeMode) {
        if let Ok(mut style_mode) = self.state.style_mode.write() {
            *style_mode = None;
        }
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
        if let Ok(mut rev) = self.state.revision.write() {
            *rev = rev.wrapping_add(1);
        }
    }

    /// The style mode the look's `.lss` theme binds: one set with [`Look::set_style_mode`], else the
    /// scheme's own (`"light"` or `"dark"`).
    pub fn style_mode(&self) -> String {
        let named = self.state.style_mode.read().ok().and_then(|m| m.clone());
        named.unwrap_or_else(|| xgpui_luma::style::scheme_name(self.mode()).to_string())
    }

    /// Choose a declared style mode by name (e.g. a `hi_contrast` mode based on `light`). The scheme
    /// follows the mode's base, so native scales and window appearance match. Errors for a mode the
    /// Radix config doesn't declare, or one that comes down to neither light nor dark. Notify affected
    /// views afterward; see [`Look`].
    pub fn set_style_mode(&self, mode: &str) -> Result<(), String> {
        let config = crate::style::config();
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

    /// Rebuilds both accent scales using Radix's custom-color generator.
    /// Uses each mode's current gray step 8 and background; leaves gray unchanged.
    /// Notify affected views after changing it; see [`Look`].
    pub fn set_accent_seed(&self, seed: ColorValue) -> anyhow::Result<()> {
        let pair = *self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        let generate = |mode| {
            let scales = pair.for_mode(mode);
            generate_colors(
                CustomColors { accent: seed, gray: scales.gray.source_step(8), background: scales.gray.source_step(1) },
                mode,
            )
        };
        // Validate/generate both modes before changing shared state.
        let generated = [generate(ThemeMode::Light)?, generate(ThemeMode::Dark)?];
        if let Ok(mut scales) = self.state.scales.write() {
            scales.light.color = generated[0].accent;
            scales.dark.color = generated[1].accent;
        }
        if let Ok(mut contrast) = self.state.custom_contrast.write() {
            *contrast = generated.map(|colors| Some((colors.accent_contrast_source, colors.accent_contrast)));
        }
        if let Ok(mut inputs) = self.state.custom_inputs.write() {
            *inputs = generated.map(|colors| Some(colors.inputs));
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            for palette in palettes.iter_mut() {
                palette.accent = PaletteSlot::Custom;
            }
        }
        self.bump_revision();
        Ok(())
    }

    /// Regenerates the active mode's accent and gray scales from all three seed colors.
    /// The caller owns editor state and must notify affected views; see [`Look`].
    /// The background affects scale generation; page composition stays with the caller.
    pub fn set_custom_colors(&self, inputs: CustomColors) -> anyhow::Result<()> {
        let mode = self.mode();
        let generated = generate_colors(inputs, mode)?;
        if let Ok(mut pair) = self.state.scales.write() {
            let scales = match mode {
                ThemeMode::Light => &mut pair.light,
                ThemeMode::Dark => &mut pair.dark,
            };
            scales.color = generated.accent;
            scales.gray = generated.gray;
        }
        if let Ok(mut contrast) = self.state.custom_contrast.write() {
            contrast[mode_to_u8(mode) as usize] = Some((generated.accent_contrast_source, generated.accent_contrast));
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            palettes[mode_to_u8(mode) as usize] =
                ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom };
        }
        if let Ok(mut stored) = self.state.custom_inputs.write() {
            stored[mode_to_u8(mode) as usize] = Some(inputs);
        }
        self.bump_revision();
        Ok(())
    }

    fn custom_contrast_for(&self, mode: ThemeMode) -> Option<Hsla> {
        self.state.custom_contrast.read().unwrap_or_else(|poisoned| poisoned.into_inner())[mode_to_u8(mode) as usize]
            .map(|(_, preview)| preview)
    }

    /// Retained authored seeds for the active mode, including seeds replaced by derived solids.
    pub fn custom_inputs(&self) -> Option<CustomColors> {
        self.state.custom_inputs.read().unwrap_or_else(|p| p.into_inner())[mode_to_u8(self.mode()) as usize]
    }

    fn bump_revision(&self) {
        if let Ok(mut revision) = self.state.revision.write() {
            *revision = revision.wrapping_add(1);
        }
    }

    /// The look's scales bound to the Radix `.lss` files in the current mode. Cached: rebinds only after
    /// a setter bumped the revision or the mode changed.
    pub fn style_theme(&self) -> xgpui_luma::style::lss::Theme {
        self.state.style_theme.get(self)
    }

    pub fn revision(&self) -> u64 {
        *self.state.revision.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn metrics(&self) -> MetricTokens {
        self.state.metrics
    }

    pub fn scales(&self) -> ModeScales {
        self.scales_for(self.mode())
    }

    /// Scales for `mode`, whatever the current mode is.
    pub(crate) fn scales_for(&self, mode: ThemeMode) -> ModeScales {
        self.state
            .scales
            .read()
            .map(|scales| scales.for_mode(mode))
            .unwrap_or_else(|poisoned| poisoned.into_inner().for_mode(mode))
    }

    /// Resolve a role without reducing its source to the current backend gamut.
    pub fn resolve_role_source(&self, role: SemanticRole) -> ResolvedSourceColor {
        if role == SemanticRole::PrimaryForeground {
            if let Some((source, _)) =
                self.state.custom_contrast.read().unwrap_or_else(|p| p.into_inner())[mode_to_u8(self.mode()) as usize]
            {
                return ResolvedSourceColor {
                    value: source,
                    source: xgpui_luma::theme::provenance::ColorSource::Authored {
                        key: "custom-accent-contrast".into(),
                    },
                };
            }
            if self.accent_uses_dark_solid_contrast() {
                let step = if self.scales().gray.step(1).l <= self.scales().gray.step(12).l {
                    1
                } else {
                    12
                };
                return self.resolve_step_source(ScaleFamily::Gray, step);
            }
        }
        let mapping = role.mapping(self.mode());
        self.resolve_step_source(mapping.family, mapping.step)
    }

    pub fn resolve_step_source(&self, family: ScaleFamily, step: ScaleStep) -> ResolvedSourceColor {
        self.scales().family(family).resolved_source(step)
    }

    pub fn resolve_role(&self, role: SemanticRole) -> ResolvedColor {
        self.resolve_role_for(role, self.mode())
    }

    /// [`Self::resolve_role`] for `mode`, whatever the current mode is.
    pub(crate) fn resolve_role_for(&self, role: SemanticRole, mode: ThemeMode) -> ResolvedColor {
        if role == SemanticRole::PrimaryForeground {
            if let Some(contrast) = self.custom_contrast_for(mode) {
                return ResolvedColor::authored(contrast, "custom-accent-contrast");
            }
            if self.accent_uses_dark_solid_contrast_for(mode) {
                return self.dark_solid_contrast_for(mode);
            }
        }
        role.resolve(self.scales_for(mode), mode)
    }

    /// Control thumbs retain their light palette in both theme modes.
    #[cfg(test)]
    pub(crate) fn light_gray_step(&self, step: ScaleStep) -> Hsla {
        self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner()).light.gray.step(step)
    }

    pub fn resolve_step(&self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.scales().resolved(family, step)
    }

    /// Whether solid accent faces need dark labels (lime / mint / sky / yellow / amber).
    pub fn accent_uses_dark_solid_contrast(&self) -> bool {
        self.accent_uses_dark_solid_contrast_for(self.mode())
    }

    fn accent_uses_dark_solid_contrast_for(&self, mode: ThemeMode) -> bool {
        match self.palettes_for(mode).accent {
            PaletteSlot::Named(name) => {
                Accent::ALL.iter().any(|accent| accent.as_str() == name && accent.uses_dark_solid_contrast())
            }
            PaletteSlot::Custom => self
                .custom_contrast_for(mode)
                .map(|color| color.l < 0.5)
                .unwrap_or_else(|| self.scales_for(mode).resolved(ScaleFamily::Color, 9).hsla().l >= 0.68),
        }
    }

    /// Dark label for bright solid faces — the darker of gray 1 / gray 12 in the active mode.
    fn dark_solid_contrast_for(&self, mode: ThemeMode) -> ResolvedColor {
        let scales = self.scales_for(mode);
        let gray1 = scales.resolved(ScaleFamily::Gray, 1);
        let gray12 = scales.resolved(ScaleFamily::Gray, 12);
        if gray1.hsla().l <= gray12.hsla().l {
            gray1
        } else {
            gray12
        }
    }

    /// Default page background recipe: **color #3 → gray #1**, settling at ⅓.
    pub fn page_background(&self) -> Background {
        PageBackground::default().paint(self)
    }
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn mode_from_u8(value: u8) -> ThemeMode {
    if value == 1 { ThemeMode::Dark } else { ThemeMode::Light }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::color::gpui_bridge;

    #[test]
    fn style_mode_is_the_scheme_unless_set_by_name_and_follows_set_mode() {
        let look = Look::built_in();
        assert_eq!(look.style_mode(), "light");
        look.set_style_mode("dark").expect("declared");
        assert_eq!((look.mode(), look.style_mode().as_str()), (ThemeMode::Dark, "dark"));
        assert_eq!(look.style_theme().mode(), "dark", "the theme binds the style mode");
        let err = look.set_style_mode("sepia").expect_err("undeclared");
        assert!(err.contains("unknown style mode \"sepia\" (modes: light, dark)"), "{err}");
        assert_eq!(look.mode(), ThemeMode::Dark, "a failed set changes nothing");
        look.set_mode(ThemeMode::Light);
        assert_eq!(look.style_mode(), "light");
    }

    #[test]
    fn shared_stylesheet_readers_survive_replacement_and_fork_edits() {
        let look = Look::built_in();
        let clone = look.clone();
        let fork = look.fork();
        let old = look.stylesheet_snapshot();
        let original = old.slider.geometry.width;
        let revision = look.revision();
        let mut edited = look.common_stylesheet();
        edited.slider.geometry.width = Some(123.0);
        // Editing the public owned copy must not mutate live readers.
        assert_eq!(look.stylesheet_snapshot().slider.geometry.width, original);
        look.set_common_stylesheet(edited);
        assert!(look.revision() > revision);
        assert_eq!(clone.stylesheet_snapshot().slider.geometry.width, Some(123.0));
        assert_eq!(old.slider.geometry.width, original);
        assert_eq!(fork.stylesheet_snapshot().slider.geometry.width, original);
        let current = look.stylesheet_snapshot();
        look.set_mode(ThemeMode::Dark);
        assert!(Arc::ptr_eq(&current, &look.stylesheet_snapshot()));
        fork.set_common_stylesheet(Default::default());
        assert_eq!(look.stylesheet_snapshot().slider.geometry.width, Some(123.0));
    }

    #[test]
    fn fonts_are_shared_by_clones_and_copied_by_forks() {
        let look = Look::built_in();
        let clone = look.clone();
        let fork = look.fork();
        let original = look.typography().font.sans.family;
        let revision = look.revision();
        let mut fonts = look.typography().font;
        fonts.sans.family = "Replacement Sans".into();
        look.set_fonts(fonts);
        assert!(look.revision() > revision, "the style theme rebinds");
        assert_eq!(clone.typography().font.sans.family, "Replacement Sans");
        assert_eq!(fork.typography().font.sans.family, original);
        let forked = look.fork();
        assert_eq!(forked.typography().font.sans.family, "Replacement Sans", "a fork copies the fonts");
    }

    #[test]
    fn source_roles_and_custom_seeds_survive_forks_and_failed_updates() {
        let look = Look::built_in();
        let p3 = ColorValue::display_p3(1.0, 0.0, 0.0, 1.0);
        look.set_accent_seed(p3).unwrap();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            assert_eq!(look.resolve_role_source(SemanticRole::Primary).value, p3);
            assert_eq!(look.custom_inputs().unwrap().accent, p3);
            assert_eq!(look.fork().custom_inputs().unwrap().accent, p3);
        }
        let revision = look.revision();
        assert!(look.set_accent_seed(ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0)).is_err());
        assert_eq!(look.revision(), revision);
        assert_eq!(look.resolve_role_source(SemanticRole::Primary).value, p3);
        look.set_palettes(Accent::Indigo, Gray::Auto);
        assert!(look.custom_inputs().is_none());
    }

    #[test]
    fn common_stylesheet_snapshot_and_fork_are_isolated() {
        use xgpui_luma::theme::stylesheet::CommonStylesheet;
        let look = Look::built_in();
        let snapshot = look.common_stylesheet();
        let fork = look.fork();
        fork.set_common_stylesheet(CommonStylesheet::default());
        assert_eq!(look.common_stylesheet(), snapshot);
        assert_ne!(fork.common_stylesheet(), snapshot);
        let sdk = xgpui_luma::controls::tabs::Tabs::new("snapshot").motion_defaults(snapshot.motion_defaults().tabs);
        look.set_common_stylesheet(CommonStylesheet::default());
        assert_eq!(
            sdk.motion_profile().content.description(),
            xgpui_luma::motion::Transition::timed(
                std::time::Duration::from_millis(300),
                xgpui_luma::motion::Easing::OutCubic
            )
            .description()
        );
        assert_eq!(look.common_stylesheet(), CommonStylesheet::default());
    }

    #[test]
    fn mode_switch_changes_resolved_background() {
        let look = Look::built_in();
        let light_bg = look.resolve_role(SemanticRole::Background).hsla();
        look.set_mode(ThemeMode::Dark);
        let dark_bg = look.resolve_role(SemanticRole::Background).hsla();
        assert_ne!(light_bg.l, dark_bg.l);
        assert_eq!(look.revision(), 1);
    }

    #[test]
    fn built_in_reports_indigo_on_slate() {
        let look = Look::built_in();

        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
        assert_eq!(look.palette_label(ScaleFamily::Color), "indigo");
        assert_eq!(look.palette_label(ScaleFamily::Gray), "slate");
    }

    #[test]
    fn a_fork_starts_equal_and_then_drifts_alone() {
        let theme = Look::built_in();
        let draft = theme.fork();
        assert_eq!(draft.palettes(), theme.palettes());
        assert_eq!(draft.resolve_role(SemanticRole::Primary).hsla(), theme.resolve_role(SemanticRole::Primary).hsla());

        draft.set_accent_seed(gpui_bridge::from_hsla(gpui::hsla(0.05, 0.9, 0.5, 1.0))).unwrap();

        assert_ne!(draft.resolve_role(SemanticRole::Primary).hsla(), theme.resolve_role(SemanticRole::Primary).hsla());
        assert_eq!(draft.palettes().accent, PaletteSlot::Custom);
        assert_eq!(theme.palettes().accent, PaletteSlot::Named("indigo"));
        // The gray slot is untouched by an accent seed.
        assert_eq!(draft.palettes().gray, theme.palettes().gray);
    }

    #[test]
    fn set_palettes_restores_named_scales() {
        let draft = Look::built_in().fork();
        draft.set_accent_seed(gpui_bridge::from_hsla(gpui::hsla(0.05, 0.9, 0.5, 1.0))).unwrap();

        draft.set_palettes(Accent::Indigo, Gray::Auto);

        assert_eq!(draft.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
        assert_eq!(
            draft.resolve_role(SemanticRole::Primary).hsla(),
            Look::built_in().resolve_role(SemanticRole::Primary).hsla()
        );
    }

    #[test]
    fn custom_palette_is_mode_local_and_fork_copies_contrast() {
        let look = Look::from_palettes(Accent::Yellow, Gray::Auto, Default::default(), ThemeMode::Light);
        let original = look.resolve_step(ScaleFamily::Color, 9).hsla();
        let light_contrast = look.resolve_role(SemanticRole::PrimaryForeground).hsla();
        look.set_mode(ThemeMode::Dark);
        look.set_custom_colors(CustomColors {
            accent: gpui_bridge::from_rgba(gpui::rgb(0xffff00)),
            gray: gpui_bridge::from_rgba(gpui::rgb(0x888888)),
            background: ColorValue::srgb(0.0, 0.0, 0.0, 1.0),
        })
        .unwrap();
        let dark_contrast = look.resolve_role(SemanticRole::PrimaryForeground).hsla();
        assert!(dark_contrast.l < 0.5);
        assert_eq!(look.fork().resolve_role(SemanticRole::PrimaryForeground).hsla(), dark_contrast);
        look.set_mode(ThemeMode::Light);
        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Yellow, Gray::Auto));
        assert_eq!(look.resolve_step(ScaleFamily::Color, 9).hsla(), original);
        assert_eq!(look.resolve_role(SemanticRole::PrimaryForeground).hsla(), light_contrast);
        look.set_palettes(Accent::Indigo, Gray::Auto);
        look.set_mode(ThemeMode::Dark);
        assert!(look.custom_contrast_for(ThemeMode::Dark).is_none());
        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
    }

    #[test]
    fn page_background_mixes_color_and_gray() {
        let look = Look::built_in();
        let recipe = PageBackground::default();
        let (from, to) = recipe.stops(&look);
        let color_3 = look.resolve_step(ScaleFamily::Color, 3).hsla();
        let gray_1 = look.resolve_step(ScaleFamily::Gray, 1).hsla();
        assert_eq!(from, color_3);
        assert_eq!(to, gray_1);
        assert_ne!(from.s, to.s);
    }

    #[test]
    fn independent_control_assembly_matches_bundled_styles() {
        use xgpui_luma::theme::stylesheet::{CommonStylesheet, define_sdk_slider};
        let original = native_radix_stylesheet();
        // Assemble the controls independently from an empty sheet.
        let mut native = CommonStylesheet::default();
        define_sdk_slider(&mut native);
        apply_radix_slider_stylesheet(&mut native);
        crate::size::apply_size_presets(&mut native);
        assert_eq!(native.slider, original.slider);
        assert_eq!(native.motion.slider, original.motion.slider);
        assert_eq!(native.recipes.0.get("slider"), original.recipes.0.get("slider"));

        let old_look = Look::built_in();
        old_look.set_common_stylesheet(original);
        let new_look = old_look.fork();
        new_look.set_common_stylesheet(native);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            old_look.set_mode(mode);
            new_look.set_mode(mode);
            for variant in crate::slider::SliderVariant::ALL {
                for size in crate::RadixSize::ALL {
                    let old_theme = crate::slider::slider_theme_with(&old_look.with_size(size), variant);
                    let new_theme = crate::slider::slider_theme_with(&new_look.with_size(size), variant);
                    for bits in 0..32 {
                        let state = xgpui_luma::theme::InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            invalid: bits & 16 != 0,
                        };
                        assert_eq!(
                            format!("{:?}", new_theme.resolve(state)),
                            format!("{:?}", old_theme.resolve(state)),
                            "{mode:?} {variant:?} {size:?} {state:?}",
                        );
                    }
                }
            }
        }
    }
}
