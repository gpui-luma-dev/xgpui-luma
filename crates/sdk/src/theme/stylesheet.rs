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
pub use crate::controls::navigation::tabs::theme::{
    TabsStylesheet, TabsGeometryStylesheet, TabsGeometry, ResolvedTabsGeometry,
};
pub use crate::controls::range::slider::theme::{
    SliderStylesheet, SliderGeometryStylesheet, SliderGeometry, SliderGeometrySources, SliderDecoration,
    native_sdk_slider_recipes, define_sdk_slider,
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
    shell: ShellStylesheet,
    shared: SharedStylesheet,
    row_limits: RowLimitsStylesheet,
    recipes: PaletteRecipes,
    font_features: FontFeaturesStylesheet,
    icons: IconsStylesheet,
    tabs: TabsStylesheet,
    slider: SliderStylesheet,
    button: ButtonStylesheet,
    toggle: ToggleStylesheet,
    selection_panel: SelectionPanelStylesheet,
    tooltip: TooltipStylesheet,
    selector_list: SelectorListStylesheet,
    autocomplete: AutocompleteStylesheet,
    combobox: ComboBoxStylesheet,
    search_selector: SearchSelectorStylesheet,
    popover: PopoverStylesheet,
    slide_panel: SlidePanelStylesheet,
    sidebar: SidebarStylesheet,
    control_group: ControlGroupStylesheet,
    radio_group: RadioGroupStylesheet,
    icon_group: IconGroupStylesheet,
    table: TableStylesheet,
    resizable_panels: ResizablePanelsStylesheet,
}

pub(crate) fn optional_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let value = Option::<f32>::deserialize(deserializer)?;
    if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
        return Err(serde::de::Error::custom("geometry must be finite and nonnegative"));
    }
    Ok(value)
}

pub(crate) fn optional_unit<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let value = Option::<f32>::deserialize(deserializer)?;
    if value.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value)) {
        return Err(serde::de::Error::custom("opacity must be finite and in 0..=1"));
    }
    Ok(value)
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
            tooltip: TooltipStylesheet::native_default(),
            control_group: ControlGroupStylesheet::native_default(),
            resizable_panels: ResizablePanelsStylesheet::native_default(),
            sidebar: SidebarStylesheet::native_default(),
            table: TableStylesheet::native_default(),
            popover: PopoverStylesheet::native_default(),
            slide_panel: SlidePanelStylesheet::native_default(),
            radio_group: RadioGroupStylesheet::native_default(),
            autocomplete: AutocompleteStylesheet::native_default(),
            combobox: ComboBoxStylesheet::native_default(),
            search_selector: SearchSelectorStylesheet::native_default(),
            tabs: TabsStylesheet::native_default(),
            shell: ShellStylesheet {
                geometry: ShellGeometryStylesheet {
                    fullscreen_left_padding: Some(12.0),
                    height: Some(34.0),
                    macos_left_padding: Some(80.0),
                    left_padding: Some(12.0),
                    traffic_light_inset: Some(9.0),
                    icon_size: Some(16.0),
                    border_width: Some(1.0),
                },
                ..Default::default()
            },
            shared: SharedStylesheet {
                geometry: SharedGeometryStylesheet {
                    popup_viewport_height: Some(220.0),
                    popup_row_height: Some(28.0),
                    menu_row_height: Some(27.0),
                    menu_row_gap: Some(4.0),
                    compact_textfield_height: Some(22.0),
                    compact_textfield_padding_y: Some(2.0),
                    textarea_resize_icon_size: Some(10.0),
                    textarea_resize_inset: Some(3.0),
                    textarea_resize_hit_size: Some(12.0),
                    textarea_scrollbar_width: Some(12.0),
                    textarea_scrollbar_inset: Some(2.0),
                    scrollbar_width: Some(12.0),
                    composition_underline_width: Some(1.0),
                    label_baseline: Some(1.0),
                    drag_offset: Some(12.0),
                    drag_preview_width: Some(220.0),
                    drag_padding_x: Some(12.0),
                    drag_padding_y: Some(8.0),
                    drag_border_width: Some(1.0),
                    drag_marker_width: Some(2.0),
                    split_sidebar_width: Some(280.0),
                    split_sidebar_min_width: Some(220.0),
                    split_sidebar_max_width: Some(420.0),
                    selection_scrollbar_width: Some(16.0),
                    textarea_resize_opacity: Some(0.75),
                    wide_middle_gap: Some(24.0),
                    wide_side_min_width: Some(240.0),
                    wide_side_preferred_width: Some(280.0),
                    wide_middle_min_width: Some(435.0),
                    wide_middle_max_width: Some(435.0),
                    form_field_gap: Some(4.0),
                    page_size_trigger_width: Some(64.0),
                    popup_icon_size: Some(16.0),
                },
                ..Default::default()
            },
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
    ///     theme::{ThemeMode, stylesheet::CommonStylesheet},
    /// };
    /// let sheet = CommonStylesheet::sdk_default().clone().with(|sheet| {
    ///     sheet.slider.geometry.track_height = Some(9.0);
    /// });
    /// let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde)?;
    /// assert_eq!(tokens.stylesheet.slider.geometry.track_height, Some(9.0));
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

