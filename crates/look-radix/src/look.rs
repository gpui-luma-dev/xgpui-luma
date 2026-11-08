//! Runtime radix look: mode + dual scale space (color + gray).

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{Background, Global, Hsla, linear_color_stop, linear_gradient};
use xgpui_luma::theme::{MetricTokens, ThemeMode};
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
        apply_radix_switch_stylesheet(&mut sheet);
        apply_radix_checkbox_stylesheet(&mut sheet);
        sheet
    })
}

/// Construct the remaining Radix common overrides without bundled TOML parsing.
pub fn native_radix_stylesheet() -> xgpui_luma::theme::stylesheet::CommonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    let mut motion = CommonStylesheet::sdk_default().motion.clone();
    motion.tabs.content =
        Some(TransitionSpec::Timed { duration_ms: 300, easing: xgpui_luma::motion::Easing::OutCubic });
    CommonStylesheet {
        motion,
        tabs: crate::tabs::native_tabs_stylesheet(),
        slider: native_radix_slider(),
        switch: native_radix_switch(),
        checkbox: native_radix_checkbox(),
        button: crate::button::native_button_stylesheet(),
        radio: crate::radio::native_radio_stylesheet(),
        textfield: crate::textfield::native_textfield_stylesheet(),
        progress: crate::progress::native_progress_stylesheet(),
        textarea: crate::textarea::native_textarea_stylesheet(),
        floating_menu: crate::context_menu::native_floating_menu_stylesheet(),
        tooltip: crate::tooltip::native_tooltip_stylesheet(),
        toolbar: crate::toolbar::native_toolbar_stylesheet(),
        segmented: crate::segmented::native_segmented_stylesheet(),
        avatar: crate::avatar::native_avatar_stylesheet(),
        badge: crate::badge::native_badge_stylesheet(),
        card: crate::card::native_card_stylesheet(),
        callout: crate::callout::native_callout_stylesheet(),
        overlay_window: crate::overlay_window::native_overlay_window_stylesheet(),
        popup_menu: crate::popup_menu::native_popup_menu_stylesheet(),
        context_menu: crate::context_menu::native_context_menu_stylesheet(),
        radio_group: crate::radio_group::native_radio_group_stylesheet(),
        ..Default::default()
    }
    .with_sdk_defaults()
}

pub use crate::slider::{native_radix_slider};
pub(crate) use crate::slider::{apply_radix_slider_stylesheet};
pub use crate::switch::{native_radix_switch};
pub(crate) use crate::switch::{apply_radix_switch_stylesheet};
pub use crate::checkbox::{native_radix_checkbox};
pub(crate) use crate::checkbox::{apply_radix_checkbox_stylesheet};

pub(crate) fn sdk_size_key(size: xgpui_luma::theme::ControlSize) -> &'static str {
    match size {
        xgpui_luma::theme::ControlSize::Sm => "1",
        xgpui_luma::theme::ControlSize::Md => "2",
        xgpui_luma::theme::ControlSize::Lg => "3",
    }
}

