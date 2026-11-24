use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::PopupMenuTriggerStyle;
use crate::controls::floating_menu::FloatingMenuLook;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

#[derive(Clone, Debug)]
pub struct PopupMenuPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub trigger_focus_border: gpui::Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Debug)]
pub struct PopupMenuLook {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub trigger_focus_border: gpui::Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_offset_y: f32,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PopupMenuTriggerMetrics {
    pub without_elevation: bool,
    pub icon_only: bool,
    pub trigger_radius_override: Option<f32>,
}

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> PopupMenuLook {
        let scale = StandardBoxScale::compute(&self.metrics(), scale_factor);
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale);
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledPopupMenuTheme::sdk())).clone()
}

pub fn compose_popup_menu_look(palette: &PopupMenuPalette, scale: &StandardBoxScale) -> PopupMenuLook {
    PopupMenuLook {
        trigger_focus_border: palette.trigger_focus_border,
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.icon_size,
        // The SDK `.lss` offset as a share of its gap, applied to this theme's gap.
        menu_offset_y: scale.gap * crate::style::sdk_menu_offset_share("popup_menu"),
        floating_menu: palette.floating_menu.clone(),
    }
}
