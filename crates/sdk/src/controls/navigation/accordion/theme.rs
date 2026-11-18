use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::scales::snap_to_pixel;
use crate::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccordionScale {
    /// Drag-handle padding in pixels and disabled-trigger opacity.
    pub drag_handle_padding: f32,
    pub disabled_opacity: f32,
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
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self::compute_with_stylesheet(metrics, scale_factor, crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }

    /// Derive the row layout from common appearance and shared metric tokens.
    pub fn compute_with_stylesheet(
        metrics: &MetricTokens,
        scale_factor: f32,
        stylesheet: &crate::theme::stylesheet::CommonStylesheet,
    ) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().accordion.authored_geometry();
        let geometry = stylesheet.accordion.authored_geometry().with_fallback(&baseline);
        Self {
            drag_handle_padding: geometry.drag_handle_padding.unwrap_or_default(),
            disabled_opacity: geometry.disabled_opacity.unwrap_or_default(),
            trigger_height: snap_to_pixel(
                geometry.trigger_height.unwrap_or_else(|| metrics.control_height()),
                scale_factor,
            ),
            padding_x: snap_to_pixel(geometry.padding_x.unwrap_or_else(|| metrics.padding_x()), scale_factor),
            padding_y: snap_to_pixel(geometry.padding_y.unwrap_or_else(|| metrics.padding_y()), scale_factor),
            content_padding_y: snap_to_pixel(
                geometry
                    .content_padding_y
                    .unwrap_or_else(|| metrics.padding_y() * geometry.content_padding_scale.unwrap_or_default()),
                scale_factor,
            ),
            radius: geometry.radius.unwrap_or_else(|| metrics.radius()),
            item_gap: snap_to_pixel(geometry.item_gap.unwrap_or_default(), scale_factor),
            inner_gap: snap_to_pixel(geometry.inner_gap.unwrap_or_else(|| metrics.gap()), scale_factor),
            icon_size: geometry.icon_size.unwrap_or_default(),
            chevron_size: geometry.chevron_size.unwrap_or_default(),
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
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default())
    }

    /// Resolve pixel-snapped layout. Themes may share this result with inspection tools.
    /// Resolve against the selected theme snapshot; live looks refresh this on render.
    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        AccordionScale::compute(&self.metrics(), scale_factor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAccordionTheme {
    tokens: ThemeTokens,
}

pub fn default_accordion_theme() -> Arc<dyn AccordionTheme> {
    static THEME: OnceLock<Arc<dyn AccordionTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultAccordionTheme::default())).clone()
}

impl DefaultAccordionTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AccordionTheme for DefaultAccordionTheme {
    fn drag_handle_icon(&self) -> crate::infra::icon::IconSource {
        self.tokens.stylesheet.icons.resolve(crate::theme::stylesheet::IconRole::DragHandle)
    }
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_tokens(&self.tokens)
    }
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette {
        let palette = self.tokens.stylesheet.recipes.palette("accordion", &self.tokens.palette);
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let background = match layer {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        let mut trigger_typography = typography.text.label;
        crate::controls::textfield::apply_control_typography(&mut trigger_typography, typography);

        AccordionPalette {
            background,
            foreground,
            border_color: palette.navigation.border,
            icon_color: foreground,
            chevron_color: palette.navigation.muted_foreground,
            typography: trigger_typography,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn resolve_content(&self, _expanded: bool) -> AccordionContentPalette {
        let palette = self.tokens.stylesheet.recipes.palette("accordion", &self.tokens.palette);

        AccordionContentPalette { background: None, foreground: palette.app.foreground }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn resolve_scale(&self, scale_factor: f32) -> AccordionScale {
        AccordionScale::compute_with_stylesheet(&self.tokens.metrics, scale_factor, &self.tokens.stylesheet)
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    AccordionStylesheet,
    AccordionGeometryStylesheet,
    AccordionGeometry,
    ResolvedAccordionGeometry,
    "accordion",
    [
        trigger_height,
        padding_x,
        padding_y,
        content_padding_y,
        content_padding_scale,
        radius,
        item_gap,
        inner_gap,
        icon_size,
        chevron_size,
        drag_handle_padding
    ],
    unit[disabled_opacity]
);

impl AccordionStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: AccordionGeometryStylesheet {
                drag_handle_padding: Some(2.0),
                disabled_opacity: Some(0.56),
                content_padding_scale: Some(1.5),
                item_gap: Some(0.0),
                icon_size: Some(16.0),
                chevron_size: Some(14.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
