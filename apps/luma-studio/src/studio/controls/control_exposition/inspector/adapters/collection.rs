//! Inspector adapters for the collection control family.

mod listbox {
    use std::sync::Arc;

    use xgpui_luma::theme::InteractionState;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;
    use lucide_svg_static::Icon as LucideIcon;

    use super::super::super::collection::{listbox_list_color_rows, listbox_row_color_rows};
    use super::super::super::common::interaction_state;
    use super::super::super::metrics::listbox_layout_section;
    use super::super::super::specs::COLOR_LAYOUT_CATEGORIES;
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, InspectorVariant, SharedInspectorResolver, InspectColorRow,
    };

    static VARIANTS: [InspectorVariant; 2] = [
        InspectorVariant { id: "vertical", label: "Vertical" },
        InspectorVariant { id: "horizontal", label: "Horizontal" },
    ];
    static STATES: [InspectorStateSpec; 6] = [
        InspectorStateSpec {
            id: "default",
            label: "Default",
            icon: LucideIcon::Circle,
            expanded_default: true,
            categories: &COLOR_LAYOUT_CATEGORIES,
        },
        InspectorStateSpec {
            id: "selected",
            label: "Selected",
            icon: LucideIcon::Check,
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
        InspectorStateSpec {
            id: "keyboard-active",
            label: "Keyboard Active",
            icon: LucideIcon::Focus,
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
    ];

    pub static LISTBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "ListBox composition",
        id_prefix: "listbox-theme-inspector",
        parts: &[],
        variants: &VARIANTS,
        states: &STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "vertical",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct ListBoxInspectorAdapter;

    impl ListBoxInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ListBoxInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "layout" => InspectorCategoryContent::Layout(listbox_layout_section(
                    look,
                    "listbox-composition-box-model",
                    selection.variant_id,
                )),
                "color" => {
                    let inspect = ShadcnInspect::new(look);
                    let state = match selection.state_id {
                        "selected" => InteractionState { focused: true, ..Default::default() },
                        // Active navigation has an outline; selection alone supplies a filled background.
                        "keyboard-active" => InteractionState::default(),
                        other => interaction_state(other),
                    };
                    let mut rows = listbox_row_color_rows(&inspect.inspect_listbox_row_color_palette(state));
                    rows.extend(listbox_list_color_rows(&inspect.inspect_listbox_list_color_palette(true, false)));
                    if selection.state_id == "keyboard-active" {
                        rows.push(InspectColorRow {
                            label: "focus border",
                            value: xgpui_luma_look_shadcn::paint::focus_ring_color(&look.mode_tokens().catalog)
                                .unwrap_or_else(|_| look.chrome().body_text),
                            source: "ring".into(),
                            detail: None,
                        });
                    }
                    InspectorCategoryContent::Colors(rows)
                }
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
            category_id != "layout" || selection.state_id == "default"
        }
    }
}
pub use listbox::{LISTBOX_INSPECTOR_SPEC, ListBoxInspectorAdapter};

mod table {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::{table_row_color_rows, table_surface_color_rows};
    use super::super::super::common::{table_row_selected, listbox_row_state, progress_enabled};
    use super::super::super::metrics::table_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, TABLE_ROW_VALUE_MODES, TABLE_STATES, TABLE_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static TABLE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Table",
        id_prefix: "table-theme-inspector",
        parts: &[],
        variants: &TABLE_VARIANTS,
        states: &TABLE_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &TABLE_ROW_VALUE_MODES,
        default_part_id: "",
        default_variant_id: "surface",
        default_size_id: "md",
        default_value_id: "unselected",
    };

    pub struct TableInspectorAdapter;

    impl TableInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for TableInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(table_layout_section(
                    look,
                    "table-theme-inspector-box-model",
                    selection.variant_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.variant_id {
                "surface" => matches!(state.id, "enabled" | "disabled"),
                "row" | "grid-cell" => {
                    matches!(state.id, "default" | "hover" | "pressed" | "keyboard-active" | "disabled")
                }
                _ => true,
            }
        }

        fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
            selection.variant_id == "row"
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let inspect = ShadcnInspect::new(look);
        match selection.variant_id {
            "row" | "grid-cell" => {
                let palette = inspect.inspect_table_row_color_palette(
                    table_row_selected(selection.value_id),
                    listbox_row_state(selection.state_id),
                );
                table_row_color_rows(&palette)
            }
            _ => {
                let palette = inspect.inspect_table_color_palette(progress_enabled(selection.state_id));
                table_surface_color_rows(&palette)
            }
        }
    }
}
pub use table::{TABLE_INSPECTOR_SPEC, TableInspectorAdapter};

mod tree_view {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::tree_view_row_color_rows;
    use super::super::super::common::{interaction_state, listbox_row_state};
    use super::super::super::metrics::tree_view_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, TREE_VIEW_ROW_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static TREE_VIEW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "TreeView",
        id_prefix: "tree-view-theme-inspector",
        parts: &[],
        variants: &[],
        states: &TREE_VIEW_ROW_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct TreeViewInspectorAdapter;

    impl TreeViewInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for TreeViewInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(tree_view_layout_section(
                    look,
                    "tree-view-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let state = if selection.state_id == "disabled" {
            interaction_state("disabled")
        } else {
            listbox_row_state(selection.state_id)
        };
        let palette = ShadcnInspect::new(look).inspect_tree_view_row_color_palette(state);
        tree_view_row_color_rows(&palette)
    }
}
pub use tree_view::{TREE_VIEW_INSPECTOR_SPEC, TreeViewInspectorAdapter};

