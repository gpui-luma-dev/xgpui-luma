use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{inspect_color_chrome_sections, ColorChromeInspectSection};

use super::provenance::color_row;
use super::schema::{InspectColorRow, InspectPropertyRow};

pub use xgpui_luma_look_shadcn::inspect::{
    ColorChromeProfile, COLOR_ARC_CHROME_PROFILES, COLOR_FIELD_CHROME_PROFILES, COLOR_RING_CHROME_PROFILES,
    COLOR_SLIDER_CHROME_PROFILES,
};

#[derive(Clone, Debug)]
pub struct ColorChromeSection {
    pub title: &'static str,
    pub note: Option<&'static str>,
    pub color_rows: Vec<InspectColorRow>,
    pub property_rows: Vec<InspectPropertyRow>,
}

pub fn resolve_color_chrome_sections(
    look: &ShadcnLook,
    profiles: &[xgpui_luma_look_shadcn::inspect::ColorChromeProfile],
) -> Vec<ColorChromeSection> {
    inspect_color_chrome_sections(look, profiles).into_iter().map(convert_section).collect()
}

fn convert_section(section: ColorChromeInspectSection) -> ColorChromeSection {
    ColorChromeSection {
        title: section.title,
        note: section.note,
        color_rows: section.color_rows.into_iter().map(|row| color_row(row.label, &row.color)).collect(),
        property_rows: section
            .property_rows
            .into_iter()
            .map(|row| InspectPropertyRow {
                label: row.label.to_owned(),
                value: row.value,
                source: row.source,
                detail: row.detail,
            })
            .collect(),
    }
}
