//! Inspector adapters for the layout control family.

mod split_view {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::split_view_color_rows;
    use super::super::super::common::progress_enabled;
    use super::super::super::metrics::split_view_layout_section;
    use super::super::super::specs::PROGRESS_STATES;
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static SPLIT_VIEW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Split View",
        id_prefix: "split-view-theme-inspector",
        parts: &[],
        variants: &[],
        states: &PROGRESS_STATES,
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "",
    };

    pub struct SplitViewInspectorAdapter;

    impl SplitViewInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for SplitViewInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(split_view_layout_section(
                    look,
                    "split-view-theme-inspector-box-model",
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
    ) -> Vec<super::super::super::schema::InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_split_view_color_palette(progress_enabled(selection.state_id));
        split_view_color_rows(&palette)
    }
}
pub use split_view::{SPLIT_VIEW_INSPECTOR_SPEC, SplitViewInspectorAdapter};

mod resizable_panels {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::resizable_panels_color_rows;
    use super::super::super::common::interaction_state;
    use super::super::super::metrics::resizable_panels_layout_section;
    use super::super::super::specs::{RESIZE_HANDLE_SIZES, TREE_VIEW_ROW_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static RESIZABLE_PANELS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Resizable Panels",
        id_prefix: "resizable-panels-theme-inspector",
        parts: &[],
        variants: &[],
        states: &TREE_VIEW_ROW_STATES,
        sizes: &RESIZE_HANDLE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct ResizablePanelsInspectorAdapter;

    impl ResizablePanelsInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ResizablePanelsInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(resizable_panels_layout_section(
                    look,
                    "resizable-panels-theme-inspector-box-model",
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
        let palette =
            ShadcnInspect::new(look).inspect_resizable_panels_color_palette(interaction_state(selection.state_id));
        resizable_panels_color_rows(&palette)
    }
}
pub use resizable_panels::{RESIZABLE_PANELS_INSPECTOR_SPEC, ResizablePanelsInspectorAdapter};

mod toolbar {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::toolbar_shell_color_rows;
    use super::super::super::common::{progress_enabled, toolbar_variant};
    use super::super::super::metrics::toolbar_layout_section;
    use super::super::super::specs::{CHOICE_SIZES, PROGRESS_STATES, TOOLBAR_VARIANTS};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static TOOLBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Toolbar",
        id_prefix: "toolbar-theme-inspector",
        parts: &[],
        variants: &TOOLBAR_VARIANTS,
        states: &PROGRESS_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "outline",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct ToolbarInspectorAdapter;

    impl ToolbarInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ToolbarInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(toolbar_layout_section(
                    look,
                    "toolbar-theme-inspector-box-model",
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
        let palette = ShadcnInspect::new(look)
            .inspect_toolbar_color_palette(progress_enabled(selection.state_id), toolbar_variant(selection.variant_id));
        toolbar_shell_color_rows(&palette)
    }
}
pub use toolbar::{TOOLBAR_INSPECTOR_SPEC, ToolbarInspectorAdapter};

mod pager {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnButtonStyle;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::collection::pager_shell_color_rows;
    use super::super::super::common::{interaction_state, pager_button_role, pager_shell_enabled, pager_style};
    use super::super::super::metrics::{pager_button_layout_section, pager_shell_layout_section};
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{PAGER_PARTS, PAGER_STATES, SELECTED_UNSELECTED_VALUES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        InspectorStateSpec, SharedInspectorResolver,
    };

    pub static PAGER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Pager",
        id_prefix: "pager-theme-inspector",
        parts: &PAGER_PARTS,
        variants: &[],
        states: &PAGER_STATES,
        sizes: &[],
        value_modes: &SELECTED_UNSELECTED_VALUES,
        default_part_id: "shell",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "selected",
    };

    pub struct PagerInspectorAdapter;

    impl PagerInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for PagerInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }

        fn state_applies(
            &self,
            _look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            state: &InspectorStateSpec,
        ) -> bool {
            match selection.part_id {
                "shell" => matches!(state.id, "enabled" | "disabled"),
                "button" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
                _ => true,
            }
        }

        fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
            selection.part_id == "button" && selection.variant_id == "page"
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        if selection.part_id == "button" {
            return resolve_button_color_rows(look, selection);
        }

        let palette = ShadcnInspect::new(look).inspect_pager_shell_color_palette(
            pager_shell_enabled(selection.state_id),
            pager_style(selection.variant_id),
        );
        pager_shell_color_rows(&palette)
    }

    fn resolve_button_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_button_color_palette(
            ShadcnButtonStyle::Outline,
            pager_button_role(selection.variant_id, selection.value_id),
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
    ) -> super::super::super::schema::InspectLayoutSection {
        match selection.part_id {
            "button" => pager_button_layout_section(look, "pager-theme-inspector-button-box-model"),
            _ => pager_shell_layout_section(look, "pager-theme-inspector-shell-box-model", selection.variant_id),
        }
    }
}
pub use pager::{PAGER_INSPECTOR_SPEC, PagerInspectorAdapter};

mod badge {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::badge_variant;
    use super::super::super::metrics::badge_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{BADGE_VARIANTS, CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub static BADGE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Badge",
        id_prefix: "badge-theme-inspector",
        parts: &[],
        variants: &BADGE_VARIANTS,
        states: &DEFAULT_INTERACTION_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "default",
        default_size_id: "md",
        default_value_id: "",
    };

    pub struct BadgeInspectorAdapter;

    impl BadgeInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for BadgeInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(badge_layout_section(
                    look,
                    "badge-theme-inspector-box-model",
                    selection.variant_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let palette = ShadcnInspect::new(look).inspect_badge_color_palette(badge_variant(selection.variant_id));
        let mut rows = vec![color_row("background", &palette.background), color_row("foreground", &palette.foreground)];
        if let Some(border) = &palette.border {
            rows.push(color_row("border", border));
        }
        rows
    }
}
pub use badge::{BADGE_INSPECTOR_SPEC, BadgeInspectorAdapter};
