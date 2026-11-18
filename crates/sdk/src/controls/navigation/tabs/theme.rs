use crate::theme::provenance::ResolvedMetric;
use crate::theme::stylesheet::CommonStylesheet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct TabsListLook {
    /// List border thickness in pixels.
    pub border_width: f32,
    pub background: Option<Hsla>,
    pub border: Option<Hsla>,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TabsItemLook {
    /// Accessory glyph size and gap in pixels.
    pub accessory_size: f32,
    pub accessory_gap: f32,
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
    /// Selected-chip fill. `None` is transparent (line / underline tabs).
    pub background: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub height: f32,
    pub indicator_height: f32,
}

/// Full-list rule painted behind the active indicator without consuming layout space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsBaseline {
    pub color: Hsla,
    pub height: f32,
    pub inset_x: f32,
    /// Distance above the bottom of the list, in logical pixels.
    pub bottom: f32,
}

pub trait TabsTheme: Send + Sync {
    /// Appearance of native drag previews and insertion feedback.
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        let mut drag = crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default());
        let list = self.resolve_list(true);
        let item = self.resolve_item(true, Default::default());
        drag.background = list.background.or(item.background).unwrap_or(drag.background);
        drag.foreground = item.label_color;
        drag.valid_marker = item.indicator.unwrap_or(item.label_color);
        drag.invalid_marker =
            self.resolve_item(false, InteractionState { invalid: true, ..Default::default() }).label_color;
        drag.radius = list.radius;
        drag.typography = item.label_typography;
        drag.font_family = self.font_family();
        drag
    }
    fn resolve_list(&self, enabled: bool) -> TabsListLook;
    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook;
    fn font_family(&self) -> SharedString;
    /// Optional horizontal/vertical padding for an inset label background.
    /// Horizontal padding is taken from the trigger padding, preserving its width.
    fn content_padding(&self) -> Option<(f32, f32)> {
        None
    }
    /// Independent of the selected-tab indicator. Absent by default.
    fn baseline(&self, _enabled: bool) -> Option<TabsBaseline> {
        None
    }
    /// Override the active indicator's inset; `None` retains the item padding inset.
    fn indicator_inset(&self) -> Option<f32> {
        None
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTabsTheme {
    tokens: ThemeTokens,
}

pub fn default_tabs_theme() -> Arc<dyn TabsTheme> {
    static THEME: OnceLock<Arc<dyn TabsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTabsTheme::default())).clone()
}

