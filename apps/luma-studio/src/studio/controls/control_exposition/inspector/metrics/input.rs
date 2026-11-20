use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{FloatingMenuInspectMetrics, ShadcnInspect, TextFieldInspectMetrics};

use super::super::box_model::InspectBoxModelSnapshot;
use super::super::common::control_size;
use super::super::provenance::metric_properties;
use super::super::schema::InspectLayoutSection;
use super::layout_section;

pub fn textfield_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_and_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_textfield_metrics(size),
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
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
