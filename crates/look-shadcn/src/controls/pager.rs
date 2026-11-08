use gpui::FontWeight;
use xgpui_luma::controls::button_family::ButtonFamilyLook;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::pager::PagerLook;
use xgpui_luma::controls::pager::PagerStyle;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionState;

use crate::look::ShadcnLook;

pub fn pager_look(theme: &ShadcnLook, enabled: bool, style: PagerStyle) -> PagerLook {
    let tokens = theme.mode_tokens();
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = theme.typography_scale(crate::ShadcnTextSize::Xs);
    let key = style.stylesheet_key();
    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().pager.authored_geometry(key);
    let recipe = tokens.stylesheet().common.pager.authored_geometry(key).with_fallback(&baseline);

    let mut look = PagerLook {
        gap_icon_size: recipe.gap_icon_size.unwrap_or_default(),
        border_width: recipe.border_width.unwrap_or_default(),
        page_size_label_gap: recipe.page_size_label_gap.unwrap_or_default(),
        popup_offset_y: recipe.popup_offset_y.unwrap_or_default(),
        popup_gap: recipe.popup_gap.unwrap_or_default(),
        popup_padding: recipe.popup_padding.unwrap_or_default(),
        popup_item_padding_y: recipe.popup_item_padding_y.unwrap_or_default(),
        popup_item_radius_inset: recipe.popup_item_radius_inset.unwrap_or_default(),

        panel_background: if enabled {
            palette.panel_background
        } else {
            palette.disabled_background
        },
        border: palette.border_default,
        body_text: if enabled {
            palette.body_text
        } else {
            palette.disabled_foreground
        },
        muted_text: if enabled {
            palette.app_muted_foreground
        } else {
            palette.disabled_foreground
        },
        selected_background: palette.selected_background,
        selected_foreground: palette.selected_foreground,
        hover_background: palette.accent_background,
        shadow: vec![],
        typography,
        button_size: metrics.control_height(ControlSize::Sm),
        button_min_width: metrics.control_height(ControlSize::Sm),
        control_height: metrics.control_height(ControlSize::Sm),
        radius: metrics.radius(ControlSize::Sm),
        padding_x: metrics.padding_x(ControlSize::Sm),
        padding_y: metrics.padding_y(ControlSize::Sm),
        gap: metrics.gap(ControlSize::Sm),
        group_gap: metrics.gap(ControlSize::Sm),
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
    };
    let geometry = tokens.stylesheet().common.pager.inspect_geometry(
        key,
        xgpui_luma::theme::stylesheet::PagerGeometry {
            button_size: look.button_size,
            button_min_width: look.button_min_width,
            control_height: look.control_height,
            padding_x: look.padding_x,
            padding_y: look.padding_y,
            gap: look.gap,
            group_gap: look.group_gap,
            font_size: look.typography.size,
            line_height: look.typography.line_height,
            radius: look.radius,
            disabled_opacity: look.disabled_opacity,
            ..Default::default()
        },
    );
    look.button_size = geometry.button_size.value_px;
    look.button_min_width = geometry.button_min_width.value_px;
    look.control_height = geometry.control_height.value_px;
    look.padding_x = geometry.padding_x.value_px;
    look.padding_y = geometry.padding_y.value_px;
    look.gap = geometry.gap.value_px;
    look.group_gap = geometry.group_gap.value_px;
    look.radius = geometry.radius.value_px;
    look.disabled_opacity = geometry.disabled_opacity.value_px;
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut look.typography,
        &geometry.font_size,
        &geometry.line_height,
    );

    look
}

pub fn pager_button_look(
    theme: &ShadcnLook,
    pager_look: &PagerLook,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyLook {
    let mut look = theme.resolve_outline_button(role, ControlSize::Sm, state);
    look.height = pager_look.button_size;
    look.radius = pager_look.radius;
    look.gap = pager_look.gap;
    look.typography.size = pager_look.typography.size;
    look.typography.line_height = pager_look.typography.line_height;
    look.typography.weight = if matches!(role, ButtonFamilyRole::Toggle { selected: true }) {
        FontWeight::SEMIBOLD
    } else {
        pager_look.typography.weight
    };

    if matches!(role, ButtonFamilyRole::Icon) {
        look.padding_x = 0.0;
        look.padding_y = 0.0;
    } else {
        look.padding_x = pager_look.padding_x;
        look.padding_y = 0.0;
    }

    look
}

/// Native common geometry owned by this control.
pub(crate) fn native_pager_stylesheet() -> xgpui_luma::theme::stylesheet::PagerStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    PagerStylesheet {
        geometry: PagerGeometryStylesheet {
            padding_x: Some(8.0),
            padding_y: Some(8.0),
            gap: Some(4.0),
            ..Default::default()
        },
        sizes: xgpui_luma::theme::ControlSizes::from([
            (
                "minimal".into(),
                PagerGeometryStylesheet {
                    button_size: Some(28.0),
                    button_min_width: Some(28.0),
                    group_gap: Some(8.0),
                    ..Default::default()
                },
            ),
            (
                "minimal-edge".into(),
                PagerGeometryStylesheet { button_min_width: Some(32.0), group_gap: Some(16.0), ..Default::default() },
            ),
            (
                "numeric".into(),
                PagerGeometryStylesheet { button_min_width: Some(32.0), group_gap: Some(16.0), ..Default::default() },
            ),
        ]),
    }
}

