use std::sync::Arc;

use gpui::{BoxShadow, Hsla, SharedString};
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use super::box_model::{BoxModelLayerColors, InspectBoxModelSnapshot, InspectOccupationSnapshot};

pub struct ControlInspectorSpec {
    pub control_label: &'static str,
    pub id_prefix: &'static str,
    pub parts: &'static [InspectorPart],
    pub variants: &'static [InspectorVariant],
    pub states: &'static [InspectorStateSpec],
    pub sizes: &'static [InspectorSize],
    pub value_modes: &'static [InspectorValueMode],
    pub default_part_id: &'static str,
    pub default_variant_id: &'static str,
    pub default_size_id: &'static str,
    pub default_value_id: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorPart {
    pub id: &'static str,
    pub label: &'static str,
    pub variants: &'static [InspectorVariant],
    pub default_variant_id: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorValueMode {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorVariant {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorStateSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: LucideIcon,
    pub expanded_default: bool,
    pub categories: &'static [InspectorCategory],
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorSize {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorCategory {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: LucideIcon,
    pub expanded_default: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorSelection<'a> {
    pub part_id: &'a str,
    pub variant_id: &'a str,
    pub state_id: &'a str,
    pub size_id: &'a str,
    pub value_id: &'a str,
    pub scale_factor: f32,
}

#[derive(Clone, Debug)]
pub enum InspectorCategoryContent {
    Colors(Vec<InspectColorRow>),
    Layout(InspectLayoutSection),
    Elevation(InspectElevationSnapshot),
    Typography(Vec<InspectPropertyRow>),
}

#[derive(Clone, Debug)]
pub struct InspectColorRow {
    pub label: &'static str,
    pub value: Hsla,
    pub source: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InspectPropertyRow {
    pub label: String,
    pub value: String,
    pub source: String,
    pub detail: Option<String>,
}

impl InspectPropertyRow {
    pub fn new(label: impl Into<String>, value: impl Into<String>, source: impl Into<String>) -> Self {
        Self { label: label.into(), value: value.into(), source: source.into(), detail: None }
    }
}

#[derive(Clone, Debug)]
pub struct InspectLayoutSection {
    pub box_model_diagram_id: SharedString,
    pub box_model: InspectBoxModelSnapshot,
    pub occupation: Option<InspectOccupationSnapshot>,
    pub box_model_colors: BoxModelLayerColors,
    pub box_model_label_color: Hsla,
    pub metrics: Vec<InspectPropertyRow>,
}

#[derive(Clone, Debug)]
pub struct InspectElevationLayer {
    pub color: Hsla,
    pub display: String,
}

#[derive(Clone, Debug)]
pub struct InspectElevationSnapshot {
    pub property_rows: Vec<InspectPropertyRow>,
    pub catalog_value: Option<String>,
    pub layers: Vec<InspectElevationLayer>,
    pub preview_shadows: Option<Vec<BoxShadow>>,
}

pub trait ControlInspectorResolver: Send + Sync {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent;

    fn category_applies(&self, _look: &ShadcnLook, _selection: InspectorSelection<'_>, _category_id: &str) -> bool {
        true
    }

    fn state_applies(
        &self,
        _look: &ShadcnLook,
        _selection: InspectorSelection<'_>,
        _state: &InspectorStateSpec,
    ) -> bool {
        true
    }

    fn value_modes_applies(&self, _look: &ShadcnLook, _selection: InspectorSelection<'_>) -> bool {
        true
    }

    fn part_applies(&self, _look: &ShadcnLook, _selection: InspectorSelection<'_>, _part: &InspectorPart) -> bool {
        true
    }
}

pub type SharedInspectorResolver = Arc<dyn ControlInspectorResolver>;
