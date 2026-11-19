use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::super::common::{choice_variant_style, control_size};
use super::super::schema::InspectLayoutSection;
use super::button::button_layout_section;

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
