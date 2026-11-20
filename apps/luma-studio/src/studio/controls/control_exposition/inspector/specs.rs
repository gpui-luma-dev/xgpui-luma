use lucide_svg_static::Icon as LucideIcon;

use super::schema::{InspectorCategory, InspectorPart, InspectorSize, InspectorStateSpec, InspectorVariant};

pub static CHOICE_SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

pub static COLOR_LAYOUT_CATEGORIES: [InspectorCategory; 2] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
];

pub static DEFAULT_INTERACTION_STATES: [InspectorStateSpec; 1] = [InspectorStateSpec {
    id: "default",
    label: "Default",
    icon: LucideIcon::Circle,
    expanded_default: true,
    categories: &COLOR_LAYOUT_CATEGORIES,
}];

pub static BADGE_VARIANTS: [InspectorVariant; 4] = [
    InspectorVariant { id: "default", label: "Default" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
];

pub static SPLIT_BUTTON_VARIANTS: [InspectorVariant; 4] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
];

pub static SPLIT_BUTTON_PARTS: [InspectorPart; 2] = [
    InspectorPart {
        id: "trigger",
        label: "Split Button",
        variants: &SPLIT_BUTTON_VARIANTS,
        default_variant_id: "outline",
    },
    InspectorPart { id: "panel", label: "Panel", variants: &[], default_variant_id: "" },
];

pub static POPUP_MENU_STATES: [InspectorStateSpec; 7] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "panel-default",
        label: "Surface",
        icon: LucideIcon::Circle,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "panel-disabled",
        label: "Item Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "panel-hover",
        label: "Item Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static FLOATING_MENU_ITEM_STATES: [InspectorStateSpec; 3] = [
    InspectorStateSpec {
        id: "default",
        label: "Surface",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Item Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Item Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];
