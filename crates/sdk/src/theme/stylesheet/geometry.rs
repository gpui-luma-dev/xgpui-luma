//! Typed common dimensions for remaining control families.

/// Selects one look-defined size key so SDK controls receive plain pixel geometry.
pub trait SelectSizeKey: Sized + Clone {
    /// Returns a copy whose general geometry includes the authored values for `key`.
    /// Families without size-keyed values return an unchanged copy.
    fn select_size_key(&self, key: &str) -> Self {
        let _ = key;
        self.clone()
    }
}

macro_rules! inherit_geometry {
    ($stylesheet:ident, sizes, [$($field:ident),+]) => {
        impl $stylesheet {
            pub(crate) fn with_baseline(mut self, baseline: &Self) -> Self {
                let mut inherited = baseline.sizes.clone();
                for value in inherited.values_mut() {
                    $(if self.geometry.$field.is_some() { value.$field = None; })+
                }
                for (key, authored) in self.sizes {
                    let value = inherited.get_or_default(&key);
                    $(value.$field = authored.$field.or(value.$field);)+
                }
                $(self.geometry.$field = self.geometry.$field.or(baseline.geometry.$field);)+
                self.sizes = inherited;
                self
            }
        }
    };

    ($stylesheet:ident, $axis:ident, [$($field:ident),+]) => {
        impl $stylesheet {
            pub(crate) fn with_baseline(mut self, baseline: &Self) -> Self {
                let mut inherited = baseline.$axis.clone();
                for value in inherited.values_mut() {
                    $(if self.geometry.$field.is_some() { value.$field = None; })+
                }
                for (key, authored) in self.$axis {
                    let value = inherited.entry(key).or_default();
                    $(value.$field = authored.$field.or(value.$field);)+
                }
                $(self.geometry.$field = self.geometry.$field.or(baseline.geometry.$field);)+
                self.$axis = inherited;
                self
            }
        }
    };
}
pub(crate) use inherit_geometry;

macro_rules! geometry_family {
    ($stylesheet:ident, $authored:ident, $fallback:ident, $resolved:ident, $family:literal, [$($field:ident),*] $(, unit [$($unit:ident),+])?) => {
        $crate::theme::stylesheet::geometry::geometry_family_impl!($stylesheet, $authored, $fallback, $resolved, $family,
            [$(($field, "crate::theme::stylesheet::optional_length"),)* $($(($unit, "crate::theme::stylesheet::optional_unit"),)+)?]);
    };
}

