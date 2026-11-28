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

#[deprecated(note = "templates draw from the resolved style; use `ControlGroupTheme::resolve_list_style` (#433)")]
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
    /// A disabled default row's opacity: the `item` part's `:disabled { opacity }`. Custom item templates own
    /// their opacity.
    pub item_disabled_opacity: f32,
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

    #[deprecated(note = "templates draw from the resolved style; use `resolve_list_style` (#433)")]
    #[allow(deprecated)]
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook {
        ControlGroupListLook::from_resolved(&self.resolve_list_style(enabled), &self.resolve_list_style(false))
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