struct LookState {
    motion: RwLock<xgpui_luma::motion::MotionOverrides<xgpui_luma::theme::MotionDefaults>>,
    scales: RwLock<ScalePair>,
    palettes: RwLock<[ThemePalettes; 2]>,
    custom_contrast: RwLock<[Option<(ColorValue, Hsla)>; 2]>,
    custom_inputs: RwLock<[Option<CustomColors>; 2]>,
    metrics: MetricTokens,
    mode: AtomicU8,
    classic_shadow: RwLock<Option<ClassicButtonParams>>,
    button_recipes: RwLock<Arc<crate::button_recipes::Recipes>>,
    tabs_style: RwLock<Option<crate::tabs::TabsStyle>>,
    common_stylesheet: RwLock<Arc<xgpui_luma::theme::stylesheet::CommonStylesheet>>,
    /// Bumped by look setters; polling does not schedule GPUI redraws.
    revision: RwLock<u64>,
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
        Self {
            state: Arc::new(LookState {
                motion: RwLock::new(Default::default()),
                scales: RwLock::new(scales),
                palettes: RwLock::new([ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom }; 2]),
                custom_contrast: RwLock::new([None; 2]),
                custom_inputs: RwLock::new([None; 2]),
                metrics,
                mode: AtomicU8::new(mode_to_u8(mode)),
                classic_shadow: RwLock::new(None),
                button_recipes: RwLock::new(Arc::new(crate::button_recipes::embedded().clone())),
                tabs_style: RwLock::new(None),
                common_stylesheet: RwLock::new(Arc::new(embedded_common_stylesheet().clone())),
                revision: RwLock::new(0),
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
        *forked.state.common_stylesheet.write().unwrap_or_else(|poisoned| poisoned.into_inner()) =
            self.stylesheet_snapshot();
        *forked.state.button_recipes.write().unwrap_or_else(|p| p.into_inner()) = self.button_recipes();
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
        // Numeric slider identities belong to Radix; the SDK stores standard sizes directly.
        let mut stylesheet = stylesheet;
        let mut numeric = xgpui_luma::theme::stylesheet::SliderStylesheet::default();
        for (key, size) in [
            ("1", xgpui_luma::theme::ControlSize::Sm),
            ("2", xgpui_luma::theme::ControlSize::Md),
            ("3", xgpui_luma::theme::ControlSize::Lg),
        ] {
            if let Some(authored) = stylesheet.slider.sizes.custom.get(key) {
                *numeric.sizes.get_or_default(size) = authored.clone();
            }
        }
        numeric.apply_to(&mut stylesheet.slider);
        *self.state.common_stylesheet.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::new(stylesheet);
        self.bump_revision();
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
        self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner())[mode_to_u8(self.mode()) as usize]
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

    pub(crate) fn button_recipes(&self) -> Arc<crate::button_recipes::Recipes> {
        self.state.button_recipes.read().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Load sparse look-owned button recipes over the native look defaults.
    /// Rules replace the complete ordered list. Programmatic Classic tuning stays
    /// authoritative over stylesheet tuning.
    pub fn set_button_recipes(&self, source: &str) -> anyhow::Result<()> {
        let mut baseline = toml::Value::Table(toml::map::Map::from_iter([(
            "button_recipes".into(),
            toml::Value::try_from(crate::button_recipes::embedded())?,
        )]));
        let authored: toml::Value = toml::from_str(source)?;
        anyhow::ensure!(
            authored.as_table().is_some_and(|table| table.keys().all(|key| key == "button_recipes")),
            "expected only a button_recipes table"
        );
        fn merge(base: &mut toml::Value, layer: toml::Value) {
            match (base, layer) {
                (toml::Value::Table(base), toml::Value::Table(layer)) => {
                    for (key, value) in layer {
                        if let Some(base) = base.get_mut(&key) {
                            merge(base, value);
                        } else {
                            base.insert(key, value);
                        }
                    }
                }
                (base, layer) => *base = layer,
            }
        }
        merge(&mut baseline, authored);
        let parsed = crate::button_recipes::parse(&toml::to_string(&baseline)?)?;
        *self.state.button_recipes.write().unwrap_or_else(|p| p.into_inner()) = Arc::new(parsed);
        self.bump_revision();
        Ok(())
    }

    /// Current [`ButtonVariant::Classic`](crate::ButtonVariant::Classic) shadow geometry.
    pub fn classic_params(&self) -> ClassicButtonParams {
        self.state
            .classic_shadow
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_else(|| self.button_recipes().classic)
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

    /// Change the active mode. Notify affected views afterward; see [`Look`].
    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
        if let Ok(mut rev) = self.state.revision.write() {
            *rev = rev.wrapping_add(1);
        }
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

    fn custom_contrast(&self) -> Option<Hsla> {
        self.state.custom_contrast.read().unwrap_or_else(|poisoned| poisoned.into_inner())
            [mode_to_u8(self.mode()) as usize]
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

    pub fn revision(&self) -> u64 {
        *self.state.revision.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn metrics(&self) -> MetricTokens {
        self.state.metrics
    }

    pub fn scales(&self) -> ModeScales {
        self.state
            .scales
            .read()
            .map(|scales| scales.for_mode(self.mode()))
            .unwrap_or_else(|poisoned| poisoned.into_inner().for_mode(self.mode()))
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
        if role == SemanticRole::PrimaryForeground {
            if let Some(contrast) = self.custom_contrast() {
                return ResolvedColor::authored(contrast, "custom-accent-contrast");
            }
            if self.accent_uses_dark_solid_contrast() {
                return self.dark_solid_contrast();
            }
        }
        role.resolve(self.scales(), self.mode())
    }

    /// Control thumbs retain their light palette in both theme modes.
    pub(crate) fn light_gray_step(&self, step: ScaleStep) -> Hsla {
        self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner()).light.gray.step(step)
    }

    pub fn resolve_step(&self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.scales().resolved(family, step)
    }

    /// Whether solid accent faces need dark labels (lime / mint / sky / yellow / amber).
    pub fn accent_uses_dark_solid_contrast(&self) -> bool {
        match self.palettes().accent {
            PaletteSlot::Named(name) => {
                Accent::ALL.iter().any(|accent| accent.as_str() == name && accent.uses_dark_solid_contrast())
            }
            PaletteSlot::Custom => self
                .custom_contrast()
                .map(|color| color.l < 0.5)
                .unwrap_or_else(|| self.resolve_step(ScaleFamily::Color, 9).hsla().l >= 0.68),
        }
    }

    /// Dark label for bright solid faces — the darker of gray 1 / gray 12 in the active mode.
    fn dark_solid_contrast(&self) -> ResolvedColor {
        let gray1 = self.resolve_step(ScaleFamily::Gray, 1);
        let gray12 = self.resolve_step(ScaleFamily::Gray, 12);
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
    fn shared_stylesheet_readers_survive_replacement_and_fork_edits() {
        let look = Look::built_in();
        let clone = look.clone();
        let fork = look.fork();
        let old = look.stylesheet_snapshot();
        let original = old.tokens.typography.font.sans.family.clone();
        let revision = look.revision();
        let mut edited = look.common_stylesheet();
        edited.tokens.typography.font.sans.family = "Replacement Sans".into();
        // Editing the public owned copy must not mutate live readers.
        assert_eq!(look.stylesheet_snapshot().tokens.typography.font.sans.family, original);
        look.set_common_stylesheet(edited);
        assert!(look.revision() > revision);
        assert_eq!(clone.stylesheet_snapshot().tokens.typography.font.sans.family, "Replacement Sans");
        assert_eq!(old.tokens.typography.font.sans.family, original);
        assert_eq!(fork.stylesheet_snapshot().tokens.typography.font.sans.family, original);
        let current = look.stylesheet_snapshot();
        look.set_mode(ThemeMode::Dark);
        assert!(Arc::ptr_eq(&current, &look.stylesheet_snapshot()));
        fork.set_common_stylesheet(Default::default());
        assert_eq!(look.stylesheet_snapshot().tokens.typography.font.sans.family, "Replacement Sans");
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
        assert!(look.custom_contrast().is_none());
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
        use xgpui_luma::theme::stylesheet::{CommonStylesheet, define_sdk_checkbox, define_sdk_slider, define_sdk_switch};
        let original = native_radix_stylesheet();
        // Assemble the controls independently from an empty sheet.
        let mut native = CommonStylesheet::default();
        define_sdk_slider(&mut native);
        define_sdk_switch(&mut native);
        define_sdk_checkbox(&mut native);
        apply_radix_slider_stylesheet(&mut native);
        apply_radix_switch_stylesheet(&mut native);
        apply_radix_checkbox_stylesheet(&mut native);
        assert_eq!(native.slider, original.slider);
        assert_eq!(native.switch, original.switch);
        assert_eq!(native.checkbox, original.checkbox);
        assert_eq!(native.motion.slider, original.motion.slider);
        assert_eq!(native.motion.switch, original.motion.switch);
        assert_eq!(native.motion.checkbox, original.motion.checkbox);
        for family in ["slider", "switch", "checkbox"] {
            assert_eq!(native.recipes.0.get(family), original.recipes.0.get(family));
        }

        let old_look = Look::built_in();
        old_look.set_common_stylesheet(original);
        let new_look = old_look.fork();
        new_look.set_common_stylesheet(native);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            old_look.set_mode(mode);
            new_look.set_mode(mode);
            for variant in crate::slider::SliderVariant::ALL {
                let old_theme = crate::slider::slider_theme_with(&old_look, variant);
                let new_theme = crate::slider::slider_theme_with(&new_look, variant);
                for size in [
                    xgpui_luma::theme::ControlSize::Sm,
                    xgpui_luma::theme::ControlSize::Md,
                    xgpui_luma::theme::ControlSize::Lg,
                ] {
                    for bits in 0..32 {
                        let state = xgpui_luma::theme::InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            invalid: bits & 16 != 0,
                        };
                        use xgpui_luma::controls::slider::SliderThumbSize;
                        for thumb in
                            [None, Some(SliderThumbSize::Sm), Some(SliderThumbSize::Md), Some(SliderThumbSize::Lg)]
                        {
                            assert_eq!(
                                format!("{:?}", new_theme.resolve(size, thumb, state)),
                                format!("{:?}", old_theme.resolve(size, thumb, state)),
                                "{mode:?} {variant:?} {size:?} {thumb:?} {state:?}",
                            );
                        }
                    }
                }
            }
        }
    }
}