impl DefaultTabsTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TabsTheme for DefaultTabsTheme {
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_tokens(&self.tokens)
    }
    fn resolve_list(&self, enabled: bool) -> TabsListLook {
        let palette = self.tokens.stylesheet.recipes.palette("tabs", &self.tokens.palette);
        let metrics = &self.tokens.metrics;

        TabsListLook {
            border_width: metrics.border_width,
            background: (!enabled).then_some(palette.state.disabled.background),
            border: None,
            radius: metrics.radius(),
            padding: self.tokens.stylesheet.tabs.geometry.list_padding.unwrap_or_default(),
            gap: self.tokens.stylesheet.tabs.geometry.list_gap.unwrap_or_else(|| metrics.gap()),
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsItemLook {
        let palette = self.tokens.stylesheet.recipes.palette("tabs", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let label_typography = typography.text.label;

        let active_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.selected.background,
            InteractionLayer::Default => palette.state.selected.background,
        };

        let (accessory_size, accessory_gap) = self.tokens.stylesheet.tabs.accessory_metrics("");
        TabsItemLook {
            accessory_size,
            accessory_gap,
            label_color: match (active, state.disabled) {
                (_, true) => palette.state.disabled.foreground,
                (true, false) => active_color,
                (false, false) => palette.app.foreground,
            },
            indicator: active.then_some(if state.focused && !state.disabled {
                palette.focus.ring
            } else {
                active_color
            }),
            background: None,
            label_typography,
            radius: metrics.radius(),
            padding_x: metrics.padding_x(),
            height: label_typography.line_height + metrics.padding_y() * 2.0,
            indicator_height: self.tokens.stylesheet.tabs.geometry.indicator_height.unwrap_or_default(),
        }
    }

    fn font_family(&self) -> SharedString {
        self.tokens.typography.font.sans.family.clone().into()
    }
}

#[cfg(test)]
mod state_audit_tests {
    use super::*;

    #[test]
    fn disabled_active_tab_does_not_paint_a_focus_indicator() {
        let tokens = ThemeTokens::default();
        let theme = DefaultTabsTheme::new(tokens.clone());
        let look = theme.resolve_item(
            true,
            InteractionState { disabled: true, focused: true, hovered: true, pressed: true, ..Default::default() },
        );
        assert_eq!(look.label_color, tokens.palette.state.disabled.foreground);
        assert_eq!(look.indicator, Some(tokens.palette.state.disabled.foreground));
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TabsStylesheet {
    pub geometry: TabsGeometryStylesheet,
    /// Variant names belong to each look; fields use the same common contract.
    pub variants: HashMap<String, TabsGeometryStylesheet>,
}

/// Common tab geometry in logical pixels. Missing fields retain the supplied
/// token/recipe fallback; variant values take precedence over general values.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TabsGeometryStylesheet {
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub accessory_size: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub accessory_gap: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub list_gap: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub list_padding: Option<f32>,
    #[serde(deserialize_with = "crate::theme::stylesheet::optional_length")]
    pub indicator_height: Option<f32>,
}

/// The same resolved geometry is consumed by paint paths and inspectors.
/// Concrete tab list and indicator dimensions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsGeometry {
    pub list_gap: f32,
    pub list_padding: f32,
    pub indicator_height: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedTabsGeometry {
    pub list_gap: ResolvedMetric,
    pub list_padding: ResolvedMetric,
    pub indicator_height: ResolvedMetric,
}

crate::theme::stylesheet::geometry::inherit_geometry!(
    TabsStylesheet,
    variants,
    [list_gap, list_padding, indicator_height, accessory_size, accessory_gap]
);

impl TabsStylesheet {
    /// Accessory glyph size and gap in pixels, with variant and SDK inheritance.
    pub fn accessory_metrics(&self, variant: &str) -> (f32, f32) {
        let baseline = &CommonStylesheet::sdk_default().tabs.geometry;
        let specific = self.variants.get(variant);
        (
            specific
                .and_then(|v| v.accessory_size)
                .or(self.geometry.accessory_size)
                .or(baseline.accessory_size)
                .unwrap_or_default(),
            specific
                .and_then(|v| v.accessory_gap)
                .or(self.geometry.accessory_gap)
                .or(baseline.accessory_gap)
                .unwrap_or_default(),
        )
    }
    /// Resolve concrete dimensions without constructing inspector metadata.
    pub fn resolve_geometry(&self, variant: &str, fallback: TabsGeometry) -> TabsGeometry {
        let specific = self.variants.get(variant);
        TabsGeometry {
            list_gap: specific.and_then(|v| v.list_gap).or(self.geometry.list_gap).unwrap_or(fallback.list_gap),
            list_padding: specific
                .and_then(|v| v.list_padding)
                .or(self.geometry.list_padding)
                .unwrap_or(fallback.list_padding),
            indicator_height: specific
                .and_then(|v| v.indicator_height)
                .or(self.geometry.indicator_height)
                .unwrap_or(fallback.indicator_height),
        }
    }

    /// Resolve variant, general, then supplied look-token/SDK fallback values.
    pub fn inspect_geometry(&self, variant: &str, fallback: ResolvedTabsGeometry) -> ResolvedTabsGeometry {
        let specific = self.variants.get(variant);
        let resolve = |field: &str, general: Option<f32>, specific: Option<f32>, fallback: ResolvedMetric| {
            if let Some(value) = specific {
                ResolvedMetric::authored(value, format!("common.tabs.variants.{variant}.{field}"))
            } else if let Some(value) = general {
                ResolvedMetric::authored(value, format!("common.tabs.geometry.{field}"))
            } else {
                fallback
            }
        };
        ResolvedTabsGeometry {
            list_gap: resolve("list_gap", self.geometry.list_gap, specific.and_then(|v| v.list_gap), fallback.list_gap),
            list_padding: resolve(
                "list_padding",
                self.geometry.list_padding,
                specific.and_then(|v| v.list_padding),
                fallback.list_padding,
            ),
            indicator_height: resolve(
                "indicator_height",
                self.geometry.indicator_height,
                specific.and_then(|v| v.indicator_height),
                fallback.indicator_height,
            ),
        }
    }
}

impl TabsStylesheet {
    /// Native SDK tab geometry.
    pub fn native_default() -> Self {
        TabsStylesheet {
            geometry: TabsGeometryStylesheet {
                accessory_size: Some(12.0),
                accessory_gap: Some(4.0),
                list_padding: Some(0.0),
                indicator_height: Some(2.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
