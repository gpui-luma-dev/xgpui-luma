use std::collections::HashMap;

use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::controls::ShadcnButtonStyle;
use crate::elements::BadgeVariant;

/// Normalized selector key/value pairs for stylesheet rule matching.
#[derive(Clone, Debug, Default)]
pub struct SelectorMap {
    pub fields: HashMap<String, String>,
}

impl SelectorMap {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(key.into(), value.into());
    }
}

pub trait AsSelectorState {
    fn to_selector_map(&self) -> SelectorMap;
}

pub struct ButtonSelectorState {
    pub style: ShadcnButtonStyle,
    pub layer: InteractionLayer,
    pub mode: ThemeMode,
    pub selected: bool,
}

impl AsSelectorState for ButtonSelectorState {
    fn to_selector_map(&self) -> SelectorMap {
        let mut map = SelectorMap::default();
        map.insert("style", button_style_key(self.style));
        map.insert("layer", interaction_layer_key(self.layer));
        map.insert("mode", theme_mode_key(self.mode));
        map.insert("selected", self.selected.to_string());
        map
    }
}

pub fn button_style_key(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}

pub fn badge_variant_key(variant: BadgeVariant) -> &'static str {
    variant.as_str()
}

pub fn interaction_layer_key(layer: InteractionLayer) -> &'static str {
    match layer {
        InteractionLayer::Default => "default",
        InteractionLayer::Hovered => "hover",
        InteractionLayer::Pressed => "pressed",
        InteractionLayer::Disabled => "disabled",
    }
}

pub fn theme_mode_key(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}
