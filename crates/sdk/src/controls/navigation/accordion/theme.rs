use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::scales::snap_to_pixel;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccordionScale {
    /// Drag-handle padding in pixels.
    pub drag_handle_padding: f32,
    pub trigger_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub content_padding_y: f32,
    pub radius: f32,
    pub item_gap: f32,
    pub inner_gap: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl AccordionScale {
    /// The row layout for a theme that supplies only metrics: fixed sizes from the SDK's
    /// `accordion.lss`, the rest from `metrics`, so the theme keeps its own spacing.
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        let sdk = |property: &str| crate::style::sdk_number("accordion", property);
        // `.lss` pads content by a share of the control's vertical padding.
        let content_share = sdk("content_padding_scale");
        Self {
            drag_handle_padding: sdk("drag_handle_padding"),
            trigger_height: snap_to_pixel(metrics.control_height(), scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(), scale_factor),
            content_padding_y: snap_to_pixel(metrics.padding_y() * content_share, scale_factor),
            radius: metrics.radius(),
            item_gap: snap_to_pixel(sdk("item_gap"), scale_factor),
            inner_gap: snap_to_pixel(metrics.gap(), scale_factor),
            icon_size: sdk("icon_size"),
            chevron_size: sdk("chevron_size"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccordionPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub border_color: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    /// The trigger's opacity: the resolved `opacity`, which a look sets under `:disabled`.
    pub opacity: f32,
}

#[derive(Clone, Debug)]
pub struct AccordionContentPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
}

pub trait AccordionTheme: Send + Sync {
    /// Default reorder handle glyph, resolved independently of item icons.
    fn drag_handle_icon(&self) -> crate::infra::icon::IconSource {
        crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .icons
            .resolve(crate::theme::stylesheet::IconRole::DragHandle)
    }
    /// Look-owned help for the drag handle.
    fn drag_tooltip_theme(&self) -> Arc<dyn crate::controls::tooltip::TooltipTheme> {
        crate::controls::tooltip::default_tooltip_theme()
    }

    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette;
    fn resolve_content(&self, expanded: bool) -> AccordionContentPalette;
    fn metrics(&self) -> MetricTokens;

    /// Inherited post-drop position motion.
    /// Theme-owned preview and insertion feedback.
    fn drag_style(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        crate::infra::drag_drop::resolve_drag_preview(&crate::infra::drag_drop::sdk_drag_preview())
    }

    /// Theme-owned preview and insertion feedback.
    #[deprecated(note = "templates draw from the resolved style; use `drag_style` (#433)")]
    #[allow(deprecated)]
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_resolved(&self.drag_style())
    }

    /// Resolve pixel-snapped layout. Themes may share this result with inspection tools.
    /// Resolve against the selected theme snapshot; live looks refresh this on render.
    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        AccordionScale::compute(&self.metrics(), scale_factor)
    }
}

pub fn default_accordion_theme() -> Arc<dyn AccordionTheme> {
    static THEME: OnceLock<Arc<dyn AccordionTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledAccordionTheme::sdk())).clone()
}
