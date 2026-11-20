use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{FloatingMenuInspectMetrics, ShadcnInspect};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::{InspectLayoutSection, InspectPropertyRow};
use super::layout_section;

pub fn floating_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    floating_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
}

fn floating_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &FloatingMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: metrics.padding.value_px,
        border_width: 0.0,
        gap: metrics.item_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(look, diagram_id, box_model, None, floating_menu_metric_rows(metrics))
}

fn floating_menu_metric_rows(metrics: &FloatingMenuInspectMetrics) -> Vec<InspectPropertyRow> {
    metric_properties(&[
        ("menu radius", &metrics.radius),
        ("menu padding", &metrics.padding),
        ("menu min width", &metrics.min_width),
        ("menu item height", &metrics.item_height),
        ("menu item padding x", &metrics.item_padding_x),
        ("menu item gap", &metrics.item_gap),
        ("menu item icon size", &metrics.item_icon_size),
        ("menu item radius", &metrics.item_radius),
        ("menu submenu offset x", &metrics.submenu_offset_x),
    ])
}