macro_rules! geometry_family_impl {
    ($stylesheet:ident, $authored:ident, $fallback:ident, $resolved:ident, $family:literal, [$(($field:ident, $validator:literal)),+ ,]) => {
        /// Common template dimensions. SDK controls use `geometry`; `sizes` holds look-defined keys.
        #[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, serde::Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $stylesheet {
            pub geometry: $authored,
            pub sizes: $crate::theme::ControlSizes<$authored>,
        }
        /// Optional logical pixel values. Authored zero overrides the fallback.
        #[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, serde::Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $authored {
            $(#[serde(deserialize_with = $validator)] pub $field: Option<f32>,)+
        }
        impl $authored {
            /// Fill missing fields without replacing explicit zero values.
            pub fn with_fallback(self, fallback: &Self) -> Self {
                Self { $($field: self.$field.or(fallback.$field),)+ }
            }
        }
        /// Concrete geometry used by controls.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $fallback { $(pub $field: f32,)+ }
        /// Inspector projection; rendering uses the concrete geometry above.
        #[derive(Clone, Debug)]
        pub struct $resolved { $(pub $field: $crate::theme::provenance::ResolvedMetric,)+ }
        $crate::theme::stylesheet::geometry::inherit_geometry!($stylesheet, sizes, [$($field),+]);
        #[cfg(test)]
        impl $stylesheet {
            pub(crate) fn validate_contract() {
                $(for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(crate::theme::stylesheet::CommonStylesheet::parse(&format!(
                        "[common.{}.sizes.\"custom\"]\n{} = {value}", $family, stringify!($field))).is_err());
                })+
                let authored = $authored { $($field: Some(0.0),)+ };
                let config = $stylesheet { geometry: $authored { $($field: Some(9.0),)+ }, sizes: $crate::theme::ControlSizes::from([("4".into(), authored)]) };
                let selected = config.inspect_geometry_for("4", $fallback { $($field: 7.0,)+ });
                let concrete = config.resolve_geometry_for("4", $fallback { $($field: 7.0,)+ });
                $(assert_eq!(concrete.$field, selected.$field.value_px);)+
                let general = config.inspect_geometry($fallback { $($field: 7.0,)+ });
                let missing = Self::default().inspect_geometry_for("4", $fallback { $($field: 7.0,)+ });
                $(assert_eq!(selected.$field.value_px, 0.0);
                assert_eq!(general.$field.value_px, 9.0);
                assert_eq!(missing.$field.value_px, 7.0);
                assert!(matches!(selected.$field.source, crate::theme::provenance::MetricSource::Authored { key }
                    if key == format!("common.{}.sizes.4.{}", $family, stringify!($field))));)+
                assert!(crate::theme::stylesheet::CommonStylesheet::parse(&format!("[common.{}.geometry]\nunknown = 1", $family)).is_err());
            }
        }
        impl $crate::theme::stylesheet::geometry::SelectSizeKey for $stylesheet {
            fn select_size_key(&self, key: &str) -> Self {
                Self { geometry: self.authored_geometry_for(key), sizes: self.sizes.clone() }
            }
        }
        impl $stylesheet {
            /// General authored geometry used by SDK controls.
            pub fn authored_geometry(&self) -> $authored {
                self.geometry.clone()
            }
            /// Authored geometry for a look-defined size key, over the general fields.
            pub fn authored_geometry_for(&self, key: &str) -> $authored {
                self.sizes.get(key).cloned().unwrap_or_default().with_fallback(&self.geometry)
            }
            /// Resolve concrete general values without constructing inspector metadata.
            pub fn resolve_geometry(&self, fallback: $fallback) -> $fallback {
                Self::concrete_geometry(self.authored_geometry(), fallback)
            }
            /// Resolve concrete values for a look-defined size key.
            pub fn resolve_geometry_for(&self, key: &str, fallback: $fallback) -> $fallback {
                Self::concrete_geometry(self.authored_geometry_for(key), fallback)
            }
            fn concrete_geometry(authored: $authored, fallback: $fallback) -> $fallback {
                $fallback { $($field: authored.$field.unwrap_or(fallback.$field),)+ }
            }
            /// General values override supplied fallbacks.
            pub fn inspect_geometry(&self, fallback: $fallback) -> $resolved {
                self.inspect_geometry_key(None, fallback)
            }
            /// Look-defined size values override general geometry, then supplied fallbacks.
            pub fn inspect_geometry_for(&self, key: &str, fallback: $fallback) -> $resolved {
                self.inspect_geometry_key(Some(key), fallback)
            }
            fn inspect_geometry_key(&self, key: Option<&str>, fallback: $fallback) -> $resolved {
                let specific = key.and_then(|key| self.sizes.get(key).map(|size| (key, size)));
                $resolved {
                    $($field: if let Some((key, value)) = specific.and_then(|(key, size)| size.$field.map(|value| (key, value))) {
                        $crate::theme::provenance::ResolvedMetric::authored(value, format!("common.{}.sizes.{key}.{}", $family, stringify!($field)))
                    } else if let Some(value) = self.geometry.$field {
                        $crate::theme::provenance::ResolvedMetric::authored(value, concat!("common.", $family, ".geometry.", stringify!($field)))
                    } else {
                        $crate::theme::provenance::ResolvedMetric::constant(fallback.$field, concat!($family, " ", stringify!($field), " fallback"))
                    },)+
                }
            }
        }
    };
}
pub(crate) use {geometry_family, geometry_family_impl};

