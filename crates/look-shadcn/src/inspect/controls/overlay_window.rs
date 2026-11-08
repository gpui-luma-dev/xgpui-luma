//! Inspect metadata for `overlay_window`.

use xgpui_luma::controls::overlay_window::OverlayWindowMode;
use xgpui_luma::theme::ControlSize;
use crate::{ColorSource, ResolvedColor, ResolvedMetric, ShadcnLook};

#[derive(Clone, Debug)]
pub struct OverlayWindowInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub backdrop: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct OverlayWindowInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding: ResolvedMetric,
    pub min_width: ResolvedMetric,
    pub max_width: ResolvedMetric,
    pub estimated_height: ResolvedMetric,
}

pub fn inspect_overlay_window_color_palette(
    look: &ShadcnLook,
    size: ControlSize,
    mode: OverlayWindowMode,
) -> OverlayWindowInspectPalette {
    let shell = crate::paint::overlay_window_look(look, size, mode);

    OverlayWindowInspectPalette {
        background: resolved_from_hsla(shell.background, ColorSource::CssVar { token: "popover".into() }),
        foreground: resolved_from_hsla(shell.foreground, ColorSource::CssVar { token: "popover-foreground".into() }),
        border: resolved_from_hsla(shell.border, ColorSource::CssVar { token: "border".into() }),
        backdrop: (mode == OverlayWindowMode::Modal).then(|| {
            resolved_from_hsla(shell.backdrop_background, ColorSource::Derived { note: "modal scrim".into() })
        }),
    }
}

pub fn inspect_overlay_window_metrics(
    look: &ShadcnLook,
    size: ControlSize,
    mode: OverlayWindowMode,
) -> OverlayWindowInspectMetrics {
    let table = crate::tables::metrics::resolve_overlay_window_metrics(look, size, mode);
    table.into()
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

impl From<crate::tables::metrics::OverlayWindowMetricTable> for OverlayWindowInspectMetrics {
    fn from(table: crate::tables::metrics::OverlayWindowMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding: table.padding,
            min_width: table.min_width,
            max_width: table.max_width,
            estimated_height: table.estimated_height,
        }
    }
}
