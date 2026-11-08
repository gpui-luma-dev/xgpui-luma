use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    BadgeInspectMetrics, PagerInspectMetrics, ResizablePanelsInspectMetrics, ShadcnInspect, SplitViewInspectMetrics,
    ToolbarInspectMetrics,
};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::{InspectLayoutSection, InspectPropertyRow};
use super::layout_section;

pub fn badge_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let variant = super::super::common::badge_variant(variant_id);
    badge_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_badge_metrics(variant, size))
}
pub fn toolbar_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    toolbar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_toolbar_metrics(control_size(size_id)),
    )
}

pub fn pager_shell_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    let style = super::super::common::pager_style(variant_id);
    pager_shell_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_pager_metrics(style))
}

pub fn pager_button_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    let style = xgpui_luma::controls::pager::PagerStyle::Numeric;
    pager_button_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_pager_metrics(style))
}
pub fn resizable_panels_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    resizable_panels_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_resizable_panels_metrics(super::super::common::resize_handle_size(size_id)),
    )
}

pub fn split_view_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    split_view_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_split_view_metrics())
}
fn badge_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &BadgeInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_badge_metrics(metrics);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("min height", &metrics.min_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("gap", &metrics.gap),
            ("icon size", &metrics.icon_size),
            ("radius", &metrics.radius),
        ]),
    )
}
fn toolbar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ToolbarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.separator_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.gap.value_px,
        radius: metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("gap", &metrics.gap),
            ("radius", &metrics.radius),
            ("separator height", &metrics.separator_height),
        ]),
    )
}

fn pager_shell_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &PagerInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.control_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.group_gap.value_px,
        radius: metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    let mut rows = metric_properties(&[
        ("control height", &metrics.control_height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("item gap", &metrics.gap),
        ("group gap", &metrics.group_gap),
        ("radius", &metrics.radius),
    ]);
    rows.push(InspectPropertyRow::new(
        "requested font family",
        metrics.font_family.value.as_str(),
        xgpui_luma_look_shadcn::inspect::format_inspect_typography_source(&metrics.font_family.source),
    ));
    rows.push(super::super::provenance::metric_row("shadow projection extent", &metrics.reserved_shadow_extent));

    layout_section(look, diagram_id, box_model, None, rows)
}

fn pager_button_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &PagerInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.button_size.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.gap.value_px,
        radius: metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    let mut rows = metric_properties(&[
        ("button size", &metrics.button_size),
        ("button min width", &metrics.button_min_width),
        ("padding x", &metrics.padding_x),
        ("gap", &metrics.gap),
        ("radius", &metrics.radius),
    ]);
    rows.push(InspectPropertyRow::new(
        "requested font family",
        metrics.font_family.value.as_str(),
        xgpui_luma_look_shadcn::inspect::format_inspect_typography_source(&metrics.font_family.source),
    ));
    rows.push(super::super::provenance::metric_row("shadow projection extent", &metrics.reserved_shadow_extent));
    rows.push(InspectPropertyRow::new(
        "width policy",
        "min-width + content",
        "pager/template.rs · page and labeled buttons",
    ));

    layout_section(look, diagram_id, box_model, None, rows)
}
fn resizable_panels_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ResizablePanelsInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.hit_target_px.value_px,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: 0.0,
        gap: 0.0,
        radius: 0.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("lane", &metrics.lane_px),
            ("hit target", &metrics.hit_target_px),
            ("grip cross axis", &metrics.grip_cross_axis_px),
            ("grip main axis", &metrics.grip_main_axis_px),
        ]),
    )
}

fn split_view_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SplitViewInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.separator_hitbox_width.value_px,
        padding_x: 0.0,
        padding_y: metrics.separator_cue_inset_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.separator_cue_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("separator hitbox width", &metrics.separator_hitbox_width),
            ("separator cue width", &metrics.separator_cue_width),
            ("separator cue hovered width", &metrics.separator_cue_hovered_width),
            ("separator cue radius", &metrics.separator_cue_radius),
            ("separator cue inset y", &metrics.separator_cue_inset_y),
        ]),
    )
}
