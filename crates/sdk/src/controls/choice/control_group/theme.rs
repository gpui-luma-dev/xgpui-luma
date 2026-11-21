use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

#[derive(Clone, Debug)]
pub struct ControlGroupItemVisualContext {
    pub foreground: Hsla,
    pub background: Hsla,
    pub muted_foreground: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlGroupListLook {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    /// Vertical layout gap in logical pixels; independent of horizontal spacing.
    pub vertical_gap: f32,
    /// Shell border thickness in logical pixels; zero removes it.
    pub border_width: f32,
    /// Opacity of default disabled items; custom item templates own their opacity.
    pub disabled_opacity: f32,
}

impl ControlGroupListLook {
    /// Apply explicit shared shell geometry before consumer template modifiers.
    pub fn with_stylesheet(mut self, stylesheet: &crate::theme::stylesheet::ControlGroupStylesheet) -> Self {
        crate::theme::stylesheet::apply_common_geometry!(stylesheet, ControlGroupGeometry, self,
            [padding_x => padding_x, padding_y => padding_y, gap => gap, radius => radius, vertical_gap => vertical_gap,
             border_width => border_width, disabled_opacity => disabled_opacity]);
        self
    }
}

pub trait ControlGroupTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook;

    fn metrics(&self) -> MetricTokens;

    fn resolve_item_visual(
        &self,
        selected: bool,
        state: InteractionState,
        scale: &StandardBoxScale,
        row_height: f32,
        row_radius: f32,
    ) -> ControlGroupItemVisualContext {
        let palette = self.default_item_palette(selected, state, scale);
        ControlGroupItemVisualContext {
            foreground: palette.foreground,
            background: palette.background,
            muted_foreground: palette.muted_foreground,
            typography: palette.typography,
            font_family: palette.font_family.clone(),
            radius: row_radius,
            height: row_height.max(scale.height),
        }
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette;
}

#[derive(Clone, Debug)]
pub struct ControlGroupItemPalette {
    pub foreground: Hsla,
    pub background: Hsla,
    pub muted_foreground: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

pub fn default_control_group_theme() -> Arc<dyn ControlGroupTheme> {
    static THEME: OnceLock<Arc<dyn ControlGroupTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledControlGroupTheme::sdk())).clone()
}

crate::theme::stylesheet::geometry::geometry_family!(
    ControlGroupStylesheet,
    ControlGroupGeometryStylesheet,
    ControlGroupGeometry,
    ResolvedControlGroupGeometry,
    "control_group",
    [padding_x, padding_y, gap, radius, vertical_gap, border_width],
    unit[disabled_opacity]
);

impl ControlGroupStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ControlGroupGeometryStylesheet {
                vertical_gap: Some(8.0),
                border_width: Some(1.0),
                disabled_opacity: Some(0.56),
                padding_x: Some(6.0),
                padding_y: Some(4.0),
                gap: Some(6.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
