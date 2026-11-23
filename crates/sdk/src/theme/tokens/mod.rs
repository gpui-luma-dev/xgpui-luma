pub mod elevation;
mod from_lss;
pub mod metrics;
pub mod palette;
pub mod typography;

pub use elevation::*;
pub use metrics::*;
pub use palette::*;
pub use typography::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

/// Look-authored values keyed by look-defined size names (for example Radix `"1"`).
/// The SDK itself defines no size names; SDK controls resolve the general geometry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlSizes<T> {
    entries: std::collections::HashMap<String, T>,
}

impl<T> Default for ControlSizes<T> {
    fn default() -> Self {
        Self { entries: std::collections::HashMap::new() }
    }
}

impl<T> ControlSizes<T> {
    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.entries.get_mut(key)
    }

    pub fn insert(&mut self, key: String, value: T) {
        self.entries.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.entries.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.values_mut()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut T)> {
        self.entries.iter_mut().map(|(key, value)| (key.as_str(), value))
    }
}

impl<T: Default> ControlSizes<T> {
    pub fn get_or_default(&mut self, key: &str) -> &mut T {
        self.entries.entry(key.into()).or_default()
    }
}

impl<T> FromIterator<(String, T)> for ControlSizes<T> {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Self { entries: iter.into_iter().collect() }
    }
}

impl<T, const N: usize> From<[(String, T); N]> for ControlSizes<T> {
    fn from(values: [(String, T); N]) -> Self {
        values.into_iter().collect()
    }
}

impl<T> IntoIterator for ControlSizes<T> {
    type Item = (String, T);
    type IntoIter = std::collections::hash_map::IntoIter<String, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[derive(Clone, Debug)]
pub struct LumaTheme {
    pub name: String,
    pub version: u32,
    pub modes: ThemeModes,
}

#[derive(Clone, Debug)]
pub struct LumaThemeMode {
    pub palette: LumaPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub elevation: LumaElevation,
}

/// Prepared theme tokens: source colors and an immutable current-backend preview.
/// Metrics/typography remain editable; color edits require rebuilding the palette.
#[derive(Clone, Debug)]
pub struct ThemeTokens {
    /// Resolved shared styling captured with this token snapshot.
    pub stylesheet: std::sync::Arc<crate::theme::stylesheet::CommonStylesheet>,
    pub palette: SrgbPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub elevation: SrgbElevation,
}

impl ThemeTokens {
    pub fn light() -> Self {
        Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: SrgbPalette::light(),
            metrics: MetricTokens::default(),
            typography: LumaTypography::default(),
            elevation: SrgbElevation::light(),
        }
    }
    pub fn dark() -> Self {
        Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: SrgbPalette::dark(),
            metrics: MetricTokens::default(),
            typography: LumaTypography::default(),
            elevation: SrgbElevation::dark(),
        }
    }

    /// Resolve once before passing tokens to control themes; invalid colors return a field-specific error.
    pub fn from_source(source: LumaThemeMode, mapping: crate::color::GamutMapping) -> anyhow::Result<Self> {
        Ok(Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: source.palette.snapshot_srgb(mapping)?,
            metrics: source.metrics,
            typography: source.typography,
            elevation: source.elevation.snapshot_srgb(mapping)?,
        })
    }

    /// Retrieve a standalone source copy, including current non-color tokens.
    pub fn source(&self) -> LumaThemeMode {
        LumaThemeMode {
            palette: self.palette.source().clone(),
            metrics: self.metrics,
            typography: self.typography.clone(),
            elevation: self.elevation.source().clone(),
        }
    }
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self::light()
    }
}

#[derive(Clone, Debug)]
pub struct ThemeModes {
    pub light: LumaThemeMode,
    pub dark: LumaThemeMode,
}

impl LumaTheme {
    pub fn structural_default() -> Self {
        Self {
            name: "Structural".to_string(),
            version: 1,
            modes: ThemeModes::new(LumaThemeMode::light(), LumaThemeMode::dark()),
        }
    }

    pub fn mode(&self, mode: ThemeMode) -> &LumaThemeMode {
        self.modes.tokens(mode)
    }
}

impl Default for LumaTheme {
    fn default() -> Self {
        Self::structural_default()
    }
}

impl LumaThemeMode {
    pub fn light() -> Self {
        Self::new(LumaPalette::light(), MetricTokens::default(), LumaTypography::default(), LumaElevation::light())
    }

    pub fn dark() -> Self {
        Self::new(LumaPalette::dark(), MetricTokens::default(), LumaTypography::default(), LumaElevation::dark())
    }

    pub fn new(
        palette: LumaPalette,
        metrics: MetricTokens,
        typography: LumaTypography,
        elevation: LumaElevation,
    ) -> Self {
        Self { palette, metrics, typography, elevation }
    }
}

impl ThemeModes {
    pub fn new(light: LumaThemeMode, dark: LumaThemeMode) -> Self {
        Self { light, dark }
    }

    pub fn single(tokens: LumaThemeMode) -> Self {
        Self { light: tokens.clone(), dark: tokens }
    }

    pub fn tokens(&self, mode: ThemeMode) -> &LumaThemeMode {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    pub fn tokens_mut(&mut self, mode: ThemeMode) -> &mut LumaThemeMode {
        match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    pub fn into_tokens(self, mode: ThemeMode) -> LumaThemeMode {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

impl Default for ThemeModes {
    fn default() -> Self {
        Self::new(LumaThemeMode::light(), LumaThemeMode::dark())
    }
}

impl Default for LumaThemeMode {
    fn default() -> Self {
        Self::light()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LumaElevation, LumaPalette, LumaTheme, LumaThemeMode, LumaTypography, MetricTokens, ThemeMode, ThemeModes,
        ThemeTokens,
    };

    #[test]
    fn default_tokens_use_light_structural_palette() {
        assert_eq!(
            LumaThemeMode::light().palette.app.background,
            ThemeTokens::default().palette.source().app.background
        );
    }

    #[test]
    fn theme_modes_select_light_and_dark_palettes() {
        let modes = ThemeModes::default();
        assert_ne!(
            modes.tokens(ThemeMode::Light).palette.app.background,
            modes.tokens(ThemeMode::Dark).palette.app.background
        );
    }

    #[test]
    fn structural_default_theme_exposes_semantic_layers() {
        let theme = LumaTheme::structural_default();
        let light = theme.mode(ThemeMode::Light);
        assert_eq!(theme.name, "Structural");
        assert_eq!(theme.version, 1);
        assert_eq!(light.metrics, MetricTokens::default());
        assert_eq!(light.typography, LumaTypography::default());
        assert_eq!(light.elevation, LumaElevation::light());
        assert_eq!(theme.mode(ThemeMode::Dark).palette, LumaPalette::dark());
    }
}
