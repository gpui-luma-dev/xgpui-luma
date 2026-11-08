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
pub use crate::controls::choice::switch::theme::{
    SwitchStylesheet, SwitchGeometryStylesheet, ResolvedSwitchGeometry, native_sdk_switch_recipes, define_sdk_switch,
};
pub use crate::controls::range::slider::theme::{
    SliderStylesheet, SliderGeometryStylesheet, SliderGeometry, SliderGeometrySources, SliderDecoration,
    native_sdk_slider_recipes, define_sdk_slider,
};
pub use crate::controls::choice::checkbox::theme::{native_sdk_checkbox_recipes, define_sdk_checkbox};
mod motion;
pub use motion::*;
mod colors;
mod math;
mod tokens;
pub use tokens::*;

#[cfg(test)]
mod control_defaults;

impl<T: Serialize> Serialize for crate::theme::ControlSizes<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut sizes = serializer.serialize_map(Some(3 + self.custom.len()))?;
        sizes.serialize_entry("sm", &self.sm)?;
        sizes.serialize_entry("md", &self.md)?;
        sizes.serialize_entry("lg", &self.lg)?;
        for (key, value) in &self.custom {
            sizes.serialize_entry(key, value)?;
        }
        sizes.end()
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
            pub tokens: TokenStylesheet,
            /// Typed role overrides, shared by SDK-only apps and all looks.
            pub motion: MotionStylesheet,
            $(pub $field: $kind,)+
        }
        impl<'de> serde::Deserialize<'de> for CommonStylesheet {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(Default, Deserialize)]
                #[serde(default, deny_unknown_fields)]
                struct Fields {
                    tokens: TokenStylesheet,
                    motion: MotionStylesheet,
                    $($field: $kind,)+
                }
                let mut value = toml::Value::deserialize(deserializer)?;
                math::normalize(&mut value).map_err(serde::de::Error::custom)?;
                let fields: Fields = value.try_into().map_err(serde::de::Error::custom)?;
                Ok(Self { tokens: fields.tokens, motion: fields.motion, $($field: fields.$field,)+ })
            }
        }
        impl CommonStylesheet {
            /// Fill missing fields from the SDK baseline before capturing a style snapshot.
            /// Explicit zero/false values and this layer's geometry retain precedence.
            pub fn with_sdk_defaults(&self) -> Self {
                let baseline = Self::sdk_default();
                Self {
                    tokens: self.tokens.clone(), motion: self.motion.clone(),
                    $($field: self.$field.clone().with_baseline(&baseline.$field),)+
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
    tree_view: TreeViewStylesheet,
    accordion: AccordionStylesheet,
    tabs: TabsStylesheet,
    switch: SwitchStylesheet,
    slider: SliderStylesheet,
    button: ButtonStylesheet,
    toggle: ToggleStylesheet,
    checkbox: CheckboxStylesheet,
    radio: RadioStylesheet,
    textfield: TextFieldStylesheet,
    textarea: TextAreaStylesheet,
    progress: ProgressStylesheet,
    floating_menu: FloatingMenuStylesheet,
    selection_panel: SelectionPanelStylesheet,
    tooltip: TooltipStylesheet,
    toolbar: ToolbarStylesheet,
    segmented: SegmentedStylesheet,
    avatar: AvatarStylesheet,
    badge: BadgeStylesheet,
    card: CardStylesheet,
    callout: CalloutStylesheet,
    overlay_window: OverlayWindowStylesheet,
    selector: SelectorStylesheet,
    selector_list: SelectorListStylesheet,
    autocomplete: AutocompleteStylesheet,
    combobox: ComboBoxStylesheet,
    search_selector: SearchSelectorStylesheet,
    popup_menu: PopupMenuStylesheet,
    popover: PopoverStylesheet,
    slide_panel: SlidePanelStylesheet,
    scrollbar: ScrollbarStylesheet,
    stepper: StepperStylesheet,
    sidebar: SidebarStylesheet,
    control_group: ControlGroupStylesheet,
    radio_group: RadioGroupStylesheet,
    icon_group: IconGroupStylesheet,
    table: TableStylesheet,
    pager: PagerStylesheet,
    context_menu: ContextMenuStylesheet,
    dock_splitter: DockSplitterStylesheet,
    split_view: SplitViewStylesheet,
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
            radio: RadioStylesheet::native_default(),
            progress: ProgressStylesheet::native_default(),
            stepper: StepperStylesheet::native_default(),
            scrollbar: ScrollbarStylesheet::native_default(),
            tooltip: TooltipStylesheet::native_default(),
            floating_menu: FloatingMenuStylesheet::native_default(),
            context_menu: ContextMenuStylesheet::native_default(),
            control_group: ControlGroupStylesheet::native_default(),
            dock_splitter: DockSplitterStylesheet::native_default(),
            resizable_panels: ResizablePanelsStylesheet::native_default(),
            overlay_window: OverlayWindowStylesheet::native_default(),
            sidebar: SidebarStylesheet::native_default(),
            tree_view: TreeViewStylesheet::native_default(),
            accordion: AccordionStylesheet::native_default(),
            table: TableStylesheet::native_default(),
            selector: SelectorStylesheet::native_default(),
            pager: PagerStylesheet::native_default(),
            popup_menu: PopupMenuStylesheet::native_default(),
            popover: PopoverStylesheet::native_default(),
            slide_panel: SlidePanelStylesheet::native_default(),
            radio_group: RadioGroupStylesheet::native_default(),
            autocomplete: AutocompleteStylesheet::native_default(),
            combobox: ComboBoxStylesheet::native_default(),
            search_selector: SearchSelectorStylesheet::native_default(),
            textfield: TextFieldStylesheet::native_default(),
            textarea: TextAreaStylesheet::native_default(),
            split_view: SplitViewStylesheet::native_default(),
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
                    label_baseline_sm: Some(0.5),
                    label_baseline_md: Some(1.0),
                    label_baseline_lg: Some(1.5),
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
        define_sdk_switch(&mut sheet);
        define_sdk_checkbox(&mut sheet);
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
                "radio_button",
                &[
                    AppForeground,
                    FocusRing,
                    FormInputBackground,
                    FormInputBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
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
            tokens::overlay(&mut document, authored);
        }
        Self::parse(&toml::to_string(&document)?)
    }

    /// Prepare an immutable SDK-only or custom-look snapshot. Rebuild tokens
    /// after editing the sheet; existing themes retain their captured styling.
    ///
    /// ```
    /// use xgpui_luma::{
    ///     color::GamutMapping,
    ///     controls::progress::{DefaultProgressTheme, ProgressTheme},
    ///     theme::{ControlSize, ThemeMode, stylesheet::CommonStylesheet},
    /// };
    /// let sheet = CommonStylesheet::sdk_default().clone().with(|sheet| {
    ///     sheet.progress.sizes.md.track_height = Some(9.0);
    /// });
    /// let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde)?;
    /// let theme = DefaultProgressTheme::new(tokens);
    /// assert_eq!(theme.resolve(true, ControlSize::Md).track_height, 9.0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn theme_tokens(
        &self,
        mode: crate::theme::ThemeMode,
        mapping: crate::color::GamutMapping,
    ) -> anyhow::Result<crate::theme::ThemeTokens> {
        let sheet = self.with_sdk_defaults();
        let mut tokens = crate::theme::ThemeTokens::from_source(sheet.tokens.source(mode), mapping)?;
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
            .resolve_geometry("", SharedGeometry::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ControlSize;
    use crate::theme::provenance::ResolvedMetric;

    #[test]
    fn external_adapter_preserves_the_complete_native_baseline() {
        assert_eq!(&CommonStylesheet::resolve_layers(&[]).unwrap(), CommonStylesheet::sdk_default());
        let native = CommonStylesheet::sdk_default().clone().with(|sheet| {
            sheet.slider.geometry.width = Some(0.0);
            sheet.font_features.0.insert("sidebar_section".into(), Vec::new());
            sheet.tokens.elevation.light.menu.layers.clear();
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
    fn slider_sizes_are_typed_and_custom_keys_remain_distinct() {
        use crate::controls::switch::SwitchScale;
        let mut stylesheet = CommonStylesheet::default();
        stylesheet.slider.sizes.sm.height = Some(19.0);
        stylesheet.slider.sizes.md.height = Some(28.0);
        let fallback = SliderGeometry::default();
        for (size, height) in [(ControlSize::Sm, 19.0), (ControlSize::Md, 28.0)] {
            let (geometry, sources) = stylesheet.slider.inspect_geometry(size, None, &fallback);
            assert_eq!(geometry.height, height);
            assert!(matches!(sources.height, crate::theme::provenance::MetricSource::Authored { key }
                if key == format!("common.slider.sizes.{}.height", size.stylesheet_key())));
        }
        for key in ["1", "4", "compact"] {
            let parsed = CommonStylesheet::parse(&format!("[common.slider.sizes.{key}]\nheight = 16")).unwrap();
            assert_eq!(parsed.slider.sizes.custom[key].height, Some(16.0));
            assert_eq!(parsed.slider.sizes.sm.height, None);
        }
        let stylesheet =
            CommonStylesheet::parse("[common.switch.sizes.\"1\"]\ngap=5\n[common.switch.sizes.compact]\ngap=7")
                .unwrap();
        let fallback = SwitchScale::compute(ControlSize::Sm, &Default::default(), 1.0);
        assert_eq!(stylesheet.switch.inspect_geometry("1", fallback, 1.0).gap.value_px, 5.0);
        assert_eq!(stylesheet.switch.inspect_geometry("compact", fallback, 1.0).gap.value_px, 7.0);
    }

    #[test]
    fn slider_geometry_validates_values_and_preserves_zero_and_thumb_selection() {
        use crate::controls::slider::SliderThumbSize;
        use crate::theme::ControlSize;
        for size in ["geometry", "sizes.1", "sizes.md", "sizes.custom"] {
            for field in ["width", "height", "track_height", "thumb_size"] {
                for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.slider.{size}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.slider.xl]\nwidth = 12").is_err());
        let stylesheet = CommonStylesheet::parse("[common.slider.geometry]\nwidth = 100\n[common.slider.sizes.md]\nwidth = 0\nthumb_size = 17\n[common.slider.sizes.lg]\nthumb_size = 23").unwrap();
        let fallback = SliderGeometry::sdk_default(
            &Default::default(),
            ControlSize::Md,
            Some(SliderThumbSize::Lg),
            CommonStylesheet::sdk_default(),
        );
        let (value, sources) = stylesheet.slider.inspect_geometry(ControlSize::Md, Some(ControlSize::Lg), &fallback);
        assert_eq!(value, stylesheet.slider.resolve_geometry(ControlSize::Md, Some(ControlSize::Lg), &fallback));
        assert_eq!(value.width, 0.0);
        assert_eq!(value.height, fallback.height);
        assert_eq!(value.thumb_size, 23.0);
        assert!(
            matches!(sources.thumb_size, crate::theme::provenance::MetricSource::Derived { note } if note.contains("instance thumb_size lg") && note.contains("common.slider.sizes.lg.thumb_size"))
        );
        let (value, _) = stylesheet.slider.inspect_geometry(ControlSize::Md, None, &fallback);
        assert_eq!(value.thumb_size, 17.0);
        assert_eq!(stylesheet.slider.inspect_geometry(ControlSize::Sm, None, &fallback).0.width, 100.0);
    }

    #[test]
    fn switch_geometry_rejects_invalid_lengths_and_sizes() {
        for size in ["geometry", "sizes.1", "sizes.md", "sizes.custom"] {
            for field in ["track_width", "track_height", "track_padding", "thumb_size", "gap"] {
                for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.switch.{size}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.switch.xl]\ngap = 2").is_err());
    }

    #[test]
    fn switch_size_precedence_zero_fallback_and_pixel_snapping() {
        use crate::theme::ControlSize;
        use crate::controls::switch::SwitchScale;
        let stylesheet = CommonStylesheet::parse("[common.switch.geometry]\ngap = 9.0\ntrack_padding = 1.25\n[common.switch.sizes.md]\ngap = 0\ntrack_width = 41.25").unwrap();
        let fallback = SwitchScale::compute(ControlSize::Md, &Default::default(), 2.0);
        let geometry = stylesheet.switch.inspect_geometry("md", fallback, 2.0);
        assert_eq!(geometry.gap.value_px, 0.0);
        assert_eq!(geometry.track_width.value_px, 41.5);
        assert_eq!(geometry.track_padding.value_px, 1.5);
        assert_eq!(geometry.thumb_size.value_px, fallback.thumb_size);
        assert!(
            matches!(geometry.gap.source, crate::theme::provenance::MetricSource::Authored { key } if key == "common.switch.sizes.md.gap")
        );
        assert_eq!(stylesheet.switch.inspect_geometry("sm", fallback, 2.0).gap.value_px, 9.0);
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
            target.size(ControlSize::Sm, |size| {
                size.width(260.0).dimensions(24.0, 4.0, 12.0);
            });
            target.size(ControlSize::Md, |size| {
                size.width(260.0).dimensions(32.0, 6.0, 16.0);
            });
            let mut layer = SliderStylesheet::default();
            layer.width(width).size(ControlSize::Md, |size| {
                size.width(180.0);
            });

            layer.apply_to(&mut target);

            assert_eq!(target.authored_geometry(ControlSize::Sm).width, Some(width));
            assert_eq!(target.authored_geometry(ControlSize::Md).width, Some(180.0));
            assert_eq!(target.authored_geometry(ControlSize::Sm).height, Some(24.0));
            assert_eq!(target.authored_geometry(ControlSize::Md).thumb_size, Some(16.0));
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
            .size(ControlSize::Sm, |m| {
                m.dimensions(16.0, 4.0, 12.0);
            })
            .size(ControlSize::Md, |m| {
                m.dimensions(20.0, 6.0, 16.0);
            });

        assert_eq!(sheet.slider.geometry.width, Some(120.0));
        assert_eq!(sheet.slider.geometry.height_floor_scale, Some(0.5));
        let size1 = &sheet.slider.sizes.sm;
        assert_eq!(size1.height, Some(16.0));
        assert_eq!(size1.track_height, Some(4.0));
        assert_eq!(size1.thumb_size, Some(12.0));

        let sheet = CommonStylesheet::default().with(|s| {
            s.slider.size(ControlSize::Lg, |m| {
                m.dimensions(24.0, 8.0, 20.0);
            });
        });
        assert_eq!(sheet.slider.sizes.lg.height, Some(24.0));
    }

    #[test]
    fn switch_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet.switch.track_padding(1.0).gap(8.0).size("1", |m| {
            m.dimensions(28.0, 16.0, 14.0);
        });

        assert_eq!(sheet.switch.geometry.track_padding, Some(1.0));
        assert_eq!(sheet.switch.geometry.gap, Some(8.0));
        let size1 = sheet.switch.sizes.get("1").unwrap();
        assert_eq!(size1.track_width, Some(28.0));
        assert_eq!(size1.track_height, Some(16.0));
        assert_eq!(size1.thumb_size, Some(14.0));

        let sheet = CommonStylesheet::default().with(|s| {
            s.switch.size("sm", |m| {
                m.scales(32.0 / 28.0, 13.0 / 28.0, 9.0 / 28.0);
            });
        });
        assert_eq!(sheet.switch.sizes.get("sm").unwrap().track_width_scale, Some(32.0 / 28.0));
    }

    #[test]
    fn checkbox_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet.checkbox.disabled_opacity(0.56).gap(8.0).size("1", |m| {
            m.indicator_size(14.0).glyph_size(9.0);
        });

        assert_eq!(sheet.checkbox.geometry.disabled_opacity, Some(0.56));
        assert_eq!(sheet.checkbox.geometry.gap, Some(8.0));
        let size1 = sheet.checkbox.sizes.get("1").unwrap();
        assert_eq!(size1.indicator_size, Some(14.0));
        assert_eq!(size1.glyph_size, Some(9.0));
    }
}
