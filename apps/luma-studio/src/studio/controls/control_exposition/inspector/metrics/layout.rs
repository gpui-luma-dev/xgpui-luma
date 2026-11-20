use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{BadgeInspectMetrics, ShadcnInspect};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::InspectLayoutSection;
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
