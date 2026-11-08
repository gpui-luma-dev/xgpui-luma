use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    ContextMenuInspectMetrics, FloatingMenuInspectMetrics, OverlayWindowInspectMetrics, ShadcnInspect,
};

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

pub fn overlay_window_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    use xgpui_luma::controls::overlay_window::OverlayWindowMode;

    let size = control_size(size_id);
    let mode = match variant_id {
        "modal" => OverlayWindowMode::Modal,
        _ => OverlayWindowMode::Modeless,
    };
    overlay_window_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_overlay_window_metrics(size, mode),
    )
}

pub fn popup_menu_trigger_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let trigger_style = super::super::common::popup_menu_trigger_style(variant_id);
    let metrics = ShadcnInspect::new(look).inspect_popup_menu_metrics(trigger_style, size);
    let box_model = InspectBoxModelSnapshot::from_button_metrics(&metrics.trigger);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("trigger height", &metrics.trigger.height),
            ("trigger padding x", &metrics.trigger.padding_x),
            ("trigger padding y", &metrics.trigger.padding_y),
            ("trigger radius", &metrics.trigger.radius),
            ("trigger border width", &metrics.trigger.border_width),
            ("trigger focus ring width", &metrics.trigger.focus_ring_width),
            ("trigger focus ring offset", &metrics.trigger.focus_ring_offset),
        ]),
    )
}

pub fn context_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    context_menu_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_context_menu_metrics(size))
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

fn overlay_window_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &OverlayWindowInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.estimated_height.value_px,
        padding_x: metrics.padding.value_px,
        padding_y: metrics.padding.value_px,
        border_width: 1.0,
        gap: 0.0,
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
            ("min width", &metrics.min_width),
            ("max width", &metrics.max_width),
            ("estimated height", &metrics.estimated_height),
            ("radius", &metrics.radius),
            ("padding", &metrics.padding),
        ]),
    )
}

fn context_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ContextMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.target_padding_y.value_px * 2.0 + 16.0,
        padding_x: metrics.target_padding_x.value_px,
        padding_y: metrics.target_padding_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.target_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    let mut rows = metric_properties(&[
        ("target padding x", &metrics.target_padding_x),
        ("target padding y", &metrics.target_padding_y),
        ("target radius", &metrics.target_radius),
        ("target min width", &metrics.target_min_width),
    ]);
    rows.extend(floating_menu_metric_rows(&metrics.menu));
    layout_section(look, diagram_id, box_model, None, rows)
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
