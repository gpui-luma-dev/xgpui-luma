use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};
use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

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

pub trait ControlGroupTheme: Send + Sync {
    /// The `control_group` binding the frame is read from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("control_group")
    }

    /// The frame's resolved style, enabled or not (#433): the root, and the `item` part for a disabled row's
    /// opacity.
    fn resolve_list_style(&self, enabled: bool) -> Arc<Resolved> {
        super::styled::resolve_control_group(&self.style(), enabled)
    }

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
