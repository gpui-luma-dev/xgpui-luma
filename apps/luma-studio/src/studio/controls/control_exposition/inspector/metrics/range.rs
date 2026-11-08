use xgpui_luma::controls::scrollbar::ScrollbarOrientation;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    ProgressInspectMetrics, ScrollbarInspectMetrics, ShadcnInspect, SliderInspectMetrics, StepperInspectMetrics,
};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::provenance::metric_properties;
use super::super::schema::InspectLayoutSection;
use super::layout_section;

pub fn progress_circular_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ProgressInspectMetrics,
) -> InspectLayoutSection {
    let size = metrics.size.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: size,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: metrics.stroke_width.value_px,
        gap: 0.0,
        radius: size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("size", &metrics.size), ("stroke width", &metrics.stroke_width)]),
    )
}

pub fn progress_linear_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ProgressInspectMetrics,
) -> InspectLayoutSection {
    let track_height = metrics.track_height.value_px;
    let thumb_size = metrics.thumb_size.value_px;
    let cross_extent = thumb_size.max(track_height);
    let track_inset = (cross_extent - track_height) / 2.0;

    let box_model = InspectBoxModelSnapshot {
        height: cross_extent,
        padding_x: 0.0,
        padding_y: track_inset,
        border_width: track_height,
        gap: 0.0,
        radius: track_height / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("track height", &metrics.track_height), ("thumb size", &metrics.thumb_size)]),
    )
}

pub fn stepper_horizontal_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &StepperInspectMetrics,
) -> InspectLayoutSection {
    let badge_size = metrics.step_badge_size.value_px;
    let track_thickness = metrics.track_thickness.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: badge_size,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: track_thickness,
        gap: 0.0,
        radius: badge_size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("badge size", &metrics.step_badge_size), ("track thickness", &metrics.track_thickness)]),
    )
}

pub fn stepper_vertical_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &StepperInspectMetrics,
) -> InspectLayoutSection {
    let badge_size = metrics.step_badge_size.value_px;
    let track_thickness = metrics.track_thickness.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: badge_size * 2.0 + 24.0,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: track_thickness,
        gap: 24.0,
        radius: badge_size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("badge size", &metrics.step_badge_size), ("track thickness", &metrics.track_thickness)]),
    )
}

pub fn slider_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    slider_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_slider_metrics())
}

pub fn scrollbar_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    let style = super::super::common::scrollbar_style(variant_id);
    let orientation = ScrollbarOrientation::Vertical;
    scrollbar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_scrollbar_metrics(orientation, style),
    )
}
fn slider_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SliderInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.height.value_px,
        padding_x: 0.0,
        padding_y: ((metrics.height.value_px - metrics.track_height.value_px) / 2.0).max(0.0),
        border_width: 0.0,
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
            ("width", &metrics.width),
            ("height", &metrics.height),
            ("track height", &metrics.track_height),
            ("thumb size", &metrics.thumb_size),
            ("radius", &metrics.radius),
        ]),
    )
}

fn scrollbar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ScrollbarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.thumb_thickness.value_px,
        padding_x: ((metrics.track_thickness.value_px - metrics.thumb_thickness.value_px) / 2.0).max(0.0),
        padding_y: 0.0,
        border_width: 0.0,
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
            ("length", &metrics.length),
            ("thickness", &metrics.thickness),
            ("track thickness", &metrics.track_thickness),
            ("thumb thickness", &metrics.thumb_thickness),
            ("min thumb length", &metrics.min_thumb_length),
            ("radius", &metrics.radius),
        ]),
    )
}