#[derive(Clone, Debug)]
pub struct PagerMetricTable {
    pub control_height: crate::ResolvedMetric,
    pub button_size: crate::ResolvedMetric,
    pub button_min_width: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub gap: crate::ResolvedMetric,
    pub group_gap: crate::ResolvedMetric,
    pub font_family: crate::ResolvedTypography,
    pub reserved_shadow_extent: crate::ResolvedMetric,
}

pub fn resolve_pager_metrics(
    look: &crate::ShadcnLook,
    style: xgpui_luma::controls::pager::PagerStyle,
) -> PagerMetricTable {
    use crate::tables::metrics::helpers::derived_metric;

    let pager = crate::paint::pager_look(look, true, style);
    let style_label = pager_style_label(style);
    let button = look.resolve_outline_button(
        xgpui_luma::controls::button_family::ButtonFamilyRole::Toggle { selected: false },
        xgpui_luma::theme::ControlSize::Sm,
        xgpui_luma::theme::InteractionState::default(),
    );
    let reserved_shadow_extent =
        xgpui_luma::infra::shadow_layout::shadow_projection_extent(button.shadow.as_deref(), 1.0, true);

    let mut table = PagerMetricTable {
        control_height: derived_metric(format!("{style_label} page-size trigger height"), pager.control_height),
        button_size: derived_metric(format!("{style_label} button size"), pager.button_size),
        button_min_width: derived_metric(format!("{style_label} button min width"), pager.button_min_width),
        radius: derived_metric(format!("{style_label} radius = radius.sm"), pager.radius),
        padding_x: derived_metric(format!("{style_label} shell padding x"), pager.padding_x),
        padding_y: derived_metric(format!("{style_label} shell padding y"), pager.padding_y),
        gap: derived_metric(format!("{style_label} item gap"), pager.gap),
        group_gap: derived_metric(format!("{style_label} group gap"), pager.group_gap),
        font_family: pager_font_family(look),
        reserved_shadow_extent: crate::ResolvedMetric {
            value_px: reserved_shadow_extent,
            source: crate::MetricSource::Derived { note: "button outline shadow projection extent".into() },
        },
    };
    let geometry = look.mode_tokens().stylesheet().common.pager.inspect_geometry(
        style_label,
        xgpui_luma::theme::stylesheet::PagerGeometry {
            control_height: table.control_height.value_px,
            button_size: table.button_size.value_px,
            button_min_width: table.button_min_width.value_px,
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
            group_gap: table.group_gap.value_px,
            radius: table.radius.value_px,
            ..Default::default()
        },
    );
    table.control_height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.control_height, table.control_height);
    table.button_size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.button_size, table.button_size);
    table.button_min_width =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.button_min_width, table.button_min_width);
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table.group_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.group_gap, table.group_gap);
    table.radius = crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, table.radius);

    table
}

fn pager_font_family(look: &crate::ShadcnLook) -> crate::ResolvedTypography {
    let family = look.mode_tokens().typography.font.sans.family.clone();
    if look.mode_tokens().catalog.get("font-sans").is_some() {
        crate::ResolvedTypography {
            value: family,
            source: crate::TypographySource::CssVar { token: "font-sans".into() },
        }
    } else {
        crate::ResolvedTypography {
            value: family,
            source: crate::TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() },
        }
    }
}

fn pager_style_label(style: xgpui_luma::controls::pager::PagerStyle) -> &'static str {
    match style {
        xgpui_luma::controls::pager::PagerStyle::Minimal => "minimal",
        xgpui_luma::controls::pager::PagerStyle::MinimalEdge => "minimal-edge",
        xgpui_luma::controls::pager::PagerStyle::Numeric => "numeric",
    }
}