pub use crate::controls::layout::split_view::theme::SplitViewGeometry;
pub use crate::controls::layout::dock_splitter::theme::DockSplitterGeometry;
pub use crate::controls::range::progress::theme::ProgressGeometry;
pub use crate::controls::overlay::tooltip::theme::{
    TooltipStylesheet, TooltipGeometryStylesheet, TooltipGeometry, ResolvedTooltipGeometry,
};
pub use crate::controls::navigation::stepper::theme::StepperGeometry;
pub use crate::controls::navigation::sidebar::theme::{
    SidebarStylesheet, SidebarGeometryStylesheet, SidebarGeometry, ResolvedSidebarGeometry,
};
pub use crate::controls::collection::table::theme::TableGeometry;
pub use crate::controls::overlay::popover_button::panel::{
    PopoverStylesheet, PopoverGeometryStylesheet, PopoverGeometry, ResolvedPopoverGeometry,
};
pub use crate::controls::overlay::slide_panel::{
    SlidePanelStylesheet, SlidePanelGeometryStylesheet, SlidePanelGeometry, ResolvedSlidePanelGeometry,
};
pub use crate::controls::choice::radio_group::RadioGroupGeometry;
pub use crate::controls::selection::autocomplete::theme::{
    AutocompleteStylesheet, AutocompleteGeometryStylesheet, AutocompleteGeometry, ResolvedAutocompleteGeometry,
};
pub use crate::controls::selection::combobox::{
    ComboBoxStylesheet, ComboBoxGeometryStylesheet, ComboBoxGeometry, ResolvedComboBoxGeometry,
};
pub use crate::controls::selection::search_selector::{
    SearchSelectorStylesheet, SearchSelectorGeometryStylesheet, SearchSelectorGeometry, ResolvedSearchSelectorGeometry,
};

geometry_family!(
    ShellStylesheet,
    ShellGeometryStylesheet,
    ShellGeometry,
    ResolvedShellGeometry,
    "shell",
    [
        height,
        macos_left_padding,
        left_padding,
        traffic_light_inset,
        icon_size,
        border_width,
        fullscreen_left_padding
    ]
);
geometry_family!(
    SharedStylesheet,
    SharedGeometryStylesheet,
    SharedGeometry,
    ResolvedSharedGeometry,
    "shared",
    [
        popup_viewport_height,
        popup_row_height,
        menu_row_height,
        menu_row_gap,
        compact_textfield_height,
        compact_textfield_padding_y,
        textarea_resize_icon_size,
        textarea_resize_inset,
        textarea_resize_hit_size,
        textarea_scrollbar_width,
        textarea_scrollbar_inset,
        scrollbar_width,
        composition_underline_width,
        label_baseline,
        drag_offset,
        drag_preview_width,
        drag_padding_x,
        drag_padding_y,
        drag_border_width,
        drag_marker_width,
        split_sidebar_width,
        split_sidebar_min_width,
        split_sidebar_max_width,
        selection_scrollbar_width,
        wide_middle_gap,
        wide_side_min_width,
        wide_side_preferred_width,
        wide_middle_min_width,
        wide_middle_max_width,
        form_field_gap,
        page_size_trigger_width,
        popup_icon_size
    ],
    unit[textarea_resize_opacity]
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_family_contracts_validate_every_field_and_preserve_precedence() {
        ShellStylesheet::validate_contract();
        SharedStylesheet::validate_contract();
        SidebarStylesheet::validate_contract();
        AutocompleteStylesheet::validate_contract();
        ComboBoxStylesheet::validate_contract();
        SearchSelectorStylesheet::validate_contract();
        PopoverStylesheet::validate_contract();
        SlidePanelStylesheet::validate_contract();
        TooltipStylesheet::validate_contract();
    }
}

impl SelectSizeKey for crate::theme::stylesheet::RowLimitsStylesheet {}
impl SelectSizeKey for crate::theme::stylesheet::PaletteRecipes {}
impl SelectSizeKey for crate::theme::stylesheet::FontFeaturesStylesheet {}
impl SelectSizeKey for crate::theme::stylesheet::IconsStylesheet {}

impl SelectSizeKey for crate::theme::stylesheet::SliderStylesheet {
    fn select_size_key(&self, key: &str) -> Self {
        Self { geometry: self.authored_geometry_for(key), sizes: self.sizes.clone() }
    }
}
