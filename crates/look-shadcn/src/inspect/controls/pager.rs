//! Inspect metadata for `pager`.

use xgpui_luma::controls::pager::PagerStyle;
use crate::{ColorSource, ResolvedColor, ResolvedMetric, ResolvedTypography, ShadcnLook};

#[derive(Clone, Debug)]
pub struct PagerShellInspectPalette {
    pub panel_background: ResolvedColor,
    pub border: ResolvedColor,
    pub body_text: ResolvedColor,
    pub muted_text: ResolvedColor,
    pub selected_background: ResolvedColor,
    pub selected_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct PagerInspectMetrics {
    pub control_height: ResolvedMetric,
    pub button_size: ResolvedMetric,
    pub button_min_width: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub group_gap: ResolvedMetric,
    pub font_family: ResolvedTypography,
    pub reserved_shadow_extent: ResolvedMetric,
}

pub fn inspect_pager_shell_color_palette(
    look: &ShadcnLook,
    enabled: bool,
    style: PagerStyle,
) -> PagerShellInspectPalette {
    let pager = crate::paint::pager_look(look, enabled, style);

    PagerShellInspectPalette {
        panel_background: resolved_shell_background(enabled, pager.panel_background),
        border: resolved_from_hsla(pager.border, ColorSource::CssVar { token: "border".into() }),
        body_text: resolved_shell_foreground(enabled, pager.body_text),
        muted_text: resolved_shell_muted(enabled, pager.muted_text),
        selected_background: resolved_from_hsla(
            pager.selected_background,
            ColorSource::CssVar { token: "accent".into() },
        ),
        selected_foreground: resolved_from_hsla(
            pager.selected_foreground,
            ColorSource::CssVar { token: "accent-foreground".into() },
        ),
    }
}

pub fn inspect_pager_metrics(look: &ShadcnLook, style: PagerStyle) -> PagerInspectMetrics {
    let table = crate::tables::metrics::resolve_pager_metrics(look, style);
    table.into()
}

fn resolved_shell_background(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "card".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "card · disabled".into() })
    }
}

fn resolved_shell_foreground(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "foreground".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "foreground · disabled".into() })
    }
}

fn resolved_shell_muted(enabled: bool, value: gpui::Hsla) -> ResolvedColor {
    if enabled {
        resolved_from_hsla(value, ColorSource::CssVar { token: "muted-foreground".into() })
    } else {
        resolved_from_hsla(value, ColorSource::Derived { note: "muted-foreground · disabled".into() })
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

impl From<crate::tables::metrics::PagerMetricTable> for PagerInspectMetrics {
    fn from(table: crate::tables::metrics::PagerMetricTable) -> Self {
        Self {
            control_height: table.control_height,
            button_size: table.button_size,
            button_min_width: table.button_min_width,
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            group_gap: table.group_gap,
            font_family: table.font_family,
            reserved_shadow_extent: table.reserved_shadow_extent,
        }
    }
}
