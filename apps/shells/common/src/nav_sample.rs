use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::sidebar::SidebarControl;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

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

const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";

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

pub fn spawn_properties_sidebar<T: 'static>(
    look: Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    cx: &mut Context<T>,
) -> Entity<SidebarControl> {
    let id = id.into();
    let panel_id = SharedString::from(format!("{id}-panel"));

    let mut pinned_menu = shadcn::Sidebar::menu(format!("{id}-pinned"));
    for leaf in PINNED_PROPERTIES {
        pinned_menu = pinned_menu.item(property_leaf_menu_item(&look, leaf));
    }

    let mut properties_menu = shadcn::Sidebar::menu(format!("{id}-properties"));
    for group in PROPERTY_GROUPS {
        let mut sub = shadcn::Sidebar::menu_sub();
        for leaf in group.leaves {
            sub = sub.item(property_leaf_menu_item(&look, leaf));
        }
        properties_menu = properties_menu
            .item(shadcn::Sidebar::menu_item(group.id, group.label).icon(group.icon).expanded(group.expanded).sub(sub));
    }

    let mut footer = shadcn::Sidebar::footer();
    for leaf in FOOTER_PROPERTIES {
        footer = footer.child(property_leaf_menu_item(&look, leaf));
    }

    shadcn::Sidebar::new(id)
        .look(look.as_ref())
        .sidebar(
            shadcn::Sidebar::panel(panel_id)
                .header(shadcn::Sidebar::header().title("Properties").subtitle("Rectangle / Prominent card"))
                .content(
                    shadcn::Sidebar::content()
                        .group(shadcn::Sidebar::group().label("Pinned").menu(pinned_menu))
                        .group(shadcn::Sidebar::group().label("Properties").menu(properties_menu)),
                )
                .footer(footer),
        )
        .spawn(cx)
}

fn property_leaf_menu_item(
    _look: &Arc<ShadcnLook>,
    leaf: &PropertyLeaf,
) -> xgpui_luma::controls::sidebar::SidebarMenuItemBuilder {
    let mut item = shadcn::Sidebar::menu_item(leaf.id, leaf.label)
        .disabled(!leaf.enabled)
        .active(leaf.id == INITIAL_PROPERTY_SELECTION_ID);
    if let Some(icon) = leaf.icon {
        item = item.icon(icon);
    }
    item
}