impl CommonStylesheet {
    /// Shared helper dimensions resolved from this layer over the native SDK baseline.
    pub fn shared_geometry(&self) -> SharedGeometry {
        self.shared
            .clone()
            .with_baseline(&Self::sdk_default().shared)
            .resolve_geometry(SharedGeometry::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::provenance::ResolvedMetric;

    #[test]
    fn external_adapter_preserves_the_complete_native_baseline() {
        assert_eq!(&CommonStylesheet::resolve_layers(&[]).unwrap(), CommonStylesheet::sdk_default());
        let native = CommonStylesheet::sdk_default().clone().with(|sheet| {
            sheet.slider.geometry.width = Some(0.0);
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

    #[test]
    fn slider_size_keys_are_look_defined() {
        for key in ["1", "4", "compact"] {
            let parsed = CommonStylesheet::parse(&format!("[common.slider.sizes.{key}]\nheight = 16")).unwrap();
            assert_eq!(parsed.slider.sizes.get(key).unwrap().height, Some(16.0));
            let (geometry, sources) = parsed.slider.inspect_geometry_for(key, &SliderGeometry::default());
            assert_eq!(geometry.height, 16.0);
            assert!(matches!(sources.height, crate::theme::provenance::MetricSource::Authored { key: source }
                if source == format!("common.slider.sizes.{key}.height")));
            assert_eq!(parsed.slider.inspect_geometry(&SliderGeometry::default()).0.height, 0.0);
        }
    }

    #[test]
    fn slider_geometry_validates_values_and_preserves_zero() {
        for size in ["geometry", "sizes.1", "sizes.custom"] {
            for field in ["width", "height", "track_height", "thumb_size"] {
                for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.slider.{size}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.slider.xl]\nwidth = 12").is_err());
        let stylesheet = CommonStylesheet::parse(
            "[common.slider.geometry]\nwidth = 100\nthumb_size = 17\n[common.slider.sizes.2]\nwidth = 0\nthumb_size = 23",
        )
        .unwrap();
        let fallback = SliderGeometry::sdk_default(&Default::default(), CommonStylesheet::sdk_default());
        let (value, sources) = stylesheet.slider.inspect_geometry_for("2", &fallback);
        assert_eq!(value, stylesheet.slider.resolve_geometry_for("2", &fallback));
        assert_eq!(value.width, 0.0);
        assert_eq!(value.height, fallback.height);
        assert_eq!(value.thumb_size, 23.0);
        assert!(matches!(sources.thumb_size, crate::theme::provenance::MetricSource::Authored { key }
            if key == "common.slider.sizes.2.thumb_size"));
        let (value, _) = stylesheet.slider.inspect_geometry(&fallback);
        assert_eq!((value.width, value.thumb_size), (100.0, 17.0));
    }

    #[test]
    fn geometry_rejects_invalid_lengths_and_unknown_fields() {
        for section in ["geometry", "variants.line"] {
            for field in ["list_gap", "list_padding", "indicator_height"] {
                for value in ["-1.0", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.tabs.{section}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.tabs.geometry]\nindictor_height = 3").is_err());
    }

    #[test]
    fn geometry_preserves_fallback_and_variant_precedence() {
        let config = CommonStylesheet::parse("[common.tabs.geometry]\nlist_gap = 7.0\nindicator_height = 3.0\n[common.tabs.variants.line]\nlist_gap = 0.0").unwrap();
        let fallback = ResolvedTabsGeometry {
            list_gap: ResolvedMetric::constant(4.0, "fallback gap"),
            list_padding: ResolvedMetric::constant(2.0, "fallback padding"),
            indicator_height: ResolvedMetric::constant(2.0, "fallback indicator"),
        };
        let line = config.tabs.inspect_geometry("line", fallback.clone());
        assert_eq!(line.list_gap.value_px, 0.0);
        assert!(
            matches!(line.list_gap.source, crate::theme::provenance::MetricSource::Authored { key } if key == "common.tabs.variants.line.list_gap")
        );
        assert_eq!(line.list_padding.value_px, 2.0);
        assert!(matches!(line.list_padding.source, crate::theme::provenance::MetricSource::Constant { .. }));
        let other = config.tabs.inspect_geometry("other", fallback);
        assert_eq!(other.list_gap.value_px, 7.0);
        assert_eq!(other.indicator_height.value_px, 3.0);
    }

    #[test]
    fn slider_overlay_general_geometry_overrides_inherited_sizes() {
        for width in [120.0, 0.0] {
            let mut target = SliderStylesheet::native_default();
            target.size("1", |size| {
                size.width(260.0).dimensions(24.0, 4.0, 12.0);
            });
            target.size("2", |size| {
                size.width(260.0).dimensions(32.0, 6.0, 16.0);
            });
            let mut layer = SliderStylesheet::default();
            layer.width(width).size("2", |size| {
                size.width(180.0);
            });

            layer.apply_to(&mut target);

            assert_eq!(target.authored_geometry_for("1").width, Some(width));
            assert_eq!(target.authored_geometry_for("2").width, Some(180.0));
            assert_eq!(target.authored_geometry_for("1").height, Some(24.0));
            assert_eq!(target.authored_geometry_for("2").thumb_size, Some(16.0));
            assert_eq!(target.geometry.dial_size, Some(200.0));
        }
    }

    #[test]
    fn slider_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet
            .slider
            .width(120.0)
            .height_floor_scale(0.5)
            .thumb_scale(0.5)
            .size("1", |m| {
                m.dimensions(16.0, 4.0, 12.0);
            })
            .size("2", |m| {
                m.dimensions(20.0, 6.0, 16.0);
            });

        assert_eq!(sheet.slider.geometry.width, Some(120.0));
        assert_eq!(sheet.slider.geometry.height_floor_scale, Some(0.5));
        let size1 = sheet.slider.sizes.get("1").unwrap();
        assert_eq!(size1.height, Some(16.0));
        assert_eq!(size1.track_height, Some(4.0));
        assert_eq!(size1.thumb_size, Some(12.0));
        assert_eq!(sheet.slider.sizes.get("2").unwrap().height, Some(20.0));
    }
}
