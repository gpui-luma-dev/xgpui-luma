use lucide_svg_static::Icon as LucideIcon;

use super::schema::{
    InspectorCategory, InspectorPart, InspectorSize, InspectorStateSpec, InspectorValueMode, InspectorVariant,
};

pub static PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "content-only", label: "Content Only" },
];

pub static CHOICE_LAYOUT_PARTS: [InspectorPart; 2] = [
    InspectorPart {
        id: "labeled",
        label: "Labeled",
        variants: &PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
        default_variant_id: "primary",
    },
    InspectorPart {
        id: "indicator-only",
        label: "Indicator Only",
        variants: &PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
        default_variant_id: "primary",
    },
];

pub static CHOICE_SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

pub static COLOR_LAYOUT_CATEGORIES: [InspectorCategory; 2] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
];

pub static COLOR_LAYOUT_ELEVATION_CATEGORIES: [InspectorCategory; 3] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "elevation", label: "Elevation", icon: LucideIcon::Layers, expanded_default: true },
];

pub static COLOR_LAYOUT_INTERACTION_STATES: [InspectorStateSpec; 4] = [
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
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static CHOICE_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
];

pub static SELECTED_UNSELECTED_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "selected", label: "selected" },
    InspectorValueMode { id: "unselected", label: "unselected" },
];

pub static ON_OFF_VALUES: [InspectorValueMode; 2] =
    [InspectorValueMode { id: "on", label: "on" }, InspectorValueMode { id: "off", label: "off" }];

pub static TEXTFIELD_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "surface", label: "Surface" },
];

pub static TEXTFIELD_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focus",
        label: "Focus",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "invalid",
        label: "Invalid",
        icon: LucideIcon::CircleAlert,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
];

pub static TEXTAREA_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focus",
        label: "Focus",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "invalid",
        label: "Invalid",
        icon: LucideIcon::CircleAlert,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
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

pub static PROGRESS_STATES: [InspectorStateSpec; 2] = [
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static PROGRESS_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "circular", label: "Circular" },
    InspectorVariant { id: "linear", label: "Linear" },
];

pub static STEPPER_STATES: [InspectorStateSpec; 2] = [
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static STEPPER_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "horizontal", label: "Horizontal" },
    InspectorVariant { id: "vertical", label: "Vertical" },
];

pub static OVERLAY_WINDOW_MODE_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "modal", label: "Modal" },
    InspectorVariant { id: "modeless", label: "Modeless" },
];

pub static SCROLLBAR_STYLE_VARIANTS: [InspectorVariant; 2] =
    [InspectorVariant { id: "ghost", label: "Ghost" }, InspectorVariant { id: "soft", label: "Soft" }];

pub static POPUP_MENU_TRIGGER_VARIANTS: [InspectorVariant; 2] = [
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

pub static TOOLBAR_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
];

pub static PAGER_STYLE_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "minimal", label: "Minimal" },
    InspectorVariant { id: "minimal-edge", label: "Minimal + edges" },
    InspectorVariant { id: "numeric", label: "Numeric" },
];

pub static PAGER_BUTTON_VARIANTS: [InspectorVariant; 2] =
    [InspectorVariant { id: "nav", label: "Nav" }, InspectorVariant { id: "page", label: "Page" }];

pub static PAGER_PARTS: [InspectorPart; 2] = [
    InspectorPart { id: "shell", label: "Shell", variants: &PAGER_STYLE_VARIANTS, default_variant_id: "minimal" },
    InspectorPart { id: "button", label: "Button", variants: &PAGER_BUTTON_VARIANTS, default_variant_id: "nav" },
];

pub static PAGER_STATES: [InspectorStateSpec; 5] = [
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
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
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
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static POPUP_MENU_PARTS: [InspectorPart; 2] = [
    InspectorPart {
        id: "trigger",
        label: "Trigger",
        variants: &POPUP_MENU_TRIGGER_VARIANTS,
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

pub static TABLE_STATES: [InspectorStateSpec; 6] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: false,
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
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
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
        id: "keyboard-active",
        label: "Keyboard Active",
        icon: LucideIcon::Focus,
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

pub static TABLE_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "surface", label: "Surface" },
    InspectorVariant { id: "row", label: "Row" },
    InspectorVariant { id: "grid-cell", label: "Grid Cell" },
];

pub static TABLE_ROW_VALUE_MODES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "unselected", label: "unselected" },
    InspectorValueMode { id: "selected", label: "selected" },
];

pub static SIDEBAR_VARIANTS: [InspectorVariant; 4] = [
    InspectorVariant { id: "container", label: "Container" },
    InspectorVariant { id: "section", label: "Section" },
    InspectorVariant { id: "branch", label: "Parent Item" },
    InspectorVariant { id: "nav-item", label: "Leaf Item" },
];

pub static NAV_ITEM_VALUE_MODES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "unselected", label: "unselected" },
    InspectorValueMode { id: "selected", label: "selected" },
];

pub static TABS_NAVIGATION_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "inactive", label: "Inactive" },
    InspectorVariant { id: "active", label: "Active" },
    InspectorVariant { id: "list", label: "List" },
];

pub static ACCORDION_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "trigger", label: "Trigger" },
    InspectorVariant { id: "content", label: "Content" },
];

pub static RESIZE_HANDLE_SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

pub static TREE_VIEW_ROW_STATES: [InspectorStateSpec; 4] = [
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
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static SIDEBAR_STATES: [InspectorStateSpec; 5] = [
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
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
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

pub static TABS_NAVIGATION_STATES: [InspectorStateSpec; 6] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: false,
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
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
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
        id: "item-disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
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

static ACCORDION_CATEGORIES: [InspectorCategory; 3] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "typography", label: "Typography", icon: LucideIcon::TypeIcon, expanded_default: true },
];

pub static ACCORDION_STATES: [InspectorStateSpec; 6] = [
    InspectorStateSpec {
        id: "collapsed",
        label: "Collapsed",
        icon: LucideIcon::ChevronRight,
        expanded_default: false,
        categories: &ACCORDION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &ACCORDION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &ACCORDION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "expanded",
        label: "Expanded",
        icon: LucideIcon::ChevronDown,
        expanded_default: false,
        categories: &ACCORDION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &ACCORDION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &ACCORDION_CATEGORIES,
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
