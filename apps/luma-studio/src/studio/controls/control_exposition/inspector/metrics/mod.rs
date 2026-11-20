use gpui::SharedString;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::box_model::InspectBoxModelSnapshot;
use super::common::{neutral_box_model_colors, neutral_box_model_label_color};
use super::schema::InspectLayoutSection;

mod input;
mod layout;
mod overlay;

pub use input::*;
pub use layout::*;
pub use overlay::*;

pub(crate) fn layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    box_model: InspectBoxModelSnapshot,
    occupation: Option<super::box_model::InspectOccupationSnapshot>,
    metrics: Vec<super::schema::InspectPropertyRow>,
) -> InspectLayoutSection {
    InspectLayoutSection {
        box_model_diagram_id: SharedString::from(diagram_id),
        box_model,
        occupation,
        box_model_colors: neutral_box_model_colors(look.mode()),
        box_model_label_color: neutral_box_model_label_color(look.mode()),
        metrics,
    }
}
