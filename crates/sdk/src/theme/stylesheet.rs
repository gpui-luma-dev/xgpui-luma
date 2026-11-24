//! Common look configuration consumed by SDK controls.
//!
//! Looks author this contract in Rust; external documents use `[common]`.
//! Other root sections remain look-owned. Unknown common fields are rejected;
//! missing fields inherit SDK data, and explicit zero remains an override.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

mod recipes;
pub use recipes::*;
mod icons;
pub use icons::*;
pub(crate) mod geometry;
pub use geometry::*;
pub use crate::controls::navigation::tabs::theme::ResolvedTabsGeometry;
pub use crate::controls::range::slider::theme::{
    SliderGeometry, SliderGeometrySources, SliderDecoration, native_sdk_slider_recipes, define_sdk_slider,
};
mod motion;
pub use motion::*;
mod math;
mod overlay;

#[cfg(test)]
mod control_defaults;

impl<T: Serialize> Serialize for crate::theme::ControlSizes<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}

impl<'de, T: Deserialize<'de> + Default> Deserialize<'de> for crate::theme::ControlSizes<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(HashMap::<String, T>::deserialize(deserializer)?.into_iter().collect())
    }
}

macro_rules! common_stylesheet {
    ($($field:ident: $kind:ty),+ $(,)?) => {
        #[derive(Clone, Debug, Default, PartialEq, Serialize)]
        pub struct CommonStylesheet {
            /// Typed role overrides, shared by SDK-only apps and all looks.
            pub motion: MotionStylesheet,
            $(pub $field: $kind,)+
        }
        impl<'de> serde::Deserialize<'de> for CommonStylesheet {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(Default, Deserialize)]
                #[serde(default, deny_unknown_fields)]
                struct Fields {
                    motion: MotionStylesheet,
                    $($field: $kind,)+
                }
                let mut value = toml::Value::deserialize(deserializer)?;
                math::normalize(&mut value).map_err(serde::de::Error::custom)?;
                let fields: Fields = value.try_into().map_err(serde::de::Error::custom)?;
                Ok(Self { motion: fields.motion, $($field: fields.$field,)+ })
            }
        }
        impl CommonStylesheet {
            /// Fill missing fields from the SDK baseline before capturing a style snapshot.
            /// Explicit zero/false values and this layer's geometry retain precedence.
            pub fn with_sdk_defaults(&self) -> Self {
                let baseline = Self::sdk_default();
                Self {
                    motion: self.motion.clone(),
                    $($field: self.$field.clone().with_baseline(&baseline.$field),)+
                }
            }

            /// Merge one look-defined size key into each family's general geometry.
            /// Looks call this with their own size names; SDK controls only read the
            /// resulting pixel values.
            pub fn with_size_key(&self, key: &str) -> Self {
                use $crate::theme::stylesheet::geometry::SelectSizeKey;
                Self {
                    motion: self.motion.clone(),
                    $($field: self.$field.select_size_key(key),)+
                }
            }

            /// Apply an in-place mutation or decorator closure.
            pub fn with(mut self, decorator: impl FnOnce(&mut Self)) -> Self {
                decorator(&mut self);
                self
            }
        }
    };
}
common_stylesheet! {
    row_limits: RowLimitsStylesheet,
    recipes: PaletteRecipes,
    font_features: FontFeaturesStylesheet,
    icons: IconsStylesheet,
}

pub(crate) fn required_unit<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(serde::de::Error::custom("opacity must be finite and in 0..=1"));
    }
    Ok(value)
}

pub(crate) fn required_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() || value < 0.0 {
        return Err(serde::de::Error::custom("length must be finite and nonnegative"));
    }
    Ok(value)
}

pub(crate) fn required_finite<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() {
        return Err(serde::de::Error::custom("value must be finite"));
    }
    Ok(value)
}

