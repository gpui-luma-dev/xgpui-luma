use gpui::Hsla;
use gpui::black;
use xgpui_luma::controls::overlay_window::OverlayWindowLook;
use xgpui_luma::controls::overlay_window::OverlayWindowMode;
use xgpui_luma::theme::ControlSize;

use crate::look::ShadcnLook;
use crate::tokens::ShadcnFont;
use crate::tokens::ShadcnRadius;
use crate::tokens::ShadcnShadow;
use crate::tokens::ShadcnToken;

pub fn overlay_window_look(theme: &ShadcnLook, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
    let tokens = theme.mode_tokens();
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let chrome = theme.chrome();
    let background = theme.token_color("popover").unwrap_or_else(|_| theme.color(ShadcnToken::Card));
    let foreground =
        theme.token_color("popover-foreground").unwrap_or_else(|_| theme.color(ShadcnToken::CardForeground));
    let border = theme.token_color("border").unwrap_or(chrome.border);
    let baseline = xgpui_luma::controls::overlay_window::default_overlay_window_theme().resolve(size, mode);
    let mut look = OverlayWindowLook {
        viewport_padding: baseline.viewport_padding,
        window_margin: baseline.window_margin,
        drag_header_height: baseline.drag_header_height,

        border_width: baseline.border_width,
        background,
        border,
        foreground,
        backdrop_background: if mode == OverlayWindowMode::Modal {
            Hsla { a: 0.42, ..black() }
        } else {
            Hsla { a: 0.0, ..black() }
        },
        shadow: theme.shadow(ShadcnShadow::Lg),
        radius: theme.radius(ShadcnRadius::Lg),
        padding: metrics.padding_x(size),
        min_width: baseline.min_width,
        max_width: baseline.max_width,
        estimated_height: baseline.estimated_height,
        body: typography.text.body,
        font_family: theme.font(ShadcnFont::Sans),
    };
    let geometry = tokens.stylesheet().common.overlay_window.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::OverlayWindowGeometry {
            padding: look.padding,
            min_width: look.min_width,
            max_width: look.max_width,
            estimated_height: look.estimated_height,
            font_size: look.body.size,
            line_height: look.body.line_height,
            backdrop_opacity: look.backdrop_background.a,
            border_width: look.border_width,
            viewport_padding: look.viewport_padding,
            window_margin: look.window_margin,
            drag_header_height: look.drag_header_height,

            radius: look.radius,
        },
    );
    look.viewport_padding = geometry.viewport_padding.value_px;
    look.window_margin = geometry.window_margin.value_px;
    look.drag_header_height = geometry.drag_header_height.value_px;
    look.border_width = geometry.border_width.value_px;
    look.radius = geometry.radius.value_px;
    look.padding = geometry.padding.value_px;
    look.min_width = geometry.min_width.value_px;
    look.max_width = geometry.max_width.value_px;
    look.estimated_height = geometry.estimated_height.value_px;
    if mode == OverlayWindowMode::Modal {
        look.backdrop_background.a = geometry.backdrop_opacity.value_px;
    }
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut look.body,
        &geometry.font_size,
        &geometry.line_height,
    );

    look
}

/// Native common geometry owned by this control.
pub(crate) fn native_overlay_window_stylesheet() -> xgpui_luma::theme::stylesheet::OverlayWindowStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    OverlayWindowStylesheet {
        geometry: OverlayWindowGeometryStylesheet { backdrop_opacity: Some(0.42), ..Default::default() },
        sizes: xgpui_luma::theme::ControlSizes::from([
            ("sm".into(), OverlayWindowGeometryStylesheet { max_width: Some(420.0), ..Default::default() }),
            ("md".into(), OverlayWindowGeometryStylesheet::default()),
            ("lg".into(), OverlayWindowGeometryStylesheet { max_width: Some(620.0), ..Default::default() }),
        ]),
    }
}

#[derive(Clone, Debug)]
pub struct OverlayWindowMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub max_width: crate::ResolvedMetric,
    pub estimated_height: crate::ResolvedMetric,
}

pub fn resolve_overlay_window_metrics(
    look: &crate::ShadcnLook,
    size: xgpui_luma::theme::ControlSize,
    mode: xgpui_luma::controls::overlay_window::OverlayWindowMode,
) -> OverlayWindowMetricTable {
    use crate::tables::metrics::helpers::derived_metric;

    let shell = crate::paint::overlay_window_look(look, size, mode);
    let size_key = match size {
        xgpui_luma::theme::ControlSize::Sm => "sm",
        xgpui_luma::theme::ControlSize::Md => "md",
        xgpui_luma::theme::ControlSize::Lg => "lg",
    };

    let mut table = OverlayWindowMetricTable {
        radius: derived_metric("overlay radius = radius.lg", shell.radius),
        padding: derived_metric(format!("{size_key} overlay padding"), shell.padding),
        min_width: derived_metric(format!("{size_key} overlay min width"), shell.min_width),
        max_width: derived_metric(format!("{size_key} overlay max width"), shell.max_width),
        estimated_height: derived_metric(format!("{size_key} overlay estimated height"), shell.estimated_height),
    };
    let geometry = look.mode_tokens().stylesheet().common.overlay_window.inspect_geometry(
        size_key,
        xgpui_luma::theme::stylesheet::OverlayWindowGeometry {
            radius: table.radius.value_px,
            padding: table.padding.value_px,
            min_width: table.min_width.value_px,
            max_width: table.max_width.value_px,
            estimated_height: table.estimated_height.value_px,
            ..Default::default()
        },
    );
    table.radius = crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, table.radius);
    table.padding = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding, table.padding);
    table.min_width = crate::tables::metrics::helpers::prefer_shared_metric(geometry.min_width, table.min_width);
    table.max_width = crate::tables::metrics::helpers::prefer_shared_metric(geometry.max_width, table.max_width);
    table.estimated_height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.estimated_height, table.estimated_height);

    table
}
