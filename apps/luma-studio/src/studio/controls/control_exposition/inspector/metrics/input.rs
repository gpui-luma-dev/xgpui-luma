use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{
    AutocompleteInspectMetrics, FloatingMenuInspectMetrics, ShadcnInspect, TextFieldInspectMetrics,
};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::InspectLayoutSection;
use super::layout_section;

pub fn textfield_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_textfield_metrics(size))
}

pub fn textarea_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_textarea_metrics(size))
}

pub fn selector_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_selector_metrics(size);
    selector_metrics_layout_section(look, diagram_id, &metrics)
}

pub fn textfield_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_and_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_textfield_metrics(size),
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
}

pub fn autocomplete_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    autocomplete_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_autocomplete_metrics(size))
}
fn textfield_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TextFieldInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_textfield_metrics(metrics);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("min height", &metrics.min_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("radius", &metrics.radius),
            ("border width", &metrics.border_width),
            ("focus ring width", &metrics.focus_ring_width),
            ("focus ring offset", &metrics.focus_ring_offset),
        ]),
    )
}

fn selector_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &xgpui_luma_look_shadcn::inspect::SelectorInspectMetrics,
) -> InspectLayoutSection {
    let trigger_box = InspectBoxModelSnapshot::from_button_metrics(&metrics.trigger);
    let rows = metric_properties(&[
        ("trigger height", &metrics.trigger.height),
        ("trigger icon size", &metrics.trigger.icon_size),
        ("trigger padding x", &metrics.trigger.padding_x),
        ("trigger padding y", &metrics.trigger.padding_y),
        ("trigger radius", &metrics.trigger.radius),
        ("trigger border width", &metrics.trigger.border_width),
        ("trigger focus ring width", &metrics.trigger.focus_ring_width),
        ("trigger focus ring offset", &metrics.trigger.focus_ring_offset),
        ("menu radius", &metrics.items_panel.radius),
        ("menu padding", &metrics.items_panel.padding),
        ("menu min width", &metrics.items_panel.min_width),
        ("menu item height", &metrics.items_panel.item_height),
        ("menu item padding x", &metrics.items_panel.item_padding_x),
        ("menu item gap", &metrics.items_panel.item_gap),
        ("menu item icon size", &metrics.items_panel.item_icon_size),
        ("menu item radius", &metrics.items_panel.item_radius),
        ("menu submenu offset x", &metrics.items_panel.submenu_offset_x),
    ]);
    layout_section(look, diagram_id, trigger_box, None, rows)
}

fn textfield_and_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    textfield: &TextFieldInspectMetrics,
    menu: &FloatingMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_textfield_metrics(textfield);
    let rows = metric_properties(&[
        ("textfield min height", &textfield.min_height),
        ("textfield icon size", &textfield.icon_size),
        ("textfield padding x", &textfield.padding_x),
        ("textfield padding y", &textfield.padding_y),
        ("textfield radius", &textfield.radius),
        ("textfield border width", &textfield.border_width),
        ("textfield focus ring width", &textfield.focus_ring_width),
        ("textfield focus ring offset", &textfield.focus_ring_offset),
        ("menu radius", &menu.radius),
        ("menu padding", &menu.padding),
        ("menu min width", &menu.min_width),
        ("menu item height", &menu.item_height),
        ("menu item padding x", &menu.item_padding_x),
        ("menu item gap", &menu.item_gap),
        ("menu item icon size", &menu.item_icon_size),
        ("menu item radius", &menu.item_radius),
        ("menu submenu offset x", &menu.submenu_offset_x),
    ]);
    layout_section(look, diagram_id, box_model, None, rows)
}

fn autocomplete_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &AutocompleteInspectMetrics,
) -> InspectLayoutSection {
    textfield_and_menu_metrics_layout_section(look, diagram_id, &metrics.textfield, &metrics.menu)
}
