use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::sidebar::{SidebarBuilder, SidebarMenuItemBuilder};
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

pub(crate) const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";

#[derive(Clone, Copy)]
struct PropertyLeaf {
    id: &'static str,
    label: &'static str,
    icon: Option<LucideIcon>,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct PropertyGroup {
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    expanded: bool,
    leaves: &'static [PropertyLeaf],
}

const PINNED_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "summary", label: "Summary", icon: Some(LucideIcon::Info), enabled: true },
    PropertyLeaf { id: "tokens", label: "Design Tokens", icon: Some(LucideIcon::Tags), enabled: true },
];

const LAYOUT_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "position", label: "Position", icon: None, enabled: true },
    PropertyLeaf { id: INITIAL_PROPERTY_SELECTION_ID, label: "Dimensions", icon: None, enabled: true },
    PropertyLeaf { id: "constraints", label: "Constraints", icon: None, enabled: true },
    PropertyLeaf { id: "grid", label: "Grid", icon: None, enabled: true },
];

const LOOK_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "fill", label: "Fill", icon: None, enabled: true },
    PropertyLeaf { id: "stroke", label: "Stroke", icon: None, enabled: true },
    PropertyLeaf { id: "typography", label: "Typography", icon: None, enabled: true },
    PropertyLeaf { id: "effects", label: "Effects", icon: None, enabled: true },
];

const BEHAVIOR_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "interactions", label: "Interactions", icon: None, enabled: true },
    PropertyLeaf { id: "conditions", label: "Conditions", icon: None, enabled: true },
    PropertyLeaf { id: "validation", label: "Validation", icon: None, enabled: true },
    PropertyLeaf { id: "data-binding", label: "Data Binding", icon: None, enabled: false },
];

const PROPERTY_GROUPS: &[PropertyGroup] = &[
    PropertyGroup { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded: true, leaves: LAYOUT_PROPERTIES },
    PropertyGroup { id: "look", label: "Look", icon: LucideIcon::Palette, expanded: true, leaves: LOOK_PROPERTIES },
    PropertyGroup {
        id: "behavior",
        label: "Behavior",
        icon: LucideIcon::MousePointer2,
        expanded: false,
        leaves: BEHAVIOR_PROPERTIES,
    },
];

const FOOTER_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "audit-log", label: "Audit Log", icon: Some(LucideIcon::FileText), enabled: true },
    PropertyLeaf { id: "reset-overrides", label: "Reset Overrides", icon: Some(LucideIcon::RotateCcw), enabled: false },
];

pub(crate) fn property_sidebar(
    look: &Arc<ShadcnLook>,
    panel_id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
) -> SidebarBuilder {
    let panel_id = panel_id.into();
    let pinned_menu_id = SharedString::from(format!("{panel_id}-pinned"));
    let properties_menu_id = SharedString::from(format!("{panel_id}-properties"));

    let mut pinned_menu = shadcn::Sidebar::menu(pinned_menu_id);
    for leaf in PINNED_PROPERTIES {
        pinned_menu = pinned_menu.item(property_leaf_menu_item(look, leaf));
    }

    let mut properties_menu = shadcn::Sidebar::menu(properties_menu_id);
    for group in PROPERTY_GROUPS {
        let mut sub = shadcn::Sidebar::menu_sub();
        for leaf in group.leaves {
            sub = sub.item(property_leaf_menu_item(look, leaf));
        }
        properties_menu = properties_menu.item(
            shadcn::Sidebar::menu_item(group.id, group.label)
                .help(format!("Expand or collapse {}", group.label))
                .icon(group.icon)
                .expanded(group.expanded)
                .sub(sub),
        );
    }

    let mut footer = shadcn::Sidebar::footer();
    for leaf in FOOTER_PROPERTIES {
        footer = footer.child(property_leaf_menu_item(look, leaf));
    }

    shadcn::Sidebar::panel(panel_id)
        .header(shadcn::Sidebar::header().title(title).subtitle(subtitle))
        .content(
            shadcn::Sidebar::content()
                .group(shadcn::Sidebar::group().id("pinned").label("Pinned").menu(pinned_menu))
                .group(shadcn::Sidebar::group().id("properties").label("Properties").menu(properties_menu)),
        )
        .footer(footer)
}

fn property_leaf_menu_item(_look: &Arc<ShadcnLook>, leaf: &PropertyLeaf) -> SidebarMenuItemBuilder {
    let mut item = shadcn::Sidebar::menu_item(leaf.id, leaf.label)
        .help(format!("Select {}", leaf.label))
        .disabled(!leaf.enabled)
        .active(leaf.id == INITIAL_PROPERTY_SELECTION_ID);
    if let Some(icon) = leaf.icon {
        item = item.icon(icon);
    }
    item
}
