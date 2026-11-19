use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{ShadcnInspect, SwitchInspectMetrics};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::{choice_variant_style, control_size};
use super::super::occupation::{occupation_metric_properties, switch_occupation};
use super::super::provenance::{metric_properties, property_rows_from_metric_data};
use super::super::schema::InspectLayoutSection;
use super::button::button_layout_section;
use super::layout_section;

pub fn toggle_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let style = choice_variant_style(variant_id);
    let size = control_size(size_id);
    button_layout_section(
        look,
        diagram_id,
        style,
        ButtonFamilyRole::Toggle { selected: true },
        size,
        InteractionState::default(),
    )
}

pub fn switch_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_switch_metrics_for_style(choice_variant_style(variant_id), size);
    switch_metrics_layout_section(look, diagram_id, &metrics)
}
fn switch_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SwitchInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_switch_metrics(metrics);
    let occupation = switch_occupation(look, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("track width", &metrics.track_width),
        ("track height", &metrics.track_height),
        ("track padding", &metrics.track_padding),
        ("thumb size", &metrics.thumb_size),
        ("gap", &metrics.gap),
        ("track radius", &metrics.track_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}
