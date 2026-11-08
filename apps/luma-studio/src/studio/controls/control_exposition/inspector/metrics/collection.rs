use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    AccordionInspectMetrics, ShadcnInspect, SidebarInspectMetrics, TableInspectMetrics, TabsInspectMetrics,
    TreeViewInspectMetrics,
};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::{InspectLayoutSection, InspectPropertyRow};
use super::layout_section;

pub fn listbox_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    use super::super::super::listbox::{horizontal, vertical, ITEM_CONTENT_GAP, VERTICAL_LIST_WIDTH};
    let layout = if variant_id == "horizontal" {
        horizontal::LAYOUT
    } else {
        vertical::LAYOUT
    };
    let source = "Studio composition · ListBoxSampleLayout";
    let box_model = InspectBoxModelSnapshot {
        height: layout.item_height,
        padding_x: layout.item_padding,
        padding_y: 0.0,
        border_width: layout.border,
        gap: ITEM_CONTENT_GAP,
        radius: layout.radius,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    let mut metrics = vec![
        InspectPropertyRow::new("item height", format!("{}px", layout.item_height), source),
        InspectPropertyRow::new("item spacing", format!("{}px", layout.spacing), source),
        InspectPropertyRow::new("viewport height (content)", format!("{}px", layout.viewport_height), source),
        InspectPropertyRow::new("viewport inset x", format!("{}px each side", layout.inset_x), source),
        InspectPropertyRow::new("viewport inset y", format!("{}px each side", layout.inset_y), source),
        InspectPropertyRow::new("radius", format!("{}px", layout.radius), source),
    ];
    if variant_id == "horizontal" {
        metrics.push(InspectPropertyRow::new(
            "card width",
            format!("{}px (fixed; does not shrink)", horizontal::CARD_WIDTH),
            source,
        ));
    } else {
        metrics.push(InspectPropertyRow::new(
            "list width (including insets and border)",
            format!("{VERTICAL_LIST_WIDTH}px (fixed; does not shrink)"),
            "Studio composition · VERTICAL_LIST_WIDTH",
        ));
        metrics.push(InspectPropertyRow::new(
            "visible rows",
            (layout.viewport_height / (layout.item_height + layout.spacing)).ceil().to_string(),
            source,
        ));
    }
    layout_section(look, diagram_id, box_model, None, metrics)
}

pub fn table_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    if variant_id == "grid-cell" {
        return table_grid_cell_layout_section(look, diagram_id, control_size(size_id));
    }
    table_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_table_metrics(control_size(size_id)),
    )
}

pub fn tree_view_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    tree_view_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_tree_view_metrics(control_size(size_id)),
    )
}

pub fn sidebar_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    sidebar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_sidebar_metrics(control_size(size_id)),
    )
}

pub fn tabs_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    tabs_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_tabs_metrics(control_size(size_id)))
}
pub fn accordion_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    size_id: &str,
    scale_factor: f32,
) -> InspectLayoutSection {
    accordion_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_accordion_metrics_at_scale(control_size(size_id), scale_factor),
    )
}
fn table_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TableInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.row_min_height.value_px,
        padding_x: metrics.row_padding_x.value_px,
        padding_y: metrics.row_padding_y.value_px,
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
            ("radius", &metrics.radius),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("row min height", &metrics.row_min_height),
            ("row padding x", &metrics.row_padding_x),
            ("row padding y", &metrics.row_padding_y),
        ]),
    )
}

fn compact_table_row_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 28.0,
        ControlSize::Md => 32.0,
        ControlSize::Lg => 36.0,
    }
}

fn table_grid_cell_layout_section(look: &ShadcnLook, diagram_id: &str, size: ControlSize) -> InspectLayoutSection {
    const GRID_CONTROL_COLUMN_WIDTH: f32 = 48.0;
    let row_metrics = ShadcnInspect::new(look).inspect_table_metrics(size);
    let checkbox_metrics = ShadcnInspect::new(look).inspect_checkbox_metrics(ControlSize::Sm);
    let compact_row_height = compact_table_row_height(size);
    let inner_row_height = compact_row_height - (row_metrics.row_padding_y.value_px * 2.0);

    let box_model = InspectBoxModelSnapshot {
        height: compact_row_height,
        padding_x: 0.0,
        padding_y: row_metrics.row_padding_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: row_metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };

    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        vec![
            InspectPropertyRow::new(
                "grid cell width",
                format!("{GRID_CONTROL_COLUMN_WIDTH:.0}px"),
                "TableColumn::fixed_control",
            ),
            InspectPropertyRow::new(
                "compact row height",
                format!("{compact_row_height:.0}px"),
                "visible_row_height override (data-table rows)",
            ),
            InspectPropertyRow::new(
                "row padding y",
                format!("{:.1}px", row_metrics.row_padding_y.value_px),
                "applied inside fixed row height",
            ),
            InspectPropertyRow::new(
                "inner content height",
                format!("{inner_row_height:.1}px"),
                "compact row height − (2 × row padding y)",
            ),
            InspectPropertyRow::new("embedded checkbox size", "sm · indicator_only", "payments card pattern"),
            InspectPropertyRow::new(
                "checkbox indicator size",
                format!("{:.1}px", checkbox_metrics.indicator_size.value_px),
                "CheckboxScale at Sm",
            ),
        ],
    )
}

fn tree_view_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TreeViewInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.row_height.value_px,
        padding_x: metrics.base_padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.inner_gap.value_px,
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
            ("row height", &metrics.row_height),
            ("base padding x", &metrics.base_padding_x),
            ("indentation width", &metrics.indentation_width),
            ("inner gap", &metrics.inner_gap),
            ("radius", &metrics.radius),
            ("icon size", &metrics.icon_size),
            ("chevron size", &metrics.chevron_size),
        ]),
    )
}

fn sidebar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SidebarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.item_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("section height", &metrics.section_height),
            ("item height", &metrics.item_height),
            ("item padding x", &metrics.item_padding_x),
            ("item radius", &metrics.item_radius),
            ("item icon size", &metrics.item_icon_size),
        ]),
    )
}
fn tabs_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TabsInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: metrics.list_padding.value_px,
        border_width: 0.0,
        gap: metrics.list_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("list radius", &metrics.list_radius),
            ("list gap", &metrics.list_gap),
            ("list padding", &metrics.list_padding),
            ("item padding x", &metrics.item_padding_x),
            ("item height", &metrics.item_height),
            ("item radius", &metrics.item_radius),
            ("indicator height", &metrics.indicator_height),
        ]),
    )
}
fn accordion_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &AccordionInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.trigger_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.inner_gap.value_px,
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
            ("trigger height", &metrics.trigger_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("content padding y", &metrics.content_padding_y),
            ("radius", &metrics.radius),
            ("item gap", &metrics.item_gap),
            ("inner gap", &metrics.inner_gap),
            ("icon size", &metrics.icon_size),
            ("chevron size", &metrics.chevron_size),
        ]),
    )
}