mod tabs {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::{tabs_item_color_rows, tabs_list_color_rows};
    use super::super::super::common::{interaction_state, progress_enabled, tabs_active};
    use super::super::super::metrics::tabs_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, TABS_NAVIGATION_STATES, TABS_NAVIGATION_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static TABS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Tabs Navigation",
        id_prefix: "tabs-navigation-theme-inspector",
        parts: &[],
        variants: &TABS_NAVIGATION_VARIANTS,
        states: &TABS_NAVIGATION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "inactive",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct TabsInspectorAdapter;

    impl TabsInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for TabsInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(tabs_layout_section(
                    look,
                    "tabs-navigation-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.variant_id {
                "list" => matches!(state.id, "enabled" | "disabled"),
                "inactive" | "active" => {
                    matches!(state.id, "default" | "hover" | "focused" | "pressed" | "item-disabled")
                }
                _ => true,
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let inspect = ShadcnInspect::new(look);
        if selection.variant_id == "list" {
            let palette = inspect.inspect_tabs_list_color_palette(progress_enabled(selection.state_id));
            return tabs_list_color_rows(&palette);
        }

        let state_id = if selection.state_id == "item-disabled" {
            "disabled"
        } else {
            selection.state_id
        };
        let palette =
            inspect.inspect_tabs_item_color_palette(tabs_active(selection.variant_id), interaction_state(state_id));
        tabs_item_color_rows(&palette)
    }
}
pub use tabs::{TABS_INSPECTOR_SPEC, TabsInspectorAdapter};

mod accordion {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::{accordion_content_color_rows, accordion_trigger_color_rows};
    use super::super::super::common::{accordion_content_expanded, interaction_state};
    use super::super::super::metrics::accordion_layout_section;
    use super::super::super::specs::{ACCORDION_STATES, ACCORDION_VARIANTS, CHOICE_SIZES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static ACCORDION_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Accordion",
        id_prefix: "accordion-theme-inspector",
        parts: &[],
        variants: &ACCORDION_VARIANTS,
        states: &ACCORDION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "trigger",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct AccordionInspectorAdapter;

    impl AccordionInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for AccordionInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(accordion_layout_section(
                    look,
                    "accordion-theme-inspector-box-model",
                    selection.size_id,
                    selection.scale_factor,
                )),
                "typography" => InspectorCategoryContent::Typography(super::super::super::provenance::typography_rows(
                    &ShadcnInspect::new(look)
                        .inspect_accordion_typography(super::super::super::common::control_size(selection.size_id)),
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
            category_id != "typography" || selection.variant_id == "trigger"
        }

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.variant_id {
                "trigger" => matches!(state.id, "default" | "hover" | "pressed" | "disabled"),
                "content" => matches!(state.id, "expanded" | "collapsed"),
                _ => true,
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let inspect = ShadcnInspect::new(look);
        if selection.variant_id == "content" {
            let palette =
                inspect.inspect_accordion_content_color_palette(accordion_content_expanded(selection.state_id));
            return accordion_content_color_rows(&palette);
        }
        let palette = inspect.inspect_accordion_trigger_color_palette(interaction_state(selection.state_id));
        accordion_trigger_color_rows(&palette)
    }
}
pub use accordion::{ACCORDION_INSPECTOR_SPEC, AccordionInspectorAdapter};

mod sidebar {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::{
        sidebar_container_color_rows, sidebar_item_color_rows, sidebar_section_color_rows,
    };
    use super::super::super::common::{interaction_state, sidebar_item_selected};
    use super::super::super::metrics::sidebar_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, SIDEBAR_STATES, SIDEBAR_VARIANTS, NAV_ITEM_VALUE_MODES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static SIDEBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Sidebar",
        id_prefix: "sidebar-theme-inspector",
        parts: &[],
        variants: &SIDEBAR_VARIANTS,
        states: &SIDEBAR_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &NAV_ITEM_VALUE_MODES,
        default_part_id: "",
        default_variant_id: "container",
        default_size_id: "md",
        default_value_id: "unselected",
    };

    pub struct SidebarInspectorAdapter;

    impl SidebarInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for SidebarInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(sidebar_layout_section(
                    look,
                    "sidebar-theme-inspector-box-model",
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.variant_id {
                "container" | "section" => state.id == "default",
                "branch" => matches!(state.id, "default" | "hover" | "pressed" | "focused" | "disabled"),
                "nav-item" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
                _ => true,
            }
        }

        fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
            selection.variant_id == "nav-item"
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let inspect = ShadcnInspect::new(look);
        match selection.variant_id {
            "section" => {
                let palette = inspect.inspect_sidebar_section_color_palette();
                sidebar_section_color_rows(&palette)
            }
            "branch" => {
                let palette = inspect.inspect_sidebar_branch_color_palette(interaction_state(selection.state_id));
                sidebar_item_color_rows(&palette)
            }
            "nav-item" => {
                let palette = inspect.inspect_sidebar_item_color_palette(
                    sidebar_item_selected(selection.value_id),
                    interaction_state(selection.state_id),
                );
                sidebar_item_color_rows(&palette)
            }
            _ => {
                let palette = inspect.inspect_sidebar_container_color_palette();
                sidebar_container_color_rows(&palette)
            }
        }
    }
}
pub use sidebar::{SIDEBAR_INSPECTOR_SPEC, SidebarInspectorAdapter};
