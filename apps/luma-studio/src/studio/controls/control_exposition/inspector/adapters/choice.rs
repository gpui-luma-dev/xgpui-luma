//! Inspector adapters for the choice control family.

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