impl CommonStylesheet {
    /// Construct the bundled SDK stylesheet using native control definitions.
    pub fn native_default() -> Self {
        let mut sheet = Self {
            motion: MotionStylesheet::native_default(),
            row_limits: RowLimitsStylesheet(HashMap::from([
                ("autocomplete".into(), RowLimits { min: Some(7), max: Some(14) }),
                ("combobox".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("search_selector".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("selection_panel".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("pager".into(), RowLimits { min: Some(5), max: Some(7) }),
            ])),
            font_features: FontFeaturesStylesheet(HashMap::from([(
                "sidebar_section".into(),
                vec![("smcp".into(), 1)],
            )])),
            icons: IconsStylesheet::native_default(),
            ..Default::default()
        };
        define_sdk_slider(&mut sheet);
        use PaletteRole::*;
        for (family, roles) in [
            (
                "button",
                &[
                    AppForeground,
                    AppMutedForeground,
                    BorderDefault,
                    FocusRing,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            (
                "control_group",
                &[AppMutedForeground, BorderDefault, StateDisabledBackground, SurfaceSubtleBackground][..],
            ),
            (
                "table",
                &[
                    AppForeground,
                    AppMutedForeground,
                    DataAccent1,
                    FocusRing,
                    FormInputBackground,
                    FormInputBorder,
                    FormInputInvalidBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StatePressedBackground,
                    SurfaceFloatingBackground,
                    SurfaceFloatingForeground,
                    SurfaceSubtleBackground,
                    SurfaceSubtleForeground,
                ][..],
            ),
            (
                "tree_view",
                &[
                    AppForeground,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    NavigationSelectedBackground,
                    NavigationSelectedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            ("dock_splitter", &[BorderDefault, StateDisabledForeground, StateSelectedBackground][..]),
            (
                "resizable_panels",
                &[BorderDefault, BorderStrong, StateDisabledBackground, StateDisabledForeground, StateHoverBackground]
                    [..],
            ),
            (
                "scrollbar",
                &[
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            ("split_view", &[BorderDefault, BorderStrong, StateDisabledBackground, StateDisabledForeground][..]),
            (
                "accordion",
                &[
                    AppForeground,
                    NavigationBorder,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "pager",
                &[
                    AppForeground,
                    AppMutedForeground,
                    FormInputBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                    SurfacePanelBackground,
                ][..],
            ),
            (
                "sidebar",
                &[
                    FocusRing,
                    NavigationForeground,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    NavigationSelectedBackground,
                    NavigationSelectedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "stepper",
                &[
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    SurfaceSubtleBackground,
                    SurfaceSubtleForeground,
                ][..],
            ),
            (
                "tabs",
                &[
                    AppForeground,
                    FocusRing,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StatePressedBackground,
                    StateSelectedBackground,
                ][..],
            ),
            (
                "context_menu",
                &[
                    AppBackground,
                    AppForeground,
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "floating_menu",
                &[
                    DataPositive,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StateHoverForeground,
                    SurfaceFloatingBackground,
                    SurfaceFloatingBorder,
                    SurfaceFloatingForeground,
                ][..],
            ),
            ("overlay_window", &[SurfaceFloatingBackground, SurfaceFloatingBorder, SurfaceFloatingForeground][..]),
            (
                "popup_menu",
                &[
                    AppBackground,
                    AppForeground,
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            (
                "progress",
                &[StateDisabledBackground, StateDisabledForeground, StateSelectedBackground, SurfaceSubtleBackground][..],
            ),
            (
                "selector",
                &[
                    AppBackground,
                    AppForeground,
                    AppMutedForeground,
                    BorderDefault,
                    FocusRing,
                    FormInputInvalidBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "textarea",
                &[
                    FormInputBackground,
                    FormInputBorder,
                    FormInputForeground,
                    FormInputInvalidBorder,
                    FormInputPlaceholder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
            ),
            (
                "textfield",
                &[
                    FormInputBackground,
                    FormInputBorder,
                    FormInputForeground,
                    FormInputInvalidBorder,
                    FormInputPlaceholder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
            ),
            ("tooltip", &[SurfaceFloatingBackground, SurfaceFloatingForeground][..]),
            ("autocomplete", &[FormInputForeground, StateSelectedBackground, SurfaceFloatingForeground][..]),
            ("toolbar", &[BorderDefault, StateDisabledBackground, SurfaceSubtleBackground][..]),
            (
                "shell",
                &[
                    ShellBackground,
                    ShellForeground,
                    ShellBorder,
                    ShellHoverBackground,
                    ShellActiveBackground,
                    ShellCloseHoverBackground,
                    ShellCloseActiveBackground,
                    ShellCloseForeground,
                ][..],
            ),
            (
                "drag_drop",
                &[SurfaceFloatingBackground, SurfaceFloatingForeground, FocusRing, FormInputInvalidBorder][..],
            ),
        ] {
            sheet.recipes.0.insert(family.into(), roles.iter().map(|role| (*role, *role)).collect());
        }
        sheet
    }

    /// Resolve validated layers over the SDK baseline, in order. Look-specific
    /// roots are ignored. Programmatic edits precede explicit control overrides.
    pub fn resolve_layers(layers: &[&str]) -> anyhow::Result<Self> {
        let mut document = toml::Value::Table(toml::map::Map::from_iter([(
            "common".into(),
            toml::Value::try_from(Self::sdk_default())?,
        )]));
        for source in layers {
            Self::parse(source)?;
            let authored: toml::Value = toml::from_str(source)?;
            overlay::overlay(&mut document, authored);
        }
        Self::parse(&toml::to_string(&document)?)
    }

    /// Prepare an immutable SDK-only or custom-look token snapshot. Rebuild tokens after editing the sheet.
    ///
    /// ```
    /// use xgpui_luma::{
    ///     color::GamutMapping,
    ///     theme::{ThemeMode, stylesheet::{CommonStylesheet, RowLimits}},
    /// };
    /// let sheet = CommonStylesheet::sdk_default().clone().with(|sheet| {
    ///     sheet.row_limits.0.insert("pager".into(), RowLimits { min: Some(3), max: Some(9) });
    /// });
    /// let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde)?;
    /// assert_eq!(tokens.stylesheet.row_limits.resolve("pager"), (3, 9));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn theme_tokens(
        &self,
        mode: crate::theme::ThemeMode,
        mapping: crate::color::GamutMapping,
    ) -> anyhow::Result<crate::theme::ThemeTokens> {
        let sheet = self.with_sdk_defaults();
        // Token values come from the SDK's token tables; the sheet holds geometry, motion and recipes.
        let source = match mode {
            crate::theme::ThemeMode::Light => crate::theme::LumaThemeMode::light(),
            crate::theme::ThemeMode::Dark => crate::theme::LumaThemeMode::dark(),
        };
        let mut tokens = crate::theme::ThemeTokens::from_source(source, mapping)?;
        tokens.stylesheet = std::sync::Arc::new(sheet);
        Ok(tokens)
    }

    /// The SDK-owned baseline, independent of any look crate. `Default` remains
    /// an empty geometry/motion overlay with inherited token data.
    pub fn sdk_default() -> &'static Self {
        static DEFAULT: std::sync::OnceLock<CommonStylesheet> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(Self::native_default)
    }

    /// Parse a complete look stylesheet. Look-owned root sections are ignored.
    /// File access and input-size limits belong to the application.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Document {
            common: CommonStylesheet,
        }
        toml::from_str::<Document>(source)
            .map(|document| document.common)
            .map_err(|err| anyhow::anyhow!("parse common stylesheet: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_adapter_preserves_the_complete_native_baseline() {
        assert_eq!(&CommonStylesheet::resolve_layers(&[]).unwrap(), CommonStylesheet::sdk_default());
        let native = CommonStylesheet::sdk_default().clone().with(|sheet| {
            sheet.row_limits.0.insert("pager".into(), super::RowLimits { min: Some(0), max: Some(3) });
            sheet.font_features.0.insert("sidebar_section".into(), Vec::new());
        });
        let source = toml::to_string(&native).unwrap();
        let restored: CommonStylesheet = toml::from_str(&source).unwrap();
        assert_eq!(restored, native);
    }

    #[test]
    fn obsolete_motion_keys_are_rejected() {
        for source in ["[common.tabs.content]\nfade_in_ms = 300", "[common.drag_drop]\ndrop_motion_ms = 200"] {
            assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
        }
    }
}
