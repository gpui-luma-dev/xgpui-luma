use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use xgpui_luma_look_shadcn::inspect::{ButtonInspectMetrics, ShadcnInspect};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::occupation::{button_family_occupation, occupation_metric_properties};
use super::super::provenance::{metric_properties, property_rows_from_metric_data};
use super::super::schema::InspectLayoutSection;
use super::layout_section;

pub fn button_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> InspectLayoutSection {
    let metrics = ShadcnInspect::new(look).inspect_button_metrics(style, role, size, state);
    button_metrics_layout_section(look, diagram_id, style, role, size, state, &metrics)
}
fn button_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
    metrics: &ButtonInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_button_metrics(metrics);
    let occupation = button_family_occupation(look, style, role, size, state, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("height", &metrics.height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
        ("radius", &metrics.radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}
