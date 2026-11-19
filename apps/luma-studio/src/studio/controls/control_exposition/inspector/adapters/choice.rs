//! Inspector adapters for the choice control family.

mod toggle {
    use std::sync::Arc;

    use xgpui_luma::controls::button_family::ButtonFamilyRole;
    use xgpui_luma_look_shadcn::ShadcnLook;
    use xgpui_luma_look_shadcn::inspect::ShadcnInspect;

    use super::super::super::common::{choice_variant_style, interaction_state, value_flag};
    use super::super::super::metrics::toggle_layout_section;
    use super::super::super::provenance::color_row;
    use super::super::super::specs::{
        CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES, PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
        SELECTED_UNSELECTED_VALUES,
    };
    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    static TOGGLE_STATES: [super::super::super::schema::InspectorStateSpec; 4] = COLOR_LAYOUT_INTERACTION_STATES;

    pub static TOGGLE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Toggle",
        id_prefix: "toggle-theme-inspector",
        parts: &[],
        variants: &PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
        states: &TOGGLE_STATES,
        sizes: &CHOICE_SIZES,
        value_modes: &SELECTED_UNSELECTED_VALUES,
        default_part_id: "",
        default_variant_id: "primary",
        default_size_id: "md",
        default_value_id: "selected",
    };

    pub struct ToggleInspectorAdapter;

    impl ToggleInspectorAdapter {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for ToggleInspectorAdapter {
        fn resolve_category(
            &self,
            look: &ShadcnLook,
            selection: InspectorSelection<'_>,
            category_id: &str,
        ) -> InspectorCategoryContent {
            match category_id {
                "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
                "layout" => InspectorCategoryContent::Layout(toggle_layout_section(
                    look,
                    "toggle-theme-inspector-box-model",
                    selection.variant_id,
                    selection.size_id,
                )),
                _ => InspectorCategoryContent::Colors(Vec::new()),
            }
        }
    }

    fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
        let selected = value_flag(selection.value_id, "selected");
        let palette = ShadcnInspect::new(look).inspect_button_color_palette(
            choice_variant_style(selection.variant_id),
            ButtonFamilyRole::Toggle { selected },
            interaction_state(selection.state_id),
        );
        let rows = vec![
            color_row("background", &palette.background),
            color_row("foreground", &palette.foreground),
            color_row("border", &palette.border),
        ];
        rows
    }
}
pub use toggle::{TOGGLE_INSPECTOR_SPEC, ToggleInspectorAdapter};

mod motion_only {
    use std::sync::Arc;

    use xgpui_luma_look_shadcn::ShadcnLook;

    use super::super::super::schema::{
        ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
        SharedInspectorResolver,
    };

    pub struct MotionOnlyInspector;

    impl MotionOnlyInspector {
        pub fn shared() -> SharedInspectorResolver {
            Arc::new(Self)
        }
    }

    impl ControlInspectorResolver for MotionOnlyInspector {
        fn resolve_category(
            &self,
            _look: &ShadcnLook,
            _selection: InspectorSelection<'_>,
            _category_id: &str,
        ) -> InspectorCategoryContent {
            InspectorCategoryContent::Colors(Vec::new())
        }
    }

    pub static RADIO_GROUP_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Radio Group",
        id_prefix: "radio-group-theme-inspector",
        parts: &[],
        variants: &[],
        states: &[],
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "",
    };

    pub static SLIDE_PANEL_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Slide Panel",
        id_prefix: "slide-panel-theme-inspector",
        parts: &[],
        variants: &[],
        states: &[],
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "",
    };

    pub static TOGGLE_GROUP_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
        control_label: "Toggle Group",
        id_prefix: "toggle-group-theme-inspector",
        parts: &[],
        variants: &[],
        states: &[],
        sizes: &[],
        value_modes: &[],
        default_part_id: "",
        default_variant_id: "",
        default_size_id: "",
        default_value_id: "",
    };
}

pub use motion_only::{
    MotionOnlyInspector, RADIO_GROUP_INSPECTOR_SPEC, SLIDE_PANEL_INSPECTOR_SPEC, TOGGLE_GROUP_INSPECTOR_SPEC,
};
