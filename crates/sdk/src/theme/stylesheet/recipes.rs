//! Palette recipes reuse semantic tokens; state dispatch remains control behavior.
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::theme::LumaPalette;
use gpui::Hsla;

macro_rules! palette_roles {
    ($($variant:ident => $name:literal => $($field:ident).+),+ $(,)?) => {
        /// A validated reference into the shared semantic palette.
        #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
        pub enum PaletteRole {
            $(#[serde(rename = $name)] $variant,)+
        }
        impl PaletteRole {
            fn read(self, palette: &LumaPalette<Hsla>) -> Hsla {
                match self { $(Self::$variant => palette.$($field).+,)+ }
            }
            fn write(self, palette: &mut LumaPalette<Hsla>, color: Hsla) {
                match self { $(Self::$variant => palette.$($field).+ = color,)+ }
            }
        }
    };
}
palette_roles! {
    ShellBackground => "shell.background" => shell.background,
    ShellForeground => "shell.foreground" => shell.foreground,
    ShellBorder => "shell.border" => shell.border,
    ShellHoverBackground => "shell.hover_background" => shell.hover_background,
    ShellActiveBackground => "shell.active_background" => shell.active_background,
    ShellCloseHoverBackground => "shell.close_hover_background" => shell.close_hover_background,
    ShellCloseActiveBackground => "shell.close_active_background" => shell.close_active_background,
    ShellCloseForeground => "shell.close_foreground" => shell.close_foreground,

    AppBackground => "app.background" => app.background,
    AppForeground => "app.foreground" => app.foreground,
    AppMutedForeground => "app.muted_foreground" => app.muted_foreground,
    SurfacePanelBackground => "surface.panel.background" => surface.panel.background,
    SurfacePanelForeground => "surface.panel.foreground" => surface.panel.foreground,
    SurfacePanelBorder => "surface.panel.border" => surface.panel.border,
    SurfaceFloatingBackground => "surface.floating.background" => surface.floating.background,
    SurfaceFloatingForeground => "surface.floating.foreground" => surface.floating.foreground,
    SurfaceFloatingBorder => "surface.floating.border" => surface.floating.border,
    SurfaceSubtleBackground => "surface.subtle.background" => surface.subtle.background,
    SurfaceSubtleForeground => "surface.subtle.foreground" => surface.subtle.foreground,
    StateHoverBackground => "state.hover.background" => state.hover.background,
    StateHoverForeground => "state.hover.foreground" => state.hover.foreground,
    StatePressedBackground => "state.pressed.background" => state.pressed.background,
    StateSelectedBackground => "state.selected.background" => state.selected.background,
    StateSelectedForeground => "state.selected.foreground" => state.selected.foreground,
    StateDisabledBackground => "state.disabled.background" => state.disabled.background,
    StateDisabledForeground => "state.disabled.foreground" => state.disabled.foreground,
    FormInputBackground => "form.input.background" => form.input.background,
    FormInputForeground => "form.input.foreground" => form.input.foreground,
    FormInputBorder => "form.input.border" => form.input.border,
    FormInputInvalidBorder => "form.input.invalid_border" => form.input.invalid_border,
    FormInputPlaceholder => "form.input.placeholder" => form.input.placeholder,
    FocusRing => "focus.ring" => focus.ring,
    BorderDefault => "border.default" => border.default,
    BorderStrong => "border.strong" => border.strong,
    NavigationBackground => "navigation.background" => navigation.background,
    NavigationForeground => "navigation.foreground" => navigation.foreground,
    NavigationMutedForeground => "navigation.muted_foreground" => navigation.muted_foreground,
    NavigationHoverBackground => "navigation.hover_background" => navigation.hover_background,
    NavigationSelectedBackground => "navigation.selected_background" => navigation.selected_background,
    NavigationSelectedForeground => "navigation.selected_foreground" => navigation.selected_foreground,
    NavigationBorder => "navigation.border" => navigation.border,
    DataPositive => "data.positive" => data.positive,
    DataAccent1 => "data.accent_1" => data.accent_1,
    DataAccent2 => "data.accent_2" => data.accent_2,
    DataAccent3 => "data.accent_3" => data.accent_3,
    DataAccent4 => "data.accent_4" => data.accent_4,
    DataAccent5 => "data.accent_5" => data.accent_5,

}

/// Family-local semantic mappings. Keys and values use the same validated token
/// references. This changes a recipe without copying concrete palette colors.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PaletteRecipes(pub HashMap<String, HashMap<PaletteRole, PaletteRole>>);

impl PaletteRecipes {
    pub(super) fn with_baseline(mut self, baseline: &Self) -> Self {
        for (family, roles) in &baseline.0 {
            let layer = self.0.entry(family.clone()).or_default();
            for (slot, source) in roles {
                layer.entry(*slot).or_insert(*source);
            }
        }
        self
    }

    /// Resolve a family snapshot from the supplied programmatic token palette.
    /// All sources read the original palette, so swaps are order independent.
    pub fn palette(&self, family: &str, source: &LumaPalette<Hsla>) -> LumaPalette<Hsla> {
        let mut palette = source.clone();
        let baseline = &super::CommonStylesheet::sdk_default().recipes;
        if let Some(roles) = baseline.0.get(family) {
            for (slot, role) in roles {
                let role = self.0.get(family).and_then(|roles| roles.get(slot)).unwrap_or(role);
                slot.write(&mut palette, role.read(source));
            }
        }
        if let Some(roles) = self.0.get(family) {
            for (slot, role) in roles {
                slot.write(&mut palette, role.read(source));
            }
        }
        palette
    }
}

/// Presentation row budgets; zero is retained in configuration and controls
/// enforce at least one actual row when calculating a nonempty viewport.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RowLimits {
    pub min: Option<usize>,
    pub max: Option<usize>,
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct RowLimitsStylesheet(pub HashMap<String, RowLimits>);
impl RowLimitsStylesheet {
    pub(super) fn with_baseline(mut self, baseline: &Self) -> Self {
        for (family, fallback) in &baseline.0 {
            let layer = self.0.entry(family.clone()).or_default();
            layer.min = layer.min.or(fallback.min);
            layer.max = layer.max.or(fallback.max);
        }
        self
    }
    /// Resolved presentation limits. Explicit control limits take precedence.
    pub fn resolve(&self, family: &str) -> (usize, usize) {
        let fallback = super::CommonStylesheet::sdk_default().row_limits.0.get(family).copied().unwrap_or_default();
        let layer = self.0.get(family).copied().unwrap_or_default();
        (layer.min.or(fallback.min).unwrap_or_default(), layer.max.or(fallback.max).unwrap_or_default())
    }
}

/// Named OpenType feature sets. Empty arrays deliberately clear an inherited set.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FontFeaturesStylesheet(pub HashMap<String, Vec<(String, u32)>>);
impl<'de> Deserialize<'de> for FontFeaturesStylesheet {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = HashMap::<String, Vec<(String, u32)>>::deserialize(deserializer)?;
        for features in values.values() {
            for (tag, _) in features {
                if tag.len() != 4 || !tag.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
                    return Err(serde::de::Error::custom("OpenType feature tags must have four ASCII letters/digits"));
                }
            }
        }
        Ok(Self(values))
    }
}
impl FontFeaturesStylesheet {
    pub(super) fn with_baseline(mut self, baseline: &Self) -> Self {
        for (family, features) in &baseline.0 {
            self.0.entry(family.clone()).or_insert_with(|| features.clone());
        }
        self
    }
    /// Capture the named feature set; an explicitly empty array remains empty.
    pub fn resolve(&self, family: &str) -> gpui::FontFeatures {
        let baseline = &super::CommonStylesheet::sdk_default().font_features;
        gpui::FontFeatures(std::sync::Arc::new(
            self.0.get(family).or_else(|| baseline.0.get(family)).cloned().unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        color::GamutMapping,
        theme::{ThemeMode, InteractionState},
    };
    use crate::controls::button_family::{ButtonFamilyTheme, ButtonFamilyRole, DefaultButtonFamilyTheme};
    use super::super::CommonStylesheet;

    #[test]
    fn feature_sets_inherit_disable_and_clear_atomically() {
        let inherited = CommonStylesheet::resolve_layers(&[]).unwrap();
        assert_eq!(inherited.font_features.resolve("sidebar_section").tag_value_list(), &[("smcp".to_owned(), 1)]);
        let disabled =
            CommonStylesheet::resolve_layers(&["[common.font_features]\nsidebar_section=[['smcp',0]]"]).unwrap();
        assert_eq!(disabled.font_features.resolve("sidebar_section").tag_value_list(), &[("smcp".to_owned(), 0)]);
        let cleared = CommonStylesheet::resolve_layers(&["[common.font_features]\nsidebar_section=[]"]).unwrap();
        assert!(cleared.font_features.resolve("sidebar_section").tag_value_list().is_empty());
        assert!(CommonStylesheet::parse("[common.font_features]\nsidebar_section=[['too_long',1]]").is_err());
    }

    #[test]
    fn baseline_semantic_recipes_are_identity_for_every_family_and_mode() {
        let sheet = CommonStylesheet::sdk_default().clone();
        assert!(!sheet.recipes.0.is_empty());
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let tokens = sheet.theme_tokens(mode, GamutMapping::CssLocalMinde).unwrap();
            for family in sheet.recipes.0.keys() {
                assert_eq!(sheet.recipes.palette(family, &tokens.palette), *tokens.palette, "{family}");
            }
        }
    }

    #[test]
    fn layered_recipes_use_programmatic_token_colors_without_cascading() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.recipes.button]
'state.hover.background'='data.positive'",
            "[common.recipes.button]
'state.hover.background'='state.selected.background'
'state.selected.background'='state.hover.background'",
        ])
        .unwrap();
        let mut tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
        let mut source = tokens.palette.source().clone();
        source.state.selected.background = crate::color::ColorValue::srgb(0.0, 0.75, 0.25, 1.0);
        tokens.palette = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        let expected = tokens.palette.state.selected.background;
        let swapped = sheet.recipes.palette("button", &tokens.palette);
        assert_eq!(swapped.state.selected.background, tokens.palette.state.hover.background);
        let theme = DefaultButtonFamilyTheme::new(tokens);
        assert_eq!(
            theme
                .resolve(ButtonFamilyRole::Text, InteractionState { hovered: true, ..Default::default() })
                .background,
            expected
        );
        assert!(
            CommonStylesheet::parse(
                "[common.recipes.button]
'state.hover.background'='typo.color'"
            )
            .is_err()
        );
        assert!(
            CommonStylesheet::parse(
                "[common.recipes.button]
'unknown.slot'='app.foreground'"
            )
            .is_err()
        );
    }

    #[test]
    fn captured_row_budgets_preserve_explicit_limits_in_both_orders() {
        use crate::controls::combobox::ComboBoxBuilder;
        let sheet = CommonStylesheet::parse(
            "[common.row_limits.combobox]
min=0
max=12",
        )
        .unwrap();
        let captured = ComboBoxBuilder::new("captured", []).stylesheet(&sheet);
        assert_eq!((captured.model.min_visible_rows, captured.model.max_visible_rows), (0, 12));
        for builder in [
            ComboBoxBuilder::new("before", []).min_visible_rows(3).max_visible_rows(5).stylesheet(&sheet),
            ComboBoxBuilder::new("after", []).stylesheet(&sheet).min_visible_rows(3).max_visible_rows(5),
        ] {
            assert_eq!((builder.model.min_visible_rows, builder.model.max_visible_rows), (3, 5));
        }
        for value in ["-1", "1.5", "nan"] {
            assert!(
                CommonStylesheet::parse(&format!(
                    "[common.row_limits.combobox]
max={value}"
                ))
                .is_err()
            );
        }
    }
}
