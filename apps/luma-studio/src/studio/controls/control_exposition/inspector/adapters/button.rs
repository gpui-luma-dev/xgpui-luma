//! Inspector adapters for the button control family.

use std::sync::Arc;

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::ShadcnInspect;
use lucide_svg_static::Icon as LucideIcon;

use super::super::common::{button_variant_style, control_size, interaction_state};
use super::super::metrics::button_layout_section;
use super::super::provenance::{color_row, elevation_snapshot};
use super::super::specs::CHOICE_SIZES;
use super::super::schema::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectPropertyRow, InspectorCategory,
    InspectorCategoryContent, InspectorSelection, InspectorStateSpec, InspectorVariant, SharedInspectorResolver,
};

const BUTTON_ROLE_TEXT: ButtonFamilyRole = ButtonFamilyRole::Text;

fn button_role(selection: InspectorSelection<'_>) -> ButtonFamilyRole {
    if selection.variant_id == "content-only" {
        ButtonFamilyRole::Icon
    } else {
        BUTTON_ROLE_TEXT
    }
}

static BUTTON_VARIANTS: [InspectorVariant; 5] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
    InspectorVariant { id: "content-only", label: "Content Only" },
];

static BUTTON_CATEGORIES: [InspectorCategory; 4] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "elevation", label: "Elevation", icon: LucideIcon::Layers, expanded_default: true },
    InspectorCategory { id: "typography", label: "Typography", icon: LucideIcon::TypeIcon, expanded_default: true },
];

static BUTTON_STATES: [InspectorStateSpec; 4] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
];

pub static BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Button",
    id_prefix: "button-theme-inspector",
    parts: &[],
    variants: &BUTTON_VARIANTS,
    states: &BUTTON_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ButtonInspectorAdapter;

impl ButtonInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ButtonInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
            "elevation" => InspectorCategoryContent::Elevation(resolve_elevation(look, selection)),
            "typography" => InspectorCategoryContent::Typography(resolve_typography_rows(look, selection)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        if category_id == "elevation" {
            return super::super::common::button_elevation_applies(selection.variant_id, selection.state_id);
        }
        true
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        button_variant_style(selection.variant_id),
        button_role(selection),
        interaction_state(selection.state_id),
    );
    let rows = vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
    ];
    rows
}

fn resolve_layout_section(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::super::schema::InspectLayoutSection {
    button_layout_section(
        look,
        "button-theme-inspector-box-model",
        button_variant_style(selection.variant_id),
        button_role(selection),
        control_size(selection.size_id),
        interaction_state(selection.state_id),
    )
}

fn resolve_elevation(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::super::schema::InspectElevationSnapshot {
    let elevation = ShadcnInspect::new(look).inspect_button_elevation(
        button_variant_style(selection.variant_id),
        button_role(selection),
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved button look", "button.elevation_rules[].style")
}

fn resolve_typography_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectPropertyRow> {
    let typography = ShadcnInspect::new(look)
        .inspect_button_typography_for_size(control_size(selection.size_id), button_role(selection));
    super::super::provenance::typography_rows(&typography)
}
